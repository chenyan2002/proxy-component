// Regular builds use stock wasi-libc malloc. With 1-byte pages, libc_alloc makes it work;
// either way, don't add a #[global_allocator] (see crates/libc-alloc).
#[cfg(feature = "one-byte-page")]
extern crate libc_alloc;

mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "guest",
        // The trace's types, which `crates/trace` generates from the same WIT.
        with: { "proxy:recorder/types@0.1.0": trace },
    });
}

use std::cell::RefCell;
use trace::{Line, LineKey, Replay, Trace};

struct Component;
impl bindings::exports::proxy::recorder::record::Guest for Component {
    // Prints the stored form of the line, so that stdout is the trace.
    fn record(line: Line) {
        println!("{}", line.to_json());
    }
}

thread_local! {
    // The trace being replayed, read from stdin on first use.
    static REPLAY: RefCell<Option<Replay>> = const { RefCell::new(None) };
}

fn with_replay<R>(f: impl FnOnce(&mut Replay) -> R) -> R {
    REPLAY.with_borrow_mut(|replay| {
        let replay = replay.get_or_insert_with(|| {
            use std::io::Read;
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            Replay::new(Trace::parse(&input))
        });
        f(replay)
    })
}

impl bindings::exports::proxy::recorder::replay::Guest for Component {
    fn next_host_line() -> Option<Line> {
        let line = with_replay(|replay| replay.next_host_line())?;
        println!("{line}");
        Some(line)
    }
    fn take_host_line(key: LineKey) -> Option<Line> {
        let line = with_replay(|replay| replay.take_host_line(&key))?;
        println!("{line}");
        Some(line)
    }
    fn finish() {
        with_replay(|replay| replay.finish());
    }
    fn check(line: Line) -> Vec<Line> {
        println!("{line}");
        // A replayed `wasi:cli/exit` ends the replay with the guest's status.
        if let Some(status) = line.exit_status() {
            with_replay(|replay| replay.check(&line));
            std::process::exit(status);
        }
        let answers = with_replay(|replay| replay.check(&line));
        for answer in &answers {
            println!("{answer}");
        }
        answers
    }
}
bindings::export!(Component with_types_in bindings);
