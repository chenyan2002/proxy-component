use clap::Parser;
use std::collections::VecDeque;
use std::path::PathBuf;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView, add_to_linker_sync};

#[derive(Parser)]
pub struct RunArgs {
    /// The path to the wasm component file.
    wasm_file: PathBuf,
}

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "host",
    });
}

struct Logger {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    logger: VecDeque<(String, String, String)>,
}
impl bindings::proxy::recorder::record::Host for Logger {
    fn record(&mut self, method: String, input: String, output: String) {
        self.logger.push_back((method, input, output));
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
    add_to_linker_sync(&mut linker)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();
    let state = Logger {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
        logger: VecDeque::new(),
    };
    let mut store = Store::new(&engine, state);

    let component = Component::from_file(&engine, args.wasm_file)?;
    let bindings = bindings::Host_::instantiate(&mut store, &component, &linker)?;
    use bindings::exports::docs::calculator::calculate::Op;
    bindings
        .docs_calculator_calculate()
        .call_eval_expression(&mut store, Op::Add, 3, 4)?;
    println!("Trace: {:?}", store.data().logger);
    Ok(())
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
