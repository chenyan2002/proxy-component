//! Plays the host of the fixture guest: makes sessions, gives one a counter, and drops them.
mod bindings {
    wit_bindgen::generate!({
        path: ["../../proxy/deps/host.wit", "../../proxy/deps/guest.wit", "wit"],
        world: "test:driver/driver",
        generate_all,
    });
}
use bindings::test::guest::api::{self, Session};
use bindings::test::host::types::Counter;

struct Driver;
impl bindings::Guest for Driver {
    fn run() {
        let a = Session::new("a");
        let b = api::begin(Counter::new(5));
        drop(a);
        drop(b);
    }
}
bindings::export!(Driver with_types_in bindings);
