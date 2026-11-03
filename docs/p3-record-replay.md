# Record and replay of WASI p3 components

proxy-component records and replays p2 components, and fuzzes and mocks (dialog mode) p3
components. This file is the design for recording and replaying p3 components: the trace format,
how the proxy records a run without changing it, how replay reproduces it, and what makes replay
portable across runtimes. Nothing here is implemented yet, except a spike that tests the central
claim; see [Spike results](#spike-results) and [Phases](#phases).

It builds on the cross-tool design in component-ui's
[`docs/p3.md`](https://github.com/chenyan2002/component-ui/blob/main/docs/p3.md), and was written
against wasmtime 49, wit-bindgen 0.63 (the `fix-async` fork) and proxy-component after fuzz and
dialog support for p3.

## What exists

- **Fuzz and dialog modes support p3.** The generated imports component mocks every import of the
  main component, from its WIT, including async functions, streams, futures and error-contexts.
  The generated `start` function is async and awaits async exports.
- **`proxy:conversion/executor`** ([`assets/executor.rs`](../assets/executor.rs)) runs the
  virtualized host's end of streams and futures in the imports component. A sync mock, e.g. of
  `consume-body`, can't block or spawn a task, so it queues the work, and the executor runs it in
  the task of its `run` export, which `start` calls. The exports component's work is made by
  `start` itself, so it spawns on the task of `start`. Fuzz mode waits for both at the end of
  `start`; dialog mode drains them after each export call.
- **Labels.** Resources, streams, futures and error-contexts are labeled by handles in the
  generated components' own tables, as [Labels](#labels) describes: a mock by the handle wasmtime
  gives it, through an override of wit-bindgen's `_resource_new`, and a stream or future by the
  end the proxy keeps. A stream or future the proxy creates is introduced with a
  `new (writer: #stream-7, reader: #stream-5)` line, which goes away once fuzz and dialog label
  streams as record does; see [Labels](#labels).

Record and replay modes reject nothing up front, but they don't handle streams, futures or async
functions yet.

## Where nondeterminism comes from

Component model concurrency is cooperative: within one component instance one thread runs at a
time, and it observes other threads, or the host, only at defined points. Two kinds of choices
decide a run:

- **The host's timing**: when an async import returns, whether it returns at once (RETURNED) or
  starts a subtask (STARTED), when and how much the host reads from or writes to a stream or
  future, and when the host calls an export.
- **The runtime's choices** at the points `Concurrency.md` (§Nondeterminism) lists: which pending
  event `waitable-set.wait` or `poll` returns, which ready thread resumes, whether a wait with an
  event already pending blocks, the order of tasks released from backpressure, and which thread
  receives a cancellation.

Calls between components, and streams and futures with wasm at both ends, add no choices of
their own. wasmtime 49 makes the runtime's choices without randomness: the lowest waitable first,
fixed run queues. jco makes them differently (join order, the oldest ready thread); see the table
in component-ui's `docs/p3.md`.

## Trace format

Backward compatibility with p2 traces is not a goal; the format is redesigned. A trace is the list
of lines the proxy observed, in the order it observed them, defined by the Rust types below, in
`crates/trace`. Replay relies on only part of that order; see Order, below. How a trace is stored is
up to the implementation.

```rust
/// The lines the proxy observed, in the order it observed them.
pub struct Trace(pub Vec<Line>);

/// One event, who did it, and, for a host line the guest observes outside its calls, the step in
/// which it happened; see [Steps](#steps).
pub struct Line {
    pub by: Party,
    pub at: Option<Step>,
    pub event: Event,
}

pub enum Party {
    Host,
    Guest,
}

/// A run of the imports component's executor, or an export call's arrival, counted from 1; see
/// [Steps](#steps).
pub struct Step(pub u32);

/// A call, numbered in order by the generated component that sees it, its `table`: the exports
/// component numbers the host's calls of exports, written `t1, t2, …`, and the imports component
/// the guest's calls of imports, written `c1, c2, …`. Sync calls are numbered too.
pub struct TaskId {
    pub table: Table,
    pub id: u32,
}

/// A handle: the table its number is in, and its name as WAVE writes it, its type and its number,
/// e.g. `#stream-7` or `#fields-3`; see [Labels](#labels).
pub struct Label {
    pub table: Table,
    pub name: String,
}

/// A generated component, which numbers handles and tasks: the imports component, whose handle
/// table holds the ends that cross at the main component's imports, and which numbers the guest's
/// calls of imports, and the exports component, for its exports.
pub enum Table {
    Imports,
    Exports,
}

/// A value in WAVE, e.g. `ok(#response-3)`, whose type comes from where it appears: a function's
/// parameter or result, or a stream's or future's payload.
pub struct Wave(pub String);

pub enum Event {
    /// `by` is the caller: the host for an export, the guest for an import.
    Call { id: TaskId, is_async: bool, func: String, args: Vec<Wave> },
    /// The callee blocked before returning, so the caller got a subtask. Only for an async call.
    Started { id: TaskId },
    /// The callee's result, `None` for a function without one.
    Return { id: TaskId, value: Option<Wave> },
    /// The caller asked to cancel a pending call. The callee answers with `Cancelled`, or with a
    /// `Return`, if it returns anyway.
    Cancel { id: TaskId },
    /// The callee acknowledged a cancellation instead of returning.
    Cancelled { id: TaskId },
    /// A copy: the host wrote `items`, which the guest read.
    Write { stream: Label, items: Wave },
    /// A copy: the host read `items`, which the guest wrote.
    Read { stream: Label, items: Wave },
    /// A future's value was taken by its reader; `by` is the writer.
    Value { future: Label, value: Wave },
    /// A stream or future end, or a resource, was dropped; `by` is who dropped it.
    Drop { handle: Label },
}
```

What the types don't enforce:

- **Who acts.** `by` is the caller for `Call` and `Cancel`, the callee for `Started`, `Return`
  and `Cancelled`, the host for `Write` and `Read`, the writer for `Value`, and whoever
  dropped it for `Drop`. Most of it follows from the event; it is explicit, for readability.
- **Task ids.** A `TaskId` comes from a counter in the generated component that sees the call,
  not from the runtime. wasmtime numbers its tasks internally, but no canonical built-in gives a
  task's id to wasm, and other runtimes number tasks their own way. Each guest import call arrives
  at the imports component as a new task, which nothing identifies but task-local storage
  (`context.get`, `context.set`), which the callee fills itself. The subtask handles of the
  imports component's own calls to the host are reused, and replay makes no host calls. Replay
  numbers calls the same way, and they come in the same order, as replay checks each guest call.
- **Answers.** A sync call is answered by a `Return`; an async call by a `Return`, `Started` then
  a `Return`, or `Started` then `Cancelled`, possibly with a `Cancel` in between. Other lines may
  come between a call and its answer, e.g. host events of the other component, so the id pairs
  them. A p2 component's calls are all sync, and each `Return` comes right after its `Call`.
- **Cancellation.** `Cancel` is a request: the caller's `subtask.cancel`, which the callee gets
  as a cancellation request. The callee may acknowledge it, with `task.cancel`, which is
  `Cancelled`, or return anyway, which is a `Return`, also when its result was already on its
  way. A call that hasn't started yet is cancelled without the callee seeing it. It goes both
  ways: the guest can cancel a pending import, and the host a pending export call.
- **Cancel and Drop** are about different things: `Cancel` about a pending call, `Drop` about a
  handle.
  - **`Cancel`.** In wit-bindgen, an async import call is a Rust `Future`; dropping it while the
    call is pending cancels the call, with `[subtask-cancel]`: a `Cancel` of the task.
  - **`Drop`** ends either end of a stream or future, or a resource. Dropping a writer ends the
    stream, which the reader sees as its read returning `dropped`; dropping a reader, the writer
    sees as its next write returning `dropped`. The proxy sees each side's drop that way, from its
    own end, and records it as a `Drop` by that side.
  - **A pending copy can be cancelled too**, with `stream.cancel-read` or `cancel-write`, which
    doesn't end the stream. It isn't recorded: a cancelled copy never completes, so there is no
    copy line, and the other end doesn't see it.
- **Labels** are handles in the two generated components' tables, and tasks are numbered by the
  two components' counters, so the `table` of a label or a `TaskId` is part of what it names. A
  handle's number is reused only after its handle has ended, which an earlier line records. A
  label inside a `Wave` is written by its name alone; see [Labels](#labels) for its table.
- **Positions.** `at` is the step of a host line the guest observes outside its calls, including
  an export call's arrival; other lines have none. See [Steps](#steps).
- **Copies.** A copy completes once, when both sides have acted, so it is one line. Copies are
  lazy: if the guest never reads a stream the host writes, no `Write` appears. For `stream<T>` a
  line is the chunk one rendezvous copied, so replay can give the guest's reads the same chunks.
  This holds for pumps that don't read ahead; [Read-ahead](#read-ahead-in-pumps) would change it.
- **Order.** The trace is not a total order. Each line belongs to a task, by its `TaskId`, or to a
  handle, by its `Label`, and the lines of different tasks or handles may come in any order. What
  orders them is each task's own sequence, `Call`, `Started`, `Return`; the `at` of the host lines
  the guest observes outside its calls; and the ids, which are numbered in the order calls arrive.
- **No guest task on import calls.** The component model passes no caller identity, so the proxy
  can't tell which of the guest's own tasks made an import call: `c5` can't be linked to the
  export call `t1` that caused it. That limits deriving other orders from a trace:
  - **What the trace does order:** a `Return` comes after its `Call` and `Started`, a copy after
    its stream's first appearance, a `Cancelled` after its `Cancel`.
  - **What it doesn't:** which guest lines a host line caused. If the guest's `Call c3` comes after
    the host's `Return c2`, it may be the same guest task, reacting to `c2`, or an unrelated one.
    Only in the second case may `Return c2` move after `Call c3`.
  - **So** a reordered trace is checked by replaying it. If the guest diverges, the order wasn't
    consistent, or the guest behaves differently in it; either way, the trace says nothing after
    that point, which p3.md calls holes. Exact dependencies need caller identity, which the
    [core-level shim](#portability) could provide, from the guest's task-local `context.get`.

An example, for `GET /wait?ms=250` on [`examples/wasi-http-p3`](../examples/wasi-http-p3), writing a
`TaskId` as `t1` or `c1`, and a `Label` by its name; every label here is in the imports table:

| `by` | `at` | `event` |
|---|---|---|
| host | 1 | `Call { id: t1, is_async: true, func: "wasi:http/handler@0.3.2.handle", args: ["#request-1"] }` |
| guest | | `Call { id: c1, is_async: false, func: "wasi:http/types/request@0.3.2.get-method", args: ["#borrow-request-1"] }` |
| host | | `Return { id: c1, value: Some("get") }` |
| guest | | `Call { id: c2, is_async: true, func: "wasi:clocks/monotonic-clock@0.3.2.wait-for", args: ["250000000"] }` |
| host | | `Started { id: c2 }` |
| host | 4 | `Return { id: c2, value: None }` |
| guest | | `Call { id: c3, is_async: false, func: "wasi:http/types/response@0.3.2.new", args: ["#fields-2", "some(#stream-5)", "#future-6"] }` |
| host | | `Return { id: c3, value: Some("(#response-3, #future-7)") }` |
| guest | | `Return { id: t1, value: Some("ok(#response-3)") }` |
| host | 6 | `Read { stream: #stream-5, items: "[97, 115, 107, …]" }` |
| guest | | `Drop { handle: #stream-5 }` |
| guest | | `Drop { handle: #future-7 }` |

The lines about the response body, `#stream-5`, which the guest passed to `response.new`, come
after the export's `Return`: the host's `Read` of the body, and the guest's `Drop` of it, which ends
it. The guest returns the response before it writes the body, in a spawned task. The last line is
the guest dropping `#future-7`, which `response.new` returned.

## Labels

A label names a handle by its table and its name, as WAVE writes it: its type and the handle's
number in that table, e.g. `#fields-3` or `#stream-7`.

- **Resources** are in the imports table, which holds them for both components. In record mode,
  a host resource is labeled by the handle the imports component keeps for it, which the exports
  component gets through `get-handle-*`; in fuzz and dialog modes, a mock by its handle, which
  `_resource_new` gives it; in replay, a mock carries the trace's label.
- **Streams and futures** are labeled by the handle of the end the proxy received, which it keeps
  for the stream's lifetime: the host's reader in an import's result or an export's arguments, the
  guest's in an import's arguments or an export's result. The proxy pumps it through a pair of its
  own, whose ends never appear in the trace. A stream or future first appears in the value that
  brings it across, in the table of the component that call or result crosses, and every later
  line uses the same label:

  | `by` | `at` | `event` |
  |---|---|---|
  | guest | | `Call { id: c4, is_async: false, func: "test:p3/host/body@0.1.0.consume", args: ["#body-1"] }` |
  | host | | `Return { id: c4, value: Some("(#stream-6, #future-9)") }` |
  | host | 5 | `Write { stream: #stream-6, items: "[52]" }` |

  - **Record mode** prints each value as it received it, before `to_proxy`. Today it prints
    arguments after converting them, which has to change.
  - **Fuzz and dialog** label as record does. A mock plays the host: it creates the host's pair
    and keeps the writer, and the proxy keeps the reader, labels the stream by it, and pumps it to
    the guest through a second pair. Today they hand that reader straight to the guest, which frees
    its slot, so they label by the writer, and bind the two with a `new (writer: …, reader: …)`
    line, which this format doesn't have.
  - **Replay** creates a pair for each stream the host hands out, hands out its reader, and
    performs the trace's lines on the stream's label through its writer.
  - **The guest's `stream.new` and `future.new`** are built-ins inside its component that the
    proxy doesn't see, so a stream the guest creates first appears where it crosses. Replay doesn't
    need its creation, as the guest creates it again itself.
- **Two tables, neither the main component's.** The imports component keeps the ends that cross
  at the main component's imports, the exports component those of its exports, and the two can
  hold the same number. So a label carries its `table`. Neither table matches the main
  component's handle ids, even in p2, as the main component's table also holds handles the proxy
  never sees: its own resources and streams, waitable sets and subtasks. Labels only need to be
  consistent: replay treats them as names, and a mock carries the trace's.

Limits:

- **An end that comes back, or passes through the guest, gets a new label.** When the guest hands
  back a stream the proxy gave it, e.g. an `echo` that returns its argument, or passes a stream
  from an export on to an import, the proxy gets a new handle, and nothing in the component model
  tells it that this is the stream it already labeled. The trace stays consistent, and replays: a
  reader of it just doesn't see the two labels as one stream. The [core-level shim](#portability)
  could carry the label through, as the end keeps one handle in the guest's table from when it
  arrives until the guest passes it on.
- **Replay can't check labels.** Replay's tables differ from the recording's, with mocks instead
  of the host's ends and the pumps, so it can't recompute the labels, only use the trace's.
- `list<u8>` items are WAVE lists for now; p3.md's byte strings (`b"…"`) would be shorter.

The alternative is to label handles by a counter, in the order they cross between the proxy and the
main component, shared by the two generated components through an interface of the imports
component. A number is then never reused, and the same in record and replay, so replay can check
each label, and there is no `table`. A resource can keep its label in its representation, but a
stream, future or error-context is a wit-bindgen type, which wasm-wave's `ToValue` prints by its
handle, so the proxy has to map each handle to its label, and set the lookup with wasm-wave's
`set_handle_label`. It has the same limit for ends that come back.

## Recording

Recording runs the composition in the real host, as for p2: the imports component forwards to
the host and records, and the exports component records the calls the host makes into the main
component.

### The executor delivers and records host events

In record mode, the imports component routes every interaction with the host through one
executor task, like fuzz mode's:

- **Async imports.** The executor's job awaits the host's subtask, and completes the proxy's
  export call with the result (its `task.return`).
- **Streams and futures, by pumps.** The proxy doesn't pass a stream end through, as the copies
  would then go between the host and the guest without it. It keeps the end it received, creates
  a pair of its own, and passes on that pair's other end. A pump is the executor job between the
  two: it reads from one end, writes what it read to the other, and records each copy, and passes
  on a drop the same way. A future is pumped once, for its value.

So nothing the host does reaches the guest outside an executor run. That's why a step number can
mark when each host event happened.

Putting the executor between the host and the guest must not change the run it records, which
these rules keep it from doing:

- **Delivery on arrival.** The executor delivers each host result as soon as it arrives, without
  holding or yielding, so that several results can be pending for the guest at once, as without
  the proxy.
- **RETURNED is preserved.** When the host's result is ready on the first poll, the proxy's
  export returns before it blocks, so the guest sees RETURNED; otherwise STARTED. Replay
  reproduces it without a host: the mock of a call recorded RETURNED returns before it awaits, and
  one recorded STARTED awaits its step first. This needs the host to be the runtime's own
  functions, as it is under `wasmtime serve` or `run`; see [Spike results](#spike-results) for a
  host that is a component.
- **No blocking WASI 0.2 calls during a run.** A sync WASI 0.2 function such as
  `blocking-write-and-flush`, which `println!` calls, can suspend the instance in the middle of the
  call, while it stays entered: other tasks run, and calls into the instance wait, which changes
  the schedule, and which the proxy can't record. The proxy, and the bundled recorder, which
  today prints each line as it is recorded, must buffer the trace, write it through a WASI 0.3
  `stream<u8>`, whose copies are async, or leave it to a host recorder.

### Steps

The imports component's executor is a single task that runs the proxy's jobs: the host's side of
its pending async imports, streams and futures (see [`assets/executor.rs`](../assets/executor.rs)).
It runs whenever the runtime polls its task: when it is started, and each time it is woken, by
new work being queued or by a host completion one of its jobs waits for. Each run is a **step**,
and so is each arrival of an export call, below. Steps are counted from 1, by one counter in the
imports component, and in each run the executor polls all its jobs.

Replay has to know when to perform each host line. There are three kinds of lines:

1. **Guest lines**, `by: guest`: the guest's calls, its export results, its drops. Replay doesn't
   perform them: they happen when the guest does them, and replay checks each against the trace.
   They have no `at`.
2. **Host lines that answer a guest call before it returns**: the `Return` of a sync call, or of a
   RETURNED async one, and `Started`. The proxy did them while handling the call, before the call
   returned to the guest, and replay does too: when the guest makes the call, replay performs them
   at once, as part of answering it. The guest's `Call` line says when they happen, so they have no
   `at`.
3. **Host lines the guest observes outside its calls**: what the host does on its own schedule,
   rather than in answering a call. Mostly, the host returns something to the guest: the `Return` of
   a STARTED call, the items of a `Write`, a future's `Value`. Also the host taking what the guest
   wrote, a `Read`, which completes the guest's write; the host dropping its end, a `Drop`, which
   the guest sees as `dropped`; a `Cancelled`; and the host calling an export, below. The executor
   performs them when the host gets to them, so `at: n` says they happened in step `n`. In replay,
   the executor counts its steps the same way, and a job holding such a line waits until the count
   reaches `n`. There is no host to wake the executor, so a job waiting for a later step makes the
   executor wake itself, by yielding, and its next run is the next step.

**Export calls** arrive at the exports component, from the host, so their arrival is a step of its
own. On each arrival, the exports component calls `step` on `proxy:conversion/executor`, a sync
function that increments the counter and returns it, as the export `Call`'s `at`; the executor's
later runs are numbered after it. In replay, `start` makes an export call recorded at step `n` once
the count reaches `n - 1`, by calling `step` and then the export. An arrival can't fall within a
run of the executor: the host only calls an export while no wasm is running, unless something
suspends where the proxy can't see it, which the requirements below rule out.

### Why the step is enough, and a gate isn't needed

The trace order alone is not a replayable schedule. Say the guest has two `fetch` subtasks in one
waitable set, and is busy:

1. The host finishes `b`, then `a`. The proxy records them in that order, and delivers both.
2. The guest then waits, finds both pending, and wasmtime returns the lowest waitable: `a`.
3. A replay that releases each event as soon as the trace cursor reaches it may release `b` while
   the guest is waiting, so the guest takes `b` first: a different run.

component-ui's `docs/p3.md` fixes this with a gate: hold every host result, and release one only
when the guest is idle. That excludes schedules: a guest never finds two results ready at once,
and every held call turns from RETURNED to STARTED. It also needs to know when the guest is idle,
which the proxy can't see.

The step fixes it without holding anything. Every host event reaches the guest through the
executor, in one of its steps. If the runtime's choices are deterministic, everything that happens
between two steps follows from what happened before, so "host event H was performed in step n" is
a position that means the same in record and replay. Replay performs H in step n,
exactly where it was delivered, and the guest's waits fall in the same places. In the example,
both results are delivered before the guest waits, in record and in replay alike.

From [the spike](#spike-results), with `fetch("b")`'s timer firing before `fetch("a")`'s. The
spike doesn't count its export call as a step:

| `by` | `at` | `event` | |
|---|---|---|---|
| guest | | `Call { id: c1, …, func: "fetch", args: ["a"] }` | |
| host | | `Started { id: c1 }` | |
| guest | | `Call { id: c2, …, func: "fetch", args: ["b"] }` | |
| host | | `Started { id: c2 }` | steps 1 and 2: the executor's first run, and its run for the queued calls |
| host | 3 | `Return { id: c2, value: Some("B") }` | step 3: `b`'s completion woke the executor |
| host | 4 | `Return { id: c1, value: Some("A") }` | step 4: `a`'s completion woke it again |

The guest returns `"BA"`. In replay there is no host: the mock of each call waits until the
executor's count reaches its `at`, and the executor, with a job waiting, yields, and comes back as
its next step. So `b`'s result is released in step 3 and `a`'s in step 4, and the guest's waits
receive them in the same order, with the same results in each. Released only by their order in the
trace, the two results could reach the guest before or after a different one of its waits.

[The spike](#spike-results) confirmed this in wasmtime. It requires:

- **Deterministic runtime choices**, see [Portability](#portability).
- **The same executor in record and replay**, with only the leaves different: host calls in
  record, futures that the trace makes ready in replay.
- **Steps in replay where the host woke the executor in the recording.** Replay has no host, so a
  job that waits for a later step makes the executor yield, and come back as the next step.
  wasmtime runs the ready tasks first, as it did before the host's completion in the recording.
- **No suspension the proxy can't see**: no calls to other components where the recording calls
  the host, and no blocking WASI 0.2 calls, as above.
- **One clock for both components.** The host calls exports through the exports component, not
  the executor, so each arrival takes a step from the same counter, through `step` on
  `proxy:conversion/executor`, as [Steps](#steps) describes.

### What recording can't reach

The proxy puts a hop, its own task, between the host and the guest. With delivery on arrival:

| | |
|---|---|
| RETURNED | preserved, when the proxy forwards to host functions rather than another component |
| several host results pending at one guest wait | possible, as nothing holds them back. A wit-bindgen guest still takes them one per wakeup, the lowest waitable first, with or without the proxy: its callback receives one event at a time, and its futures don't see the others until their own events |
| a new export call | preserved: forwarded in the same task |
| a guest read or write that completes at once, as the host had data or was reading | only if the pump [reads ahead](#read-ahead-in-pumps); otherwise it blocks first |
| a host result arriving before the guest's next wait | may arrive after it instead, as the executor needs a turn to pass it on: a slower host, which is legal. [Read-ahead](#read-ahead-in-pumps) doesn't help: an async import's result has nothing to read ahead, and a stream's data that arrives just before the wait still waits for the pump. A wit-bindgen guest, whose callback gets one event per wakeup, can't tell the difference when this is the only event, but can when another is pending at the same wait: without the proxy, the lowest waitable of the two comes first, with it, the other one. A guest that calls `waitable-set.poll` also finds nothing pending |

Whatever schedule happens is recorded completely. Not every timing the unproxied component could
have is reachable while recording, as the proxy only makes the host look slower, or, with
read-ahead, eagerer. Covering other schedules is exploration, which fuzz mode does.

Only removing the hop would close the gap of a host result arriving late: the proxy would pass the
host's subtasks and stream ends straight through, and see nothing of them, so recording would have
to observe from inside the guest, as the [core-level shim](#portability) does, which sees what each
of the guest's waits returns. Replay would still mock the imports in the proxy.

### Read-ahead in pumps

Whether a pump reads from one side before the other side asks is a choice of record mode's
pumps, and fuzz mode's, not of replay's. Replay has to make the same choice as the recording, as
it runs the same executor, and the guest sees the difference: whether its copy completes at once
or blocks first.

- **Without read-ahead**, the pump reads from the host only once the guest reads, and reads from
  the guest only once the host reads. Each of the guest's copies blocks first, so a guest copy
  that completes at once is never recorded, nor is whatever the guest does differently after one.
  The two copies of a chunk, between the host and the pump, and between the pump and the guest,
  happen together, so one line describes both.
- **With read-ahead**, the pump keeps a read pending on the host's side, and data ready for the
  guest, or the other way, for a stream the guest writes. Immediate copies become reachable, and
  every copy the guest makes is still recorded. What changes is what the host sees: it is read
  earlier, and possibly more, than the guest asked.
  - Items the pump read ahead, but the guest never took, were written by the host, though no
    guest read took them.
  - If the guest drops its reader after `k` items, the host's next write returns `dropped`
    without the proxy, but succeeds into the pump with it, and the host learns of the drop later.
  - A host whose behavior depends on being read, e.g. a network body consumed earlier, or a
    producer that produces only when read (wasmtime's `StreamProducer::poll_produce`), may then
    behave in a way the unproxied composition wouldn't.

  Replay stays consistent, as it reproduces the recording, not the unproxied run; the recording
  is skewed on the host's side.
- **Read-ahead changes what a copy line records.** The two copies of a chunk now happen at
  different times. For the guest's read to complete at once in replay, replay's pump has to have
  the data in the step record mode's pump got it from the host, so a `Write` is the host's chunk,
  with the step it arrived in, and the guest's reads may split it, which replay derives again. A
  `Read`, likewise, is the host's take from the pump, in the step it happened, which frees the
  pump to take more from the guest; the guest's writes are checked as they happen. Copies are then
  no longer lazy, and a line is no longer one rendezvous of the guest's.

### Hosts

- Recording a wasi-http component under `wasmtime serve -S p3,http`, or viceroy, needs no change.
- Recording under `proxy-component run` passes the main component's p3 imports through to the
  host, so `run` needs `wasmtime_wasi::p3` and wasmtime-wasi-http's p3 linker, and, for
  `wasi:http/handler`, a way to receive requests, as `--invoke` can't build one.

## Spike results

[`tests/spike-p3-race`](../tests/spike-p3-race) tests steps in wasmtime 49, with
hand-written components: a guest whose `run` calls `fetch("a")` and `fetch("b")` concurrently and
joins the results in the order its wakeups receive them; a record proxy whose `fetch` forwards to
wasmtime's `wait-for`, with delays from `DELAY_A` and `DELAY_B` or random, and whose executor
counts its steps; a replay proxy that delivers each result in its recorded step; and a driver
that runs the executor alongside the guest. `make -C tests/spike-p3-race` records each scenario,
replays it, and requires the same result, the same trace, and the same results in each of the
guest's wakeups.

- **The clock works.** All scenarios replay exactly: each result first (`"AB"`, `"BA"`), each call
  RETURNED, both RETURNED, the guest busy until both results are ready, and random delays.
- **RETURNED needs the host to be host functions.** With the host as a component instead
  (`make -C tests/spike-p3-race component-host`), a call the proxy saw RETURNED reached the guest
  as STARTED, and the guest got the two results in separate wakeups, while replay gave them
  together. wasmtime runs the proxy's call into a component as a queued work item, which suspends
  the proxy while it is entered, and holds back the guest's next call into it (`do_not_enter`,
  `concurrent.rs:774`). Calling a
  host component in replay as well made other scenarios diverge, as a component host suspends
  differently in record and replay. Removing it, i.e. forwarding to host functions, fixed it.
- **Sync WASI 0.2 calls change the schedule.** At first the proxy and the guest printed each line
  with `println!`, and several scenarios diverged; buffering the output fixed them.
- **Two results never reached the guest in one wakeup**, through the proxy, nor, in one-off
  experiments outside the spike, through a component awaiting `wait-for` directly, or with the guest
  awaiting two `wait-for`s itself, as a wit-bindgen guest takes one event per wakeup. wasmtime gave
  the lowest waitable first: `a` first, even when `b`'s timer fired first. Only both RETURNED,
  without events, gave both at once.
- **wit-bindgen moves a pending call between tasks.** The proxy's export polls the host call once,
  for RETURNED, and hands it to the executor otherwise; the executor's poll moves its waitable to
  the executor's task (wit-bindgen's `waitable.rs`, "cross-task situations").

## Replay

Replay runs `composed.wasm`, with no host imports, as for p2. The imports component mocks every
import from the trace, the exports component's `start` makes the export calls, and the same
executor runs both.

- **Host lines are performed** in their step `at`, or within the guest's call for those without one:
  a `Return` or `Started` of an import, the chunk of a `Write`, the take of a `Read`, a host `Drop`,
  the next export `Call`. An async mock whose call was RETURNED returns before it awaits, so the
  guest sees RETURNED; a STARTED one returns after.
- **Guest lines are checked** when they happen: an import call with its arguments, an export's
  result, the items of a copy from the guest, a guest drop. A mismatch is a divergence, reported
  with the expected and the actual line.
- **Divergence** can also show up as a stall, when nothing can reach the next line; wasmtime then
  traps with "deadlock detected", and the recorder prints the line it was waiting for.
- **Recorder interface.** `proxy:recorder` gets functions to record and read the new lines.
  Waiting for a step is the executor's, not the recorder's. The bundled recorder
  (`assets/recorder*.wasm`) and the host recorder of `run` (`--use-host-recorder`) implement them.

## Correctness

Two questions, of which only the first can be guaranteed:

1. **Soundness**: every run that recording sees replays exactly.
2. **Coverage**: recording sees every schedule the composition could have. It doesn't; see
   [What recording can't reach](#what-recording-cant-reach). Replay only has to reproduce what was
   recorded, and covering other schedules is exploration.

### Soundness, by induction on steps

A run is a function of its trace if the composition is deterministic apart from the inputs the
trace records: the classic condition for deterministic replay, that every nondeterministic choice
is recorded. Here it is an induction on steps: if record and replay are in the same state when
step `n` begins, and step `n` performs the same host lines, they are in the same state when step
`n + 1` begins. Between steps, the guest's lines are checked, and its calls answered by the lines
without an `at`. The induction holds under four assumptions, none of them proven yet:

- **The executor is the only way in.** Every host event reaches the guest in a run of the
  executor; see [Recording](#the-executor-delivers-and-records-host-events).
- **Between steps, everything is deterministic.** The runtime's choices are fixed, and nothing
  suspends where the proxy can't see it: no host components, no blocking WASI 0.2 calls. The
  component model doesn't guarantee this (`Concurrency.md` allows nondeterminism); wasmtime 49
  happens to, which is why replay in another runtime needs [the profile or the
  shim](#portability).
- **The same executor in record and replay**, whose work in a step depends only on its state and
  the trace's lines for that step.
- **The trace holds every input**: the values, RETURNED or STARTED, and the step of each host
  line.

### Fork-join program

In a fork-join program without determinacy races, every schedule gives the same result (the
result behind Feng and Leiserson's SP-bags), so for such a guest, recording the values would be
enough, without steps. A component model async guest isn't one in general: it can observe the
order in which completions arrive, as the spike's guest does, joining results in the order its
wakeups receive them, and as a `select` does. That is a nondeterministic merge, outside fork-join's
determinacy, and it is what steps record. The theory tells which guests need steps, not that steps
are enough.

For coverage, the theory to use is systematic concurrency testing: stateless model checking with
partial-order reduction (DPOR, as in CHESS), which, for a bounded program, visits every class of
schedules that differ only in the order of independent steps. The trace's partial order fits it,
but deciding which steps are independent needs caller identity on import calls, which only [the
shim](#portability) could provide; see No guest task on import calls, in
[Trace format](#trace-format).

### Checking the implementation

- **Replay checks itself, for each trace.** It checks every guest line, so a wrong replay shows up
  as a divergence or a stall, not silently. It can miss only what the trace doesn't record, such
  as guest-internal streams, or what the guest's waits returned, which the shim could record.
- **A fixpoint test**: record a run, replay it while recording, and require the two traces to be
  equal. The spike does this, for its scenarios.
- **Enumerated scenarios**: for small guests, every placement of host completions across steps,
  rather than random delays; guests that merge nondeterministically, which are the ones steps
  exist for; and schedules that fuzz mode generates, recorded and replayed.
- **The assumptions, checked at run time**: in record mode, a host event that the guest observes
  outside a step; in replay, a step count that differs from the recording's.
- **A model of the spec.** The canonical ABI's reference, `definitions.py`, is executable, and
  marks its nondeterministic choices. Modeling the guest, the proxy and the scheduler there, and
  checking exhaustively, for small programs, that replay at the recorded steps gives the same run,
  would test the design independently of wasmtime: the closest to a proof, short of writing one.

## Portability

**Replay in the runtime that recorded needs no change to anything.** It needs the runtime's
choices to be deterministic given the same inputs, the same runtime version and configuration,
and the same executor in record and replay.

- **wasmtime 49** makes its choices without randomness.
- **jco** replays a composition with no async host imports with its fixed rules; its
  `asyncDeterminism` setting is for this. Recording in jco meets its RETURNED race in
  `_lowerImport`, which only changes what the proxy sees from the host, and the proxy records what
  it saw.

**Replay in another runtime** needs the runtime's choices to be the same in both. Two ways:

1. **A deterministic scheduling profile** in the component model, as core Wasm 3.0 has for NaNs
   and relaxed SIMD: fix each choice `Concurrency.md` lists, e.g. the lowest waitable first, the
   oldest ready thread first, and when a host completion is delivered relative to ready wasm
   tasks. A profile fixes the choices rather than recording them, so the composition stays as it
   is.
   - The process: an issue, then a PR to `WebAssembly/component-model`, updating
     `design/mvp/Concurrency.md` and the reference `design/mvp/canonical-abi/definitions.py`
     with tests, discussed in the WebAssembly CG's component model meetings; runtimes adopt it
     behind a feature flag, as they do async (🚝) and error-context (📝).
   - wasmtime and jco already have deterministic rules, but different ones, so the profile picks
     one at each point.
2. **A shim inside the guest**, without anyone's agreement: rewrite the guest component to put a
   core module between its core instance and its async built-ins (`waitable-set.wait`, `poll`,
   `thread.yield`, the stream and future operations) and callback exports. The built-ins are
   defined inside the guest component, and act on its own tables, so this can't be a separate
   component; the shim reports each observation (the event a wait returned, a copy that blocked)
   through a new import, e.g. `proxy:sched/hook`, composed with the proxy. Replay then releases
   exactly what each recorded wait saw. The shim observes; replacing the built-ins would mean
   reimplementing the component model's async runtime inside the guest.

Patching the runtime, e.g. a scheduling hook in wasmtime, would also work, but replay must not
depend on runtime patches. wasmtime 49 has the start of its own record and replay
(`Config::rr`), without concurrency yet.

## Phases

1. **Spike, in wasmtime**: done, see [Spike results](#spike-results).
2. **Trace format** in `crates/trace`, and the recorder interface.
3. **Replay**, tested on traces that fuzz mode writes in this format, as its executor can count
   steps too, and its mocks can label streams as record does, with the checks of
   [Correctness](#checking-the-implementation).
4. **Record**, with the executor and its pumps, under `wasmtime serve` and `run`.
5. **Portability across runtimes**: the profile proposal, or the shim.

## Open questions

1. **Steps with several arrivals**, and the host's side of stream copies (wasmtime's
   `StreamProducer::poll_produce`, which is called when the guest reads): whether counting
   steps places them exactly.
2. **[Read-ahead in pumps](#read-ahead-in-pumps)**: whether to read ahead, which makes the
   guest's immediate copies reachable, at the cost of the host's view, and of copy lines that
   record the host's chunks rather than the guest's.
3. **`list<u8>`** as WAVE lists or byte strings.
4. **Which way to portability**: the profile, the shim, or both.
5. **Hosts that are components**, e.g. virtualized WASI composed with the main component: whether
   to support recording them, which needs the guest to always see STARTED, or the replay to
   reproduce the component host's suspensions.
6. **Guests that see several events at once**: bindings that drain pending events, or call
   `waitable-set.poll`, unlike wit-bindgen's; the spike covers only wit-bindgen guests.
7. **The recorder's output**: buffering it to the end, a WASI 0.3 stream, or only a host
   recorder.
