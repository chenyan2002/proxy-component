mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "crate-debug",
    });
}

use bindings::exports::proxy::recorder::debug::Guest;
struct Component;
impl Guest for Component {
    fn print(s: String) {
        println!("{}", s);
    }
}
bindings::export!(Component with_types_in bindings);
