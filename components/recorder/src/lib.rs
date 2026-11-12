// Regular builds use stock wasi-libc malloc. With 1-byte pages, libc_alloc makes it work;
// either way, don't add a #[global_allocator] (see crates/libc-alloc).
#[cfg(feature = "one-byte-page")]
extern crate libc_alloc;

mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "guest",
    });
}

use trace::Logger;
struct Component;
impl bindings::exports::proxy::recorder::record::Guest for Component {
    fn record_args(method: Option<String>, args: Vec<String>, is_export: bool) {
        let mut logger = Logger::new();
        let call = logger.record_args(method, args, is_export);
        emit(&call);
    }
    fn record_ret(method: Option<String>, ret: Option<String>, is_export: bool) {
        let mut logger = Logger::new();
        let call = logger.record_ret(method, ret, is_export);
        emit(&call);
    }
}

#[cfg(not(feature = "fastly"))]
fn emit(call: &trace::FuncCall) {
    println!("{}", serde_json::to_string(call).unwrap());
}

/// Writes each event to the service's `proxy-trace` log endpoint (any provider), as
/// `{"id":<sandbox id>,"seq":<n>,"event":<event>}`. Each event is written as it happens, so a
/// request that crashes keeps its trace up to the crash. Unlike `fastly log-tail`, log endpoints
/// don't tag lines with the sandbox, and mix many requests' lines in batches, possibly out of
/// order: `id` (`FASTLY_TRACE_ID`) and `seq` put each request's trace back together. Falls back
/// to stdout if the endpoint can't be opened.
#[cfg(feature = "fastly")]
fn emit(call: &trace::FuncCall) {
    use fastly_sys::service_0_1::fastly::compute::log::Endpoint;
    struct Sink {
        endpoint: Option<Endpoint>,
        id: String,
        seq: u64,
    }
    thread_local! {
        static SINK: RefCell<Sink> = RefCell::new(Sink {
            endpoint: Endpoint::open("proxy-trace").ok(),
            id: std::env::var("FASTLY_TRACE_ID").unwrap_or_default(),
            seq: 0,
        });
    }
    SINK.with_borrow_mut(|sink| {
        let line = serde_json::json!({ "id": sink.id, "seq": sink.seq, "event": call }).to_string();
        sink.seq += 1;
        match &sink.endpoint {
            Some(endpoint) => endpoint.write(line.as_bytes()),
            None => println!("{line}"),
        }
    });
}

use std::cell::RefCell;
thread_local! {
    static TRACE: RefCell<Option<Logger>> = RefCell::new(None);
}

fn load_trace() {
    let load = TRACE.with_borrow(|v| v.is_none());
    if load {
        TRACE.with_borrow_mut(|v| {
            use std::io::Read;
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            let mut logger = Logger::new();
            logger.load_trace(&input);
            *v = Some(logger);
        });
    }
}

impl bindings::exports::proxy::recorder::replay::Guest for Component {
    fn replay_export() -> Option<(String, Vec<String>)> {
        load_trace();
        TRACE.with_borrow_mut(|v| v.as_mut().unwrap().replay_export())
    }
    fn assert_export_ret(assert_method: Option<String>, assert_ret: Option<String>) {
        TRACE.with_borrow_mut(|v| {
            v.as_mut()
                .unwrap()
                .assert_export_ret(assert_method, assert_ret)
        });
    }
    fn replay_import(
        assert_method: Option<String>,
        assert_args: Option<Vec<String>>,
    ) -> Option<String> {
        TRACE.with_borrow_mut(|v| {
            let (_, ret) = v
                .as_mut()
                .unwrap()
                .replay_import(assert_method, assert_args, true);
            ret
        })
    }
}
bindings::export!(Component with_types_in bindings);
