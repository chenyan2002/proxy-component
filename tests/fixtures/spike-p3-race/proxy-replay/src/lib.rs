//! Replays the trace on stdin: checks each guest call against it, and delivers each host result
//! at the executor's wakeup the trace gives. Prints the trace it sees, to compare.
wit_bindgen::generate!({ path: "../wit", world: "proxy-replay" });

#[path = "../../proxy-common.rs"]
mod common;
use common::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::task::Poll;

/// The recorded trace: the lines of guest calls, in order, and each delivery, by key.
struct Expected {
    calls: VecDeque<String>,
    deliveries: HashMap<String, (u32, String)>,
}
thread_local! {
    static EXPECTED: RefCell<Option<Expected>> = const { RefCell::new(None) };
}

fn load() {
    EXPECTED.with_borrow_mut(|expected| {
        if expected.is_some() {
            return;
        }
        let mut calls = VecDeque::new();
        let mut deliveries = HashMap::new();
        for line in std::io::stdin().lines() {
            let line = line.unwrap();
            let Some(line) = line.strip_prefix("trace: ") else {
                continue;
            };
            let words: Vec<&str> = line.split(' ').collect();
            if words[0] == "deliver" {
                let n = words[1].parse().unwrap();
                deliveries.insert(words[2].to_string(), (n, words[3].to_string()));
            } else {
                calls.push_back(line.to_string());
            }
        }
        *expected = Some(Expected { calls, deliveries });
    });
}

fn next_call() -> String {
    EXPECTED.with_borrow_mut(|e| e.as_mut().unwrap().calls.pop_front().expect("the trace ended"))
}

struct Proxy;

impl exports::spike::race::host::Guest for Proxy {
    async fn fetch(key: String) -> String {
        load();
        trace(format!("call {key}"));
        let expected = next_call();
        assert_eq!(expected, format!("call {key}"), "diverged");
        let next = next_call();
        if let Some(value) = next.strip_prefix(&format!("returned {key} ")) {
            trace(format!("returned {key} {value}"));
            return value.to_string();
        }
        assert_eq!(next, format!("started {key}"), "diverged");
        trace(format!("started {key}"));
        let (n, value) = EXPECTED
            .with_borrow_mut(|e| e.as_mut().unwrap().deliveries.remove(&key))
            .expect("no delivery in the trace");
        let (tx, rx) = futures_channel::oneshot::channel();
        spawn(async move {
            std::future::poll_fn(|_| {
                if wakeup() >= n {
                    Poll::Ready(())
                } else {
                    wake_again();
                    Poll::Pending
                }
            })
            .await;
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
