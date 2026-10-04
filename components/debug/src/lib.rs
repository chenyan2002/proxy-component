// Regular builds use stock wasi-libc malloc. With 1-byte pages, libc_alloc makes it work;
// either way, don't add a #[global_allocator] (see crates/libc-alloc).
#[cfg(feature = "one-byte-page")]
extern crate libc_alloc;

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
