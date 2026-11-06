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
//
// Steps: each run of the executor, i.e. each poll of `run_executor`, is a step, and so is each
// arrival of an export call, which the exports component counts with the `step` export. Host
// lines delivered outside a call carry the step, `at`, in which they happened (see
// `docs/p3-record-replay.md`, Steps). A job that waits for a later run, e.g. an async import
// the fuzzer made STARTED, makes the executor wake itself, as a host completion would.
type Job = std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>;
#[derive(Default)]
struct Executor {
    queue: Vec<Job>,
    stopped: bool,
    waker: Option<std::task::Waker>,
}
thread_local! {
    static EXECUTOR: std::cell::RefCell<Executor> = std::cell::RefCell::new(Executor::default());
    // The last step, counted from 1.
    static STEP: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    // Set by a job waiting for a later run, so that the executor wakes itself.
    static WAKE_AGAIN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    // The waits that ended in a run, whose tasks haven't resumed yet, which hold the executor (see
    // `wait_for_run`), and whether it was woken meanwhile.
    static HELD: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static WOKEN_WHILE_HELD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// Counts a step, and returns it.
#[allow(dead_code)]
fn next_step() -> u32 {
    STEP.set(STEP.get() + 1);
    STEP.get()
}
/// The step a job runs in.
#[allow(dead_code)]
fn current_step() -> u32 {
    STEP.get()
}
/// Waits, from any task, until the executor has run `runs` more times, and returns the step of
/// the last run.
#[allow(dead_code)]
async fn wait_for_runs(runs: u32) -> u32 {
    let mut left = runs;
    wait_for_run(move || {
        if left == 0 {
            return true;
        }
        left -= 1;
        false
    })
    .await
}
/// Waits, from any task, until a run of the executor in `step` or later, or returns at once if
/// the count is already there, e.g. replay's for a host line recorded in that step, which checks
/// that it came in that step.
#[allow(dead_code)]
async fn wait_until_step(step: u32) {
    if current_step() >= step {
        return;
    }
    wait_for_run(move || current_step() >= step).await;
}
/// Waits, from any task, as `wait_until_step` does, and takes the next step, of a host line
/// `start` makes, e.g. an export call's arrival, which it returns: the one after the run it
/// waited for, as the executor doesn't run in between (see `wait_for_run`).
#[allow(dead_code)]
async fn step_after(step: u32) -> u32 {
    wait_until_step(step).await;
    next_step()
}
/// Waits, in a job of the executor, until its run in `step`, waking the executor itself until
/// then, e.g. replay's job for a copy recorded in that step.
///
/// Panics if the count passed `step`: replay diverged from the recording.
#[allow(dead_code)]
async fn reach_step(step: u32) {
    std::future::poll_fn(|_| {
        let now = current_step();
        assert!(now <= step, "replay reached step {now} before the run in step {step}");
        if now == step {
            std::task::Poll::Ready(())
        } else {
            WAKE_AGAIN.set(true);
            std::task::Poll::Pending
        }
    })
    .await
}
/// Waits until a run of the executor in which `done` returns true, and returns its step. A job
/// asks in each run, waking the executor itself in between, as no host event would.
async fn wait_for_run(mut done: impl FnMut() -> bool + 'static) -> u32 {
    let job = std::future::poll_fn(move |_| {
        if done() {
            return std::task::Poll::Ready(());
        }
        WAKE_AGAIN.set(true);
        std::task::Poll::Pending
    });
    wait_for_job(job).await.1
}
/// Runs `job` in the executor, from any task, and returns its value, and the step of the run it
/// completed in, e.g. record mode's host call that blocked, whose completion wakes the executor.
///
/// The executor doesn't run again until the waiting task resumes, e.g. for a guest's event, so
/// that what the task does next, e.g. return a STARTED call, or take an export call's step,
/// follows the run it waited for, in record and replay alike: replay knows only the step.
///
/// `job` may borrow from the waiting task, e.g. a host call on the resource the guest passed: it
/// is only polled while the waiting task waits, and dropped if the task is dropped first, e.g.
/// the guest cancelled its call, which cancels the host's call too.
#[allow(dead_code)]
async fn wait_for_job<'a, T: 'static>(job: impl std::future::Future<Output = T> + 'a) -> (T, u32) {
    type Job<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'a>>;
    struct Done<T> {
        job: Option<Job<'static, T>>,
        value: Option<(T, u32)>,
        waker: Option<std::task::Waker>,
        held: bool,
    }
    /// The waiting task's side, which drops the job if it is still pending, and releases the
    /// executor once the task resumes, or is dropped first.
    struct Waiter<T>(std::rc::Rc<std::cell::RefCell<Done<T>>>);
    impl<T> Drop for Waiter<T> {
        fn drop(&mut self) {
            let mut done = self.0.borrow_mut();
            drop(done.job.take());
            if std::mem::take(&mut done.held) {
                HELD.set(HELD.get() - 1);
                if HELD.get() == 0 && WOKEN_WHILE_HELD.replace(false) {
                    wake_executor();
                }
            }
        }
    }
    let job: Job<'a, T> = Box::pin(job);
    // SAFETY: the job outlives `'a` only as an empty `Option`: the waiter, which `'a` outlives,
    // drops it when it is dropped, and it is only polled before.
    let job: Job<'static, T> = unsafe { std::mem::transmute(job) };
    let done = Done { job: Some(job), value: None, waker: None, held: false };
    let waiter = Waiter(std::rc::Rc::new(std::cell::RefCell::new(done)));
    let job_shared = waiter.0.clone();
    spawn(std::future::poll_fn(move |cx| {
        let mut shared = job_shared.borrow_mut();
        let Some(job) = shared.job.as_mut() else {
            return std::task::Poll::Ready(());
        };
        let std::task::Poll::Ready(value) = job.as_mut().poll(cx) else {
            return std::task::Poll::Pending;
        };
        shared.job = None;
        HELD.set(HELD.get() + 1);
        shared.value = Some((value, current_step()));
        shared.held = true;
        if let Some(waker) = shared.waker.take() {
            waker.wake();
        }
        std::task::Poll::Ready(())
    }));
    let value = std::future::poll_fn(|cx| {
        let mut shared = waiter.0.borrow_mut();
        match shared.value.take() {
            Some(value) => std::task::Poll::Ready(value),
            None => {
                shared.waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }
    })
    .await;
    drop(waiter);
    value
}
/// Queues `job`, from any function, sync or async.
#[allow(dead_code)]
fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
    EXECUTOR.with_borrow_mut(|e| {
        e.queue.push(Box::pin(job));
    });
    wake_executor();
}
/// Wakes `run_executor` from another task, with wit-bindgen's `inter-task-wakeup`.
fn wake_executor() {
    let waker = EXECUTOR.with_borrow_mut(|e| e.waker.take());
    if let Some(waker) = waker {
        waker.wake();
    }
}
/// Polls the queued jobs until `stop_executor` is called and they are all done.
#[allow(dead_code)]
async fn run_executor() {
    let mut running: Vec<Job> = Vec::new();
    std::future::poll_fn(|cx| {
        // Held, it isn't a run: the jobs are polled at the next, as every run polls them all.
        if HELD.get() > 0 {
            WOKEN_WHILE_HELD.set(true);
            EXECUTOR.with_borrow_mut(|e| e.waker = Some(cx.waker().clone()));
            return std::task::Poll::Pending;
        }
        next_step();
        WAKE_AGAIN.set(false);
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
                if WAKE_AGAIN.get() {
                    // Yields, and comes back as the next run.
                    cx.waker().wake_by_ref();
                }
                std::task::Poll::Pending
            }
        })
    })
    .await
}
#[allow(dead_code)]
fn stop_executor() {
    EXECUTOR.with_borrow_mut(|e| e.stopped = true);
    wake_executor();
}
