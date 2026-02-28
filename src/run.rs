use clap::Parser;
use std::path::PathBuf;
use trace::Logger;
use wasmtime::component::types::{ComponentFunc, ComponentItem as CItem};
use wasmtime::component::wasm_wave::{untyped::UntypedFuncCall, wasm::WasmFunc};
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable, Val};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView, p2::add_to_linker_sync};

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
    });
}

pub struct State {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    logger: Logger,
    exit_called: bool,
}
impl bindings::proxy::recorder::record::Host for State {
    fn record_args(&mut self, method: Option<String>, args: Vec<String>, is_export: bool) {
        let call = self.logger.record_args(method, args, is_export);
        println!("call: {}", call.to_string());
    }
    fn record_ret(&mut self, method: Option<String>, ret: Option<String>, is_export: bool) {
        let call = self.logger.record_ret(method, ret, is_export);
        println!("ret: {}", call.to_string());
    }
}
impl bindings::proxy::recorder::replay::Host for State {
    fn replay_export(&mut self) -> Option<(String, Vec<String>)> {
        self.logger.replay_export()
    }
    fn assert_export_ret(&mut self, assert_method: Option<String>, assert_ret: Option<String>) {
        self.logger.assert_export_ret(assert_method, assert_ret);
    }
    fn replay_import(
        &mut self,
        assert_method: Option<String>,
        assert_args: Option<Vec<String>>,
    ) -> Option<String> {
        let (exit_called, ret) = self.logger.replay_import(assert_method, assert_args, false);
        if exit_called {
            self.exit_called = exit_called;
            return Some("Something that can crash".to_string());
        }
        ret
    }
}

const MAX_FUEL: u64 = u64::MAX;

pub fn run(args: RunArgs) -> anyhow::Result<()> {
    let mut config = Config::new();
    config
        .consume_fuel(true)
        //.debug_info(true)
        .wasm_backtrace_details(WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config)?;

    let mut linker = Linker::<State>::new(&engine);
    add_to_linker_sync(&mut linker)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut state = State {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        logger: Logger::new(),
        exit_called: false,
    };
    dialog::proxy::util::dialog::add_to_linker::<State, HasSelf<State>>(&mut linker, |state| {
        state
    })?;
    if let Some(path) = &args.trace {
        bindings::proxy::recorder::replay::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| {
            state
        })?;
        let trace = std::fs::read_to_string(path)?;
        state.logger.load_trace(&trace);
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
        let mut find_export = exports.into_iter().filter_map(|(names, func)| {
            let func_name = names.last().unwrap();
            (func_name == untyped_call.name()).then_some((names, func))
        });
        let (names, func_type) = &find_export.next().unwrap();
        let export = names
            .iter()
            .fold(None, |instance, name| {
                component.get_export_index(instance.as_ref(), name)
            })
            .unwrap();
        let instance = linker.instantiate(&mut store, &component)?;

        let param_types = WasmFunc::params(func_type).collect::<Vec<_>>();
        let params = untyped_call.to_wasm_params(&param_types)?;
        let func = instance.get_func(&mut store, export).unwrap();
        let mut results = vec![Val::Bool(false); func_type.results().len()];
        match func.call(&mut store, &params, &mut results) {
            Ok(_) => Ok(()),
            Err(e) => {
                if store.data().exit_called {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }?;
        if args.trace.is_none() {
            let trace = store.data().logger.dump_trace();
            std::fs::write("trace.out", &trace)?;
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
    match item {
        CItem::Component(c) => c
            .exports(engine)
            .flat_map(move |(name, item)| {
                let mut names = basename.clone();
                names.push(name.to_string());
                collect_exports(engine, item, names)
            })
            .collect(),
        CItem::ComponentInstance(c) => c
            .exports(engine)
            .flat_map(move |(name, item)| {
                let mut names = basename.clone();
                names.push(name.to_string());
                collect_exports(engine, item, names)
            })
            .collect(),
        _ => vec![(basename, item)],
    }
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

mod dialog {
    wasmtime::component::bindgen!({
        path: "assets/util.wit",
        world: "host-dialog",
    });
}

impl dialog::proxy::util::dialog::Host for crate::run::State {
    fn input(&mut self, message: String) -> String {
        message
    }
    fn prompt(&mut self, message: String) {
        println!("{message}");
    }
}
