#[cfg(all(not(target_feature = "atomics"), target_family = "wasm"))]
#[global_allocator]
static TALC: talc::wasm::WasmArenaTalc = {
    use core::mem::MaybeUninit;
    static mut MEMORY: [MaybeUninit<u8>; 0x80000] = [MaybeUninit::uninit(); 0x80000];
    // SAFETY: the memory for MEMORY is never modified externally. It's the allocator's.
    unsafe { talc::wasm::new_wasm_arena_allocator(&raw mut MEMORY) }
};

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
        let json = serde_json::to_string(&call).unwrap();
        println!("{json}");
    }
    fn record_ret(method: Option<String>, ret: Option<String>, is_export: bool) {
        let mut logger = Logger::new();
        let call = logger.record_ret(method, ret, is_export);
        let json = serde_json::to_string(&call).unwrap();
        println!("{json}");
    }
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
