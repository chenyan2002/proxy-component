//! A WASI p3 main component for record mode, whose async imports, streams and futures wasmtime
//! implements itself: `run` waits for `a` and `b` concurrently, with `wait-for`, and joins them in
//! the order its wakeups receive them, as the spike's guest does (see
//! tests/fixtures/spike-p3-race); `cat` copies stdin to stderr, through their streams; `race`
//! waits for `a` and `b`, and cancels the one that hasn't returned by the time the other has.
wit_bindgen::generate!({
    inline: r#"
        package test:rec@0.1.0;
        interface api {
            run: async func() -> string;
            cat: async func() -> string;
            race: async func() -> string;
        }
        world root {
            export api;
        }
    "#,
});

use std::future::Future;
use std::task::Poll;

/// Waits `DELAY_<KEY>` milliseconds, or a random 0-19, and returns the key. WASI 0.3's
/// environment, a sync function that doesn't suspend, unlike WASI 0.2's.
async fn wait(key: &str) -> String {
    let name = format!("DELAY_{}", key.to_uppercase());
    let delay = wasip3::cli::environment::get_environment()
        .into_iter()
        .find(|(n, _)| *n == name)
        .and_then(|(_, delay)| delay.parse::<u64>().ok())
        .unwrap_or_else(|| u64::from(wasip3::random::random::get_random_bytes(1)[0] % 20));
    wasip3::clocks::monotonic_clock::wait_for(delay * 1_000_000).await;
    key.to_uppercase()
}

struct Guest;

impl exports::test::rec::api::Guest for Guest {
    async fn run() -> String {
        let mut a = Box::pin(wait("a"));
        let mut b = Box::pin(wait("b"));
        let (mut a_done, mut b_done) = (false, false);
        let mut out = String::new();
        // What each wakeup received, returned with the result rather than printed: printing is a
        // sync call to the host, which can suspend the guest and change the schedule.
        let mut wakeups = Vec::new();
        // Both are polled on each wakeup, `a` first: if both are ready, `a` comes first.
        std::future::poll_fn(|cx| {
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
            if a_done && b_done { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;
        format!("{out} (wakeups: {})", wakeups.join(" "))
    }
    async fn race() -> String {
        let mut a = Box::pin(wait("a"));
        let mut b = Box::pin(wait("b"));
        let first = std::future::poll_fn(|cx| {
            if let Poll::Ready(value) = a.as_mut().poll(cx) {
                return Poll::Ready(value);
            }
            b.as_mut().poll(cx)
        })
        .await;
        // Drops the other, which cancels it if it's still pending.
        drop((a, b));
        format!("{first} first")
    }
    async fn cat() -> String {
        let (mut stdin, stdin_done) = wasip3::cli::stdin::read_via_stream();
        let (mut writer, reader) = wasip3::wit_stream::new::<u8>();
        let stderr_done = wasip3::cli::stderr::write_via_stream(reader);
        let (mut bytes, mut reads) = (0, 0);
        loop {
            let (result, buf) = stdin.read(Vec::with_capacity(16)).await;
            if !buf.is_empty() {
                bytes += buf.len();
                reads += 1;
                writer.write_all(buf).await;
            }
            if let wit_bindgen::StreamResult::Dropped = result {
                break;
            }
        }
        drop(writer);
        let (stdin_done, stderr_done) = (stdin_done.await, stderr_done.await);
        format!("{bytes} bytes in {reads} reads ({stdin_done:?}, {stderr_done:?})")
    }
}

export!(Guest);
