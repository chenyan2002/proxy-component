use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;
use trace::{Replay, Trace};
use wasmtime::component::types::{ComponentExtern, ComponentFunc, ComponentItem as CItem};
use wasmtime::component::wasm_wave::{untyped::UntypedFuncCall, wasm::WasmFunc};
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable, Val};
use wasmtime::*;
use wasmtime_wasi::runtime::in_tokio;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView, p2::add_to_linker_async};

#[derive(Parser)]
pub struct RunArgs {
    /// The path to the wasm component file.
    wasm_file: PathBuf,
    /// Invoke an exported function to record execution
    #[arg(short, long)]
    invoke: Option<String>,
    /// Replay a trace file
    #[arg(short, long)]
    trace: Option<PathBuf>,
}

mod bindings {
    wasmtime::component::bindgen!({
        path: "assets/recorder.wit",
        world: "host",
        // To convert the lines to `crates/trace`'s types, which serialize alike (`trace::convert`).
        additional_derives: [serde::Serialize, serde::Deserialize],
    });
}

use bindings::proxy::recorder::types::Line;

pub struct State {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    // The lines recorded, which `run` writes to `trace.out` at the end.
    recorded: Vec<trace::Line>,
    replay: Option<Replay>,
    exit_called: bool,
}
impl bindings::proxy::recorder::types::Host for State {}
impl bindings::proxy::recorder::record::Host for State {
    fn record(&mut self, line: Line) {
        let line: trace::Line = trace::convert(line);
        println!("{line}");
        self.recorded.push(line);
    }
}
impl bindings::proxy::recorder::replay::Host for State {
    fn next_host_line(&mut self) -> Option<Line> {
        let line = self.replay.as_mut().unwrap().next_host_line()?;
        println!("{line}");
        Some(trace::convert(line))
    }
    fn take_host_line(&mut self, key: bindings::proxy::recorder::types::LineKey) -> Option<Line> {
        let key: trace::LineKey = trace::convert(key);
        let line = self.replay.as_mut().unwrap().take_host_line(&key)?;
        println!("{line}");
        Some(trace::convert(line))
    }
    fn finish(&mut self) {
        self.replay.as_ref().unwrap().finish();
    }
    fn check(&mut self, line: Line) -> Vec<Line> {
        let line: trace::Line = trace::convert(line);
        println!("{line}");
        let mut answers = self.replay.as_mut().unwrap().check(&line);
        for answer in &answers {
            println!("{answer}");
        }
        // A replayed `wasi:cli/exit` traps the guest on purpose: the import has no result, so
        // the mock traps on one.
        if line.exit_status().is_some() {
            self.exit_called = true;
            let id = line.event.task().unwrap();
            let crash = trace::Event::Return(trace::ReturnValue {
                id,
                value: Some("Something that can crash".to_string()),
            });
            answers = vec![trace::Line {
                by: trace::Party::Host,
                at: None,
                event: crash,
            }];
        }
        trace::convert(answers)
    }
}

const MAX_FUEL: u64 = u64::MAX;

pub fn run(args: RunArgs) -> anyhow::Result<()> {
    // Patch ctrlc until https://github.com/console-rs/dialoguer/issues/77 is fixed
    let _ = ctrlc::try_set_handler(move || {
        let term = dialog::console::Term::stdout();
        let _ = term.show_cursor();
    });
    let mut config = Config::new();
    config
        .consume_fuel(true)
        .wasm_custom_page_sizes(true)
        // The generated `start` is async, and WASI p3 components have async exports and imports.
        .wasm_component_model_async(true)
        .wasm_component_model_error_context(true)
        .concurrency_support(true)
        //.debug_info(true)
        .wasm_backtrace_details(WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config)?;

    let mut linker = Linker::<State>::new(&engine);
    // The WASI p2 host functions are async ones, as the store runs async exports.
    add_to_linker_async(&mut linker)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut state = State {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        recorded: Vec::new(),
        replay: None,
        exit_called: false,
    };
    dialog_bindings::proxy::util::dialog::add_to_linker::<State, HasSelf<State>>(
        &mut linker,
        |state| state,
    )?;
    if let Some(path) = &args.trace {
        bindings::proxy::recorder::replay::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| {
            state
        })?;
        let trace = std::fs::read_to_string(path)?;
        state.replay = Some(Replay::new(Trace::parse(&trace)));
    } else {
        bindings::proxy::recorder::record::add_to_linker::<State, HasSelf<State>>(
            &mut linker,
            |state| state,
        )?;
    }

    let mut store = Store::new(&engine, state);
    store.set_fuel(MAX_FUEL)?;
    let component = Component::from_file(&engine, args.wasm_file)?;
    if let Some(invoke) = &args.invoke {
        let untyped_call = UntypedFuncCall::parse(invoke)?;
        let exports = collect_export_funcs(&engine, &component);
        //println!("Exported funcs: {exports:?}");
        let (names, func_type) = exports
            .into_iter()
            .find(|(names, _)| names.last().is_some_and(|name| name == untyped_call.name()))
            .with_context(|| format!("no exported function named `{}`", untyped_call.name()))?;
        let export = names
            .iter()
            .fold(None, |instance, name| {
                component.get_export_index(instance.as_ref(), name)
            })
            .unwrap();
        let instance = in_tokio(linker.instantiate_async(&mut store, &component))?;

        let param_types = WasmFunc::params(&func_type).collect::<Vec<_>>();
        let params = untyped_call.to_wasm_params(&param_types)?;
        let func = instance.get_func(&mut store, export).unwrap();
        let mut results = vec![Val::Bool(false); func_type.results().len()];
        // A replayed `wasi:cli/exit` traps the guest on purpose.
        in_tokio(func.call_async(&mut store, &params, &mut results)).or_else(|e| {
            if store.data().exit_called {
                Ok(())
            } else {
                Err(e)
            }
        })?;
        if args.trace.is_none() {
            let lines: Vec<_> = store.data().recorded.iter().map(trace::Line::to_json).collect();
            std::fs::write("trace.out", lines.join("\n"))?;
        }
    }
    let fuel = MAX_FUEL - store.get_fuel()?;
    println!("Executed {fuel} Wasm instructions.");
    Ok(())
}

fn collect_exports(
    engine: &Engine,
    item: CItem,
    basename: Vec<String>,
) -> Vec<(Vec<String>, CItem)> {
    let exports: Box<dyn Iterator<Item = (&str, ComponentExtern<'_>)> + '_> = match &item {
        CItem::Component(c) => Box::new(c.exports(engine)),
        CItem::ComponentInstance(c) => Box::new(c.exports(engine)),
        _ => return vec![(basename, item)],
    };
    exports
        .flat_map(|(name, export)| {
            let mut names = basename.clone();
            names.push(name.to_string());
            collect_exports(engine, export.ty, names)
        })
        .collect()
}
fn collect_export_funcs(
    engine: &Engine,
    component: &Component,
) -> Vec<(Vec<String>, ComponentFunc)> {
    collect_exports(
        engine,
        CItem::Component(component.component_type()),
        Vec::new(),
    )
    .into_iter()
    .filter_map(|(names, item)| {
        let CItem::ComponentFunc(func) = item else {
            return None;
        };
        Some((names, func))
    })
    .collect()
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

mod dialog_bindings {
    wasmtime::component::bindgen!({
        path: "assets/util.wit",
        world: "host-dialog",
    });
}

/// Forwards `dialog` host functions of the form `fn(&mut self, dep: u32) -> String`.
macro_rules! forward_dialog_reads {
    ($($name:ident),* $(,)?) => {
        $(
            fn $name(&mut self, dep: u32) -> String {
                dialog::$name(dep)
            }
        )*
    };
}
impl dialog_bindings::proxy::util::dialog::Host for crate::run::State {
    fn print(&mut self, dep: u32, message: String) {
        dialog::print(dep, &message);
    }
    forward_dialog_reads!(
        read_string,
        read_u8,
        read_u16,
        read_u32,
        read_u64,
        read_s8,
        read_s16,
        read_s32,
        read_s64,
        read_f32,
        read_f64,
        read_bool,
        read_char,
    );
    fn read_select(&mut self, dep: u32, prompt: String, items: Vec<String>) -> u32 {
        dialog::read_select(dep, prompt, items)
    }
    fn read_multi_select(&mut self, dep: u32, prompt: String, items: Vec<String>) -> Vec<u32> {
        dialog::read_multi_select(dep, prompt, items)
    }
    fn read_num(&mut self, dep: u32, prompt: String) -> u32 {
        dialog::read_num(dep, prompt)
    }
    fn read_raw_string(&mut self, dep: u32, prompt: String) -> String {
        dialog::read_raw_string(dep, prompt)
    }
}
