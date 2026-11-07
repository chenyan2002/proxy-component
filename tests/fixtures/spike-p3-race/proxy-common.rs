// What the record and replay proxies share: the executor of `assets/executor.rs`, which counts its
// wakeups, the clock that positions host events, and the trace.
//
// Trace lines, printed to stdout with a `trace: ` prefix:
//   call <key>                 the guest called `fetch(key)`
//   returned <key> <value>     the host returned at once, so the guest saw RETURNED
//   started <key>              the host blocked, so the guest got a subtask
//   deliver <n> <key> <value>  the host's result reached the executor at its wakeup `n`
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Waker};

type Job = Pin<Box<dyn Future<Output = ()>>>;
#[derive(Default)]
struct Executor {
    queue: Vec<Job>,
    stopped: bool,
    waker: Option<Waker>,
}
thread_local! {
    static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::default());
    // The number of times the executor has woken up.
    static WAKEUP: Cell<u32> = const { Cell::new(0) };
    // Set by a job waiting for a later wakeup, in replay, so that the executor wakes itself, as a
    // host completion woke it in the recording.
    static WAKE_AGAIN: Cell<bool> = const { Cell::new(false) };
}

thread_local! {
    // The trace, printed by `stop_executor` at the end: printing is a sync call to the host, which
    // can suspend the instance while it is entered, and so change the schedule.
    static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}
pub fn trace(line: String) {
    TRACE.with_borrow_mut(|trace| trace.push(line));
}

pub fn wakeup() -> u32 {
    WAKEUP.get()
}

#[allow(dead_code)]
pub fn wake_again() {
    WAKE_AGAIN.set(true);
}

pub fn spawn(job: impl Future<Output = ()> + 'static) {
    EXECUTOR.with_borrow_mut(|e| {
        e.queue.push(Box::pin(job));
        if let Some(waker) = e.waker.take() {
            waker.wake();
        }
    });
}

pub async fn run_executor() {
    let mut running: Vec<Job> = Vec::new();
    std::future::poll_fn(|cx| {
        WAKEUP.set(WAKEUP.get() + 1);
        WAKE_AGAIN.set(false);
        running.retain_mut(|job| job.as_mut().poll(cx).is_pending());
        loop {
            let mut queued = EXECUTOR.with_borrow_mut(|e| std::mem::take(&mut e.queue));
            if queued.is_empty() {
                break;
            }
            queued.retain_mut(|job| job.as_mut().poll(cx).is_pending());
            running.append(&mut queued);
        }
        EXECUTOR.with_borrow_mut(|e| {
            if e.stopped && running.is_empty() {
                Poll::Ready(())
            } else {
                e.waker = Some(cx.waker().clone());
                if WAKE_AGAIN.get() {
                    // Yields, and comes back as the next wakeup.
                    cx.waker().wake_by_ref();
                }
                Poll::Pending
            }
        })
    })
    .await
}

pub fn stop_executor() {
    for line in TRACE.with_borrow_mut(std::mem::take) {
        println!("trace: {line}");
    }
    EXECUTOR.with_borrow_mut(|e| {
        e.stopped = true;
        if let Some(waker) = e.waker.take() {
            waker.wake();
        }
    });
}
