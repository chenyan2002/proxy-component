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
//
// How it runs: `spawn` pushes a job and wakes `run_executor`, through wit-bindgen's inter-task
// wakeup, as a sync mock runs in another task. `run_executor` polls the running jobs, then the
// queued ones, until no more are queued, as a job may spawn another. As the jobs are polled in
// the task of `run`, a job blocked on a stream or future joins its waitable set, so the guest's
// side of the copy wakes `run`, which polls them all again: they share one waker. `start` calls
// `stop` once the exports it called are done, and `run` returns once the jobs are too. In dialog
// mode, `start` does so after each export call, and calls `run` again for the next one.
//
// When a value with a stream or future changes hands, e.g. an export returns a stream, the
// generated code doesn't await it. It gives it to `consume` (or `fuzz`, for one the host gives),
// which decides at once what the host does, e.g. in fuzz mode whether it reads the stream to the
// end, reads a few chunks, or drops it, and with what capacity; in dialog mode, the job asks the
// user at each step. A job then does it. When it runs is up to cooperative scheduling: first at
// the next point where its task suspends, e.g. the yields of `start` after each export call, an
// async call, or at the latest when `start` waits for the jobs: at the end in fuzz mode, and
// after each export call in dialog mode, so that the user is done with the streams and futures of
// a call before choosing the next one. Each read or write
// completes once the guest does its side. So the host's reads and writes overlap the following
// export calls, in an order the schedule decides, which is part of what fuzz mode explores.
// Awaiting the value right after the call instead would hide those orders, and deadlock a guest
// that only writes a stream in a later call, or keeps it for the rest of the run.
//
// The flip side: if the guest never finishes, or drops, a stream or future it gave the host, its
// job never ends, so neither do `run` nor `start`, and wasmtime traps with "deadlock detected".
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
