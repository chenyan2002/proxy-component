//! Forwards the guest's `fetch` to the host, and records each host event with the executor's
//! wakeup that observed it.
//!
//! The host is wasmtime's own host functions, as in record mode: `fetch(key)` waits `DELAY_<KEY>`
//! milliseconds, or a random 0-19, with `wait-for`. With the `component-host` feature, it is the
//! `race-host` component instead. wasmtime runs a call into another component as a queued work
//! item, which suspends the caller while it's still entered, so the guest sees a call the proxy
//! saw RETURNED as STARTED, and its next call into the proxy waits; replay can't reproduce that.
#[cfg(not(feature = "component-host"))]
wit_bindgen::generate!({ path: "../wit", world: "proxy-record" });
#[cfg(feature = "component-host")]
wit_bindgen::generate!({ path: "../wit", world: "proxy-record-component-host" });

#[path = "../../proxy-common.rs"]
mod common;
use common::*;
use std::future::Future;
use std::task::Poll;

#[cfg(feature = "component-host")]
async fn host_fetch(key: String) -> String {
    spike::race::upstream::fetch(key).await
}
/// `fetch` on wasmtime's own host functions. WASI 0.3's environment, a sync function that
/// doesn't suspend, unlike WASI 0.2's.
#[cfg(not(feature = "component-host"))]
async fn host_fetch(key: String) -> String {
    let name = format!("DELAY_{}", key.to_uppercase());
    let delay = wasip3::cli::environment::get_environment()
        .into_iter()
        .find(|(n, _)| *n == name)
        .and_then(|(_, delay)| delay.parse::<u64>().ok())
        .unwrap_or_else(|| u64::from(wasip3::random::random::get_random_bytes(1)[0] % 20));
    if delay > 0 {
        wasip3::clocks::monotonic_clock::wait_for(delay * 1_000_000).await;
    }
    key.to_uppercase()
}

struct Proxy;

impl exports::spike::race::host::Guest for Proxy {
    async fn fetch(key: String) -> String {
        trace(format!("call {key}"));
        let mut call = Box::pin(host_fetch(key.clone()));
        // If the host's result is ready on the first poll, this export returns before it blocks,
        // so the guest sees RETURNED.
        if let Poll::Ready(value) = std::future::poll_fn(|cx| Poll::Ready(call.as_mut().poll(cx))).await {
            trace(format!("returned {key} {value}"));
            return value;
        }
        trace(format!("started {key}"));
        // The executor takes over the pending call: wit-bindgen moves its waitable to the
        // executor's task when the executor polls it.
        let (tx, rx) = futures_channel::oneshot::channel();
        spawn(async move {
            let value = call.await;
            trace(format!("deliver {} {key} {value}", wakeup()));
            let _ = tx.send(value);
        });
        rx.await.unwrap()
    }
}

impl exports::spike::race::executor::Guest for Proxy {
    async fn run() {
        run_executor().await
    }
    fn stop() {
        stop_executor()
    }
}

export!(Proxy);
