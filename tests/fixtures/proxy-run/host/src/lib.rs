//! A host for the fixture guest: counters that count, and a log that prints.
mod bindings {
    wit_bindgen::generate!({
        path: ["../../proxy/deps/host.wit", "../../proxy/deps/log@0.1.0.wit", "wit"],
        world: "test:mockhost/mockhost",
        generate_all,
    });
}
use bindings::exports::test::host::types::{self, Color, Counter, Perms, Point, Shape};
use bindings::exports::test::log::sink;
use std::cell::Cell;

struct Host;
pub struct HostCounter(Cell<u32>);

impl types::Guest for Host {
    type Counter = HostCounter;
    fn make_point(x: i32, y: i32) -> Point {
        Point { x, y, label: None }
    }
    fn check(_p: Perms, _c: Color) -> Result<Shape, String> {
        Ok(Shape::Empty)
    }
    fn open(_name: String) -> Result<Counter, String> {
        Ok(Counter::new(HostCounter(Cell::new(0))))
    }
}
impl types::GuestCounter for HostCounter {
    fn new(start: u32) -> Self {
        HostCounter(Cell::new(start))
    }
    fn get(&self) -> u32 {
        self.0.get()
    }
    fn add(&self, n: u32) {
        self.0.set(self.0.get() + n)
    }
    fn merge(a: types::CounterBorrow<'_>, b: Counter) -> Counter {
        let sum = a.get::<HostCounter>().get() + b.get::<HostCounter>().get();
        Counter::new(HostCounter(Cell::new(sum)))
    }
}
impl sink::Guest for Host {
    fn write(msg: String) {
        eprintln!("[log] {msg}");
    }
}
bindings::export!(Host with_types_in bindings);
