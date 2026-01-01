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
}
bindings::export!(Component with_types_in bindings);
