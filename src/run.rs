use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use wasmtime::component::types::{ComponentFunc, ComponentItem as CItem};
use wasmtime::component::wasm_wave::{untyped::UntypedFuncCall, wasm::WasmFunc};
use wasmtime::component::{Component, Linker, Resource, ResourceTable, Val};
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

pub struct Handle;
mod bindings {
    wasmtime::component::bindgen!({
        // TODO: change to assets/recorder.wit
        path: "wit",
        world: "host",
        with: {
            "wasi:io/io/handle": crate::run::Handle,
        }
    });
}

#[derive(Serialize, Deserialize, Debug)]
enum FuncCall {
    ExportArgs {
        method: String,
        args: Vec<String>,
    },
    ExportRet {
        method: Option<String>,
        ret: Option<String>,
    },
    ImportArgs {
        method: Option<String>,
        args: Vec<String>,
    },
    ImportRet {
        method: Option<String>,
        ret: Option<String>,
    },
}
impl FuncCall {
    fn to_string(&self) -> String {
        match self {
            FuncCall::ExportArgs { method, args } => format!("{method}({})", args.join(", ")),
            FuncCall::ImportArgs { method, args } => format!(
                "{}({})",
                method.as_deref().unwrap_or("<unknown>"),
                args.join(", ")
            ),
            FuncCall::ExportRet { ret, .. } | FuncCall::ImportRet { ret, .. } => {
                ret.as_deref().unwrap_or("()").to_owned()
            }
        }
    }
}

struct Logger {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    logger: VecDeque<FuncCall>,
}
impl bindings::proxy::recorder::record::Host for Logger {
    fn record_args(&mut self, method: Option<String>, args: Vec<String>, is_export: bool) {
        let call = if is_export {
            FuncCall::ExportArgs {
                method: method.unwrap(),
                args,
            }
        } else {
            FuncCall::ImportArgs { method, args }
        };
        println!("call: {}", call.to_string());
        self.logger.push_back(call);
    }
    fn record_ret(&mut self, method: Option<String>, ret: Option<String>, is_export: bool) {
        let call = if is_export {
            FuncCall::ExportRet { method, ret }
        } else {
            FuncCall::ImportRet { method, ret }
        };
        println!("ret: {}", call.to_string());
        self.logger.push_back(call);
    }
}
impl bindings::proxy::recorder::replay::Host for Logger {
    fn replay_export(&mut self) -> Option<(String, Vec<String>)> {
        let call = self.logger.pop_front()?;
        println!("export call: {}", call.to_string());
        let FuncCall::ExportArgs { method, args } = call else {
            panic!()
        };
        Some((method, args))
    }
    fn assert_export_ret(&mut self, assert_method: Option<String>, assert_ret: Option<String>) {
        if let Some(FuncCall::ExportRet { .. }) = self.logger.get(0) {
            let call = self.logger.pop_front().unwrap();
            println!("export ret: {}", call.to_string());
            let FuncCall::ExportRet { method, ret } = call else {
                panic!()
            };
            if let (Some(method), Some(assert_method)) = (method, assert_method) {
                assert_eq!(method, assert_method);
            }
            assert_eq!(ret, assert_ret);
        }
    }
    fn replay_import(
        &mut self,
        assert_method: Option<String>,
        assert_args: Option<Vec<String>>,
    ) -> Option<String> {
        let mut call = self.logger.pop_front().unwrap();
        if let FuncCall::ImportArgs { method, args } = &call {
            if let (Some(method), Some(assert_method)) = (method, assert_method) {
                assert_eq!(method, &assert_method);
            }
            if let Some(assert_args) = assert_args {
                assert_eq!(args, &assert_args);
            }
            println!("import call: {}", call.to_string());
            call = self.logger.pop_front().unwrap();
        }
        println!("import ret: {}", call.to_string());
        let FuncCall::ImportRet { ret, .. } = call else {
            panic!()
        };
        ret
    }
}
impl bindings::docs::adder::add::Host for Logger {
    fn add(&mut self, a: u32, b: u32) -> u32 {
        a + b
    }
}
impl bindings::wasi::io::io::HostHandle for Logger {
    fn get(&mut self) -> Resource<Handle> {
        self.resource_table.push(Handle).unwrap()
    }
    fn write(&mut self, _: Resource<Handle>, _: String) {}
    fn drop(&mut self, _: Resource<Handle>) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
impl bindings::wasi::io::io::Host for Logger {}

const MAX_FUEL: u64 = u64::MAX;

pub fn run(args: RunArgs) -> anyhow::Result<()> {
    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;

    let mut linker = Linker::<Logger>::new(&engine);
    add_to_linker_sync(&mut linker)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let mut state = Logger {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        logger: VecDeque::new(),
    };
    if let Some(path) = &args.trace {
        bindings::proxy::recorder::replay::add_to_linker(&mut linker, |logger| logger)?;
        let trace = std::fs::read_to_string(path)?;
        state.logger = serde_json::from_str(&trace)?;
    } else {
        bindings::docs::adder::add::add_to_linker(&mut linker, |logger| logger)?;
        bindings::proxy::recorder::record::add_to_linker::<Logger, Logger>(
            &mut linker,
            |logger| logger,
        )?;
        bindings::wasi::io::io::add_to_linker(&mut linker, |logger| logger)?;
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
        func.call(&mut store, &params, &mut results)?;
        if args.trace.is_none() {
            let trace = serde_json::to_string(&store.data().logger)?;
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
