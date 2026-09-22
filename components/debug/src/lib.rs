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
        path: "../../assets/util.wit",
        world: "crate-debug",
    });
}

use bindings::exports::proxy::util::debug::Guest;
struct Component;
impl Guest for Component {
    fn print(s: String) {
        println!("{}", s);
    }
    fn eprint(s: String) {
        eprintln!("{}", s);
    }
    fn get_random() -> Vec<u8> {
        let mut data = vec![0u8; 1024];
        getrandom::fill(&mut data).unwrap();
        data
    }
}
bindings::export!(Component with_types_in bindings);
