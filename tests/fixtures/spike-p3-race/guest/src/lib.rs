//! `p3-race` from component-ui's docs/p3.md: fetches "a" and "b" concurrently, and joins the
//! results in the order they complete.
wit_bindgen::generate!({ path: "../wit", world: "race-guest" });

use spike::race::host::fetch;
use std::future::Future;
use std::task::Poll;

struct Guest;

impl exports::spike::race::guest::Guest for Guest {
    async fn run() -> String {
        let mut a = Box::pin(fetch("a".to_string()));
        let mut b = Box::pin(fetch("b".to_string()));
        let (mut a_done, mut b_done) = (false, false);
        let mut out = String::new();
        // What each wakeup received, returned with the result rather than printed: printing is a
        // sync call to the host, which can suspend the guest and change the schedule.
        let mut wakeups = Vec::new();
        // `BUSY_MS`: how long to keep running after starting both calls, so that both results
        // can arrive before the guest next waits.
        let mut busy = std::env::var("BUSY_MS").ok().and_then(|ms| ms.parse::<u64>().ok());
        // Both are polled on each wakeup, `a` first: if both are ready, `a` comes first.
        std::future::poll_fn(|cx| {
            // What this wakeup received, which the replay must see alike.
            let mut got = String::new();
            if !a_done && let Poll::Ready(value) = a.as_mut().poll(cx) {
                got += &value;
                a_done = true;
            }
            if !b_done && let Poll::Ready(value) = b.as_mut().poll(cx) {
                got += &value;
                b_done = true;
            }
            if !got.is_empty() {
                wakeups.push(got.clone());
            }
            out += &got;
            if let Some(ms) = busy.take() {
                let start = std::time::Instant::now();
                while start.elapsed() < std::time::Duration::from_millis(ms) {}
            }
            if a_done && b_done { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;
        format!("{out} (wakeups: {})", wakeups.join(" "))
    }
}

export!(Guest);
