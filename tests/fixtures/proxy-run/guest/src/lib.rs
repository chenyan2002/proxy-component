//! A main component for the WIT in `tests/fixtures/proxy`, which drops and moves resources both
//! ways, to test that record mode records exactly the drops, and replay replays them. `make
//! test-fixture` runs it with `../host` and `../driver`, see `../compose.wac`:
//! - `new session` drops a counter it made, and moves another to the host with `merge`;
//! - `run` drops a counter it opened (not called: record mode doesn't support the borrowed host
//!   resources of exports);
//! - `begin` moves a host counter into the guest, which the session keeps, and uses it, so that
//!   the trace labels it alike where the host gives it and where the guest uses it;
//! - dropping a session logs, then drops its counter: imports inside an export's drop.
mod bindings {
    wit_bindgen::generate!({
        path: "../../proxy",
        world: "root",
        generate_all,
    });
}

use bindings::exports::test::guest::api::{Guest, GuestSession, Point, Shape};
use bindings::test::host::types::{self, Counter};
use bindings::test::log0_1_0::sink;

struct Component;

impl Guest for Component {
    type Session = Session;
    fn run(p: Point, c: &Counter) -> Vec<u8> {
        c.add(p.x as u32);
        // Dropped at the end of the statement
        let _ = types::open("tmp");
        c.get().to_le_bytes().to_vec()
    }
    fn classify(s: Shape) -> Option<Point> {
        match s {
            Shape::Rect(p) => Some(p),
            _ => None,
        }
    }
    fn begin(c: Counter) -> bindings::exports::test::guest::api::Session {
        c.add(1);
        bindings::exports::test::guest::api::Session::new(Session {
            name: "begun".to_string(),
            counter: Some(c),
        })
    }
}

pub struct Session {
    name: String,
    counter: Option<Counter>,
}

impl GuestSession for Session {
    fn new(name: String) -> Self {
        let a = Counter::new(1);
        let b = Counter::new(2);
        // Moves `b` to the host; `a` is dropped on return.
        let merged = Counter::merge(&a, b);
        Session {
            name,
            counter: Some(merged),
        }
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        sink::write(&format!("drop session {}", self.name));
        drop(self.counter.take());
    }
}

bindings::export!(Component with_types_in bindings);
