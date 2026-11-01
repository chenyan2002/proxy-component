// The executor of the imports component, which writes and reads the virtualized host's end of
// streams and futures, e.g. writing the body stream that a mock of the sync `consume-body`
// returns, or reading the body a mock of the sync `response.new` receives.
//
// Streams and futures themselves are not virtualized: `stream.new`, `stream.read` and the like
// are canonical built-ins, which the runtime implements, not imports. What the virtualized host
// plays is the other end. Only readable ends cross components; the writable end stays with the
// component that created the pair. So when a mock returns a stream, e.g. `consume-body`, it
// creates the pair, and must write to it after it has returned. Linking to the real host's
// streams would mean not virtualizing that interface, and in fuzz and replay modes there is no
// host. Playing the other end lets the host choose what it writes, and how it reads, e.g.
// stopping partway; replay the recorded chunks; observe what the guest writes; and put mocked
// resources in the values, e.g. the `fields` of a `future<result<option<trailers>, error-code>>`.
//
// A sync function can't block or spawn a task, so it queues the work with `spawn` instead, and
// `run_executor` runs it in an async task, which lives for the whole run: the
// `proxy:conversion/executor.run` export, which the `start` function of the exports component
// calls. The exports component needs no executor: its work is only made by `start`, which is
// async, so its `spawn` spawns the work on the task of `start` (see `generate_start`).
type Job = std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>;
#[derive(Default)]
struct Executor {
    queue: Vec<Job>,
    stopped: bool,
    waker: Option<std::task::Waker>,
}
thread_local! {
    static EXECUTOR: std::cell::RefCell<Executor> = std::cell::RefCell::new(Executor::default());
}
/// Queues `job`, from any function, sync or async.
#[allow(dead_code)]
fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
    EXECUTOR.with_borrow_mut(|e| {
        e.queue.push(Box::pin(job));
        // Wakes `run_executor` from another task, with wit-bindgen's `inter-task-wakeup`.
        if let Some(waker) = e.waker.take() {
            waker.wake();
        }
    });
}
/// Polls the queued jobs until `stop_executor` is called and they are all done.
#[allow(dead_code)]
async fn run_executor() {
    let mut running: Vec<Job> = Vec::new();
    std::future::poll_fn(|cx| {
        running.retain_mut(|job| job.as_mut().poll(cx).is_pending());
        // A job may spawn another, so `EXECUTOR` isn't borrowed while jobs run.
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
                std::task::Poll::Ready(())
            } else {
                e.waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        })
    })
    .await
}
#[allow(dead_code)]
fn stop_executor() {
    EXECUTOR.with_borrow_mut(|e| {
        e.stopped = true;
        if let Some(waker) = e.waker.take() {
            waker.wake();
        }
    });
}
