use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use wasmtime::component::types::{ComponentFunc, ComponentItem as CItem};
use wasmtime::component::wasm_wave::{untyped::UntypedFuncCall, wasm::WasmFunc};
use wasmtime::component::{Component, Linker, ResourceTable, Val};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView, add_to_linker_sync};

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
        // TODO: change to assets/recorder.wit
        path: "wit",
        world: "host",
    });
}

#[derive(Serialize, Deserialize, Debug)]
enum FuncCall {
    ExportArgs {
        method: String,
        args: String,
    },
    ExportRet {
        method: Option<String>,
        ret: String,
    },
    ImportArgs {
        method: Option<String>,
        args: String,
    },
    ImportRet {
        method: Option<String>,
        ret: String,
    },
}

struct Logger {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    logger: VecDeque<FuncCall>,
}
impl bindings::proxy::recorder::record::Host for Logger {
    fn record_args(&mut self, method: Option<String>, args: String, is_export: bool) {
        let call = if is_export {
            FuncCall::ExportArgs {
                method: method.unwrap(),
                args,
            }
        } else {
            FuncCall::ImportArgs { method, args }
        };
        println!("{:?}", call);
        self.logger.push_back(call);
    }
    fn record_ret(&mut self, method: Option<String>, ret: String, is_export: bool) {
        let call = if is_export {
            FuncCall::ExportRet { method, ret }
        } else {
            FuncCall::ImportRet { method, ret }
        };
        println!("{:?}", call);
        self.logger.push_back(call);
    }
}
impl bindings::proxy::recorder::replay::Host for Logger {
    fn replay(
        &mut self,
        _method: Option<String>,
        _assert_input: Option<String>,
        is_export: bool,
    ) -> String {
        let mut call = self.logger.pop_front().unwrap();
        if is_export {
            String::new()
        } else {
            while !matches!(call, FuncCall::ImportRet { .. }) {
                println!("Skip {:?}", call);
                call = self.logger.pop_front().unwrap();
            }
            let FuncCall::ImportRet { ret, .. } = call else {
                unreachable!()
            };
            println!("ret: {ret}");
            ret
        }
    }
}
impl bindings::docs::adder::add::Host for Logger {
    fn add(&mut self, a: u32, b: u32) -> u32 {
        a + b
    }
}

pub fn run(args: RunArgs) -> anyhow::Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::<Logger>::new(&engine);
    bindings::proxy::recorder::record::add_to_linker::<Logger, Logger>(&mut linker, |logger| {
        logger
    })?;
    bindings::docs::adder::add::add_to_linker(&mut linker, |logger| logger)?;
    bindings::proxy::recorder::replay::add_to_linker(&mut linker, |logger| logger)?;
    add_to_linker_sync(&mut linker)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut state = Logger {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        logger: VecDeque::new(),
    };
    if let Some(path) = &args.trace {
        let trace = std::fs::read_to_string(path)?;
        let trace: VecDeque<FuncCall> = serde_json::from_str(&trace)?;
        state.logger = trace;
    }

    let mut store = Store::new(&engine, state);
    let component = Component::from_file(&engine, args.wasm_file)?;
    if let Some(invoke) = &args.invoke {
        let untyped_call = UntypedFuncCall::parse(invoke)?;
        let exports = collect_export_funcs(&engine, &component);
        println!("Exported funcs: {exports:?}");
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
        func.call(&mut store, &params, &mut results)?;
        if args.trace.is_none() {
            let trace = serde_json::to_string(&store.data().logger)?;
            std::fs::write("trace.out", &trace)?;
        }
    }
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

impl IoView for Logger {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}
impl WasiView for Logger {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}
