# Record and replay of WASI p3 components

proxy-component records and replays p2 components, fuzzes and mocks (dialog mode) p3 components,
and replays the traces fuzz mode writes for them. This file is the design for recording and
replaying p3 components: the trace format,
how the proxy records a run without changing it, how replay reproduces it, and what makes replay
portable across runtimes. What is implemented so far is listed below, and in [Phases](#phases).

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
- **The trace format** ([Trace format](#trace-format)) and its recorder interface. Fuzz mode
  writes it for p2 and p3 components, with [steps](#steps) and [labels](#labels), and record mode
  for p2 components.
- **Replay** of p2 and p3 traces ([Replay](#replay)): calls and steps, RETURNED and STARTED,
  streams, futures and error-contexts.
- **Record mode for p3**: async imports, RETURNED or STARTED, with their steps, and streams and
  futures, through pumps.

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
of lines the proxy observed, in the order it observed them. Replay relies on only part of that
order; see Order, below. How a trace is stored is up to the implementation: the recorders write
one JSON line per line, and the bundled recorder prints text instead, e.g. `host @2: t1.call …`,
with `PROXY_TRACE=text`, for reading a fuzz run.

The format is defined once, as the WIT interface `proxy:recorder/types`
([`assets/recorder.wit`](../assets/recorder.wit)), which the generated components use to record
and check lines. `crates/trace` generates its Rust types from it, and the recorders use those:
the bundled recorder maps the interface to them, and the host recorder of `run` converts its own
bindings' types to them. Values are WAVE text, e.g. `ok(#response-3)`, whose type comes from
where they appear: a function's parameter or result, or a stream's or future's payload.

```wit
enum party { host, guest }

// A generated component, which numbers handles and tasks: the imports component, whose handle
// table holds the ends that cross at the main component's imports, and which numbers the guest's
// calls of imports, and the exports component, for its exports.
enum table { imports, exports }

// A call, numbered in order by the generated component that sees it, its `table`: the exports
// component numbers the host's calls of exports, written `t1, t2, …`, and the imports component
// the guest's calls of imports, written `c1, c2, …`. Sync calls are numbered too.
record task-id { table: table, id: u32 }

// A handle: the table its number is in, and its name as WAVE writes it, its type and its number,
// e.g. `#stream-7` or `#fields-3`; see Labels.
record label { table: table, name: string }

record call { id: task-id, is-async: bool, %func: string, args: list<string> }
// The callee's result, `none` for a function without one.
record return-value { id: task-id, value: option<string> }
// The items one copy of a stream moved.
record chunk { %stream: label, items: string }
record future-value { %future: label, value: string }

variant event {
  // `by` is the caller: the host for an export, the guest for an import.
  call(call),
  // The callee blocked before returning, so the caller got a subtask. Only for an async call.
  started(task-id),
  %return(return-value),
  // The caller asked to cancel a pending call. The callee answers with `cancelled`, or with a
  // `return`, if it returns anyway.
  cancel(task-id),
  // The callee acknowledged a cancellation instead of returning.
  cancelled(task-id),
  // A copy: the host wrote the items, which the guest read.
  write(chunk),
  // A copy: the host read the items, which the guest wrote.
  read(chunk),
  // A future's value, and who wrote it (`by`). If the host wrote it, the line is recorded when
  // the host writes, before the guest can read it; the guest may drop its end instead, which
  // isn't recorded. If the guest wrote it, the line is recorded when the host reads it.
  value(future-value),
  // A stream or future end, or a resource, was dropped; `by` is who dropped it.
  drop(label),
}

// One event, who did it, and, for a host line the guest observes outside its calls, the step in
// which it happened: a run of the imports component's executor, or an export call's arrival,
// counted from 1; see Steps.
record line { by: party, at: option<u32>, event: event }
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
  - **The guest's cancel of an import** reaches the proxy as its export's cancellation, which
    drops the call's wrapper, and wit-bindgen acknowledges it at once. So the host answers
    `Cancelled`, without an `at`, as it answers a call; in record mode, the host's call is
    cancelled with it, and a result it returned meanwhile is dropped, which the guest wouldn't
    have read either, as it dropped the call. Only a STARTED call can be cancelled. Replay's mock
    of a call the trace answers with `Cancelled` waits until the guest cancels it.
  - **The host's cancel of an export** isn't recorded yet: the proxy would have to tell whether
    the guest then returned or acknowledged it, which wit-bindgen doesn't say.
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
  label names one handle, from its first line to the one that ends it; see [Labels](#labels),
  also for the table of a label inside a value, which is written by its name alone.
- **Positions.** `at` is the step of a host line the guest observes outside its calls, including
  an export call's arrival; other lines have none. See [Steps](#steps).
- **Copies.** A copy completes once, when both sides have acted, so it is one line. Copies are
  lazy: if the guest never reads a stream the host writes, no `Write` appears. For `stream<T>` a
  line is the chunk one rendezvous copied, so replay can give the guest's reads the same chunks.
  This holds for pumps that don't read ahead; [Read-ahead](#read-ahead-in-pumps) would change it.
  The host's side waits until the other side is ready, with a zero-length write or read, which
  completes once the other side reads or writes and leaves its copy pending, so that the copy
  completes as the host issues it, and its line is recorded before the guest goes on. A future
  has no such wait, so the host's value is recorded as it is written. Replay's jobs wait alike,
  so that the same events wake the executor.
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

- **Resources** of the host are in the imports table, which holds them for both components. In
  record mode, a host resource is labeled by the handle the imports component keeps for it, which
  the exports component gets through `get-handle-*`; in fuzz and dialog modes, a mock by a number
  the imports component gives it when it makes it, counting from 1, which no other mock gets; in
  replay, a mock carries the trace's label. The mock's own handle can't label it: it moves on with
  the mock, which frees its number, and a borrow the guest passes back comes in as the mock
  itself, without a handle. Fuzz mode records the end of every mock, also the host's drops, which
  record mode can't see; they take no step, as the guest doesn't see them, and replay has nothing
  to do for them. A resource of the guest, which the exports component wraps, is in the
  exports table, e.g. the host's drop of a session, `drop #session-1 (exports)`.
- **Streams and futures** are labeled by the handle of the end the proxy received, which it keeps
  for the stream's lifetime: the host's reader in an import's result or an export's arguments, the
  guest's in an import's arguments or an export's result. The proxy pumps it through a pair of its
  own, whose writer never appears in the trace. A stream or future first appears in the value that
  brings it across, in the table of the component that call or result crosses, and every later
  line uses the same label:

  | `by` | `at` | `event` |
  |---|---|---|
  | guest | | `Call { id: c4, is_async: false, func: "test:p3/host/body@0.1.0.consume", args: ["#body-1"] }` |
  | host | | `Return { id: c4, value: Some("(#stream-6, #future-9)") }` |
  | host | 5 | `Write { stream: #stream-6, items: "[52]" }` |

  - **Record mode** prints a call's arguments converted, and its result as received, as it does
    for every value. So the reader it hands on in the arguments is printed by the received end's
    label too, which it registers with wasm-wave's `set_handle_label`, as fuzz mode does.
  - **Fuzz and dialog modes** make the stream themselves: they keep the writer, and hand the
    reader out, which frees the reader's slot. So they label the stream by the writer, also where
    the reader is printed, in the value that hands it out: they register the pair with wasm-wave's
    `set_handle_label`, which prints the reader under its writer's number until the reader has
    been printed. Pumping it through a pair the mock keeps the reader of, as record mode does,
    isn't possible: wasmtime traps a copy between two ends of one component instance, unless
    the payload is numeric.
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
- **An end the proxy gave the guest, coming back, can't be copied from, unless its payload is
  numeric.** The end is the reader of a pair the proxy made, e.g. a pump's, or a mock's in fuzz
  and replay modes, whose writer the proxy keeps. Reading it in the proxy copies between two ends
  of one component instance, which wasmtime traps for a payload that isn't numeric ("cannot read
  from and write to intra-component future/stream with non-numeric payload"), as the component
  model requires. So recording a guest that hands such a future or stream back traps, e.g. an
  HTTP echo that passes a request's trailers, a `future<result<option<trailers>, error-code>>`,
  on to its response; a `stream<u8>`, e.g. the body, is copied. The proxy can't tell that the end
  is its own, as it comes back as a new handle. Pairs made in another component instance would
  avoid the trap, at the cost of a function per payload type to make them and pump them there.
- **Replay compares labels up to renaming.** A table holds more than the values the trace
  labels: waitable sets and subtasks, which each mode makes its own, so the same value can have
  another number in replay. Replay pairs each of the trace's labels with the one it sees for the
  same value, one to one, per table, from the first guest line that has both, and requires every
  later line to agree. A label names a handle until the handle leaves the table, which frees its
  number: at a `Drop`, a future's `Value`, or when the host hands it on, owned, to the guest, in
  an export call's arguments or an import's result; the rule is the same for every label. After
  that, the number may name another handle, as a variable may be bound again. So the recorder
  names each binding apart, as it records each line, from the lines before it alone, which a
  run that traps leaves too: the first keeps the number, e.g. `#stream-7`, and the next are
  `#stream-7-2`, `#stream-7-3`, and so on. A trace's label then names one handle, whatever order
  replay's jobs process the lines in. A host line can come after the guest acted on a handle the
  line brings in, as the host records a copy once its job sees it complete: e.g. a future's value
  with a mock in it, which the guest took and dropped first. So a `Drop` or a `Value` of a label
  no line has mentioned yet ends the binding of the line that brings it in. Replay's labels are
  its own numbers, which it reuses too: a replay label is paired again once the trace's label it
  was paired with has ended. A stream or
  future replay makes, from a value the trace hands out, first appears in a host line, which
  replay doesn't check, so replay pairs its label explicitly, with the binding after that line:
  the end the host keeps. The lines replay performs it renames to its own labels, e.g. the
  guest's resource a host `Drop` names. So replay checks that the labels are used consistently,
  not their numbers. For that, the trace has the end of every handle: fuzz mode records the
  host's drops of error-contexts too.
- `list<u8>` items are WAVE lists for now; p3.md's byte strings (`b"…"`) would be shorter.

## Recording

Recording runs the composition in the real host, as for p2: the imports component forwards to
the host and records, and the exports component records the calls the host makes into the main
component.

### The executor delivers and records host events

In record mode, the imports component routes every interaction with the host through one
executor task, like fuzz mode's:

- **Async imports.** The proxy's export polls the host's call once: if the result is ready, it
  returns it at once. Otherwise it records `Started`, and hands the call to an executor job,
  which awaits the host's subtask and records the `Return` in the step of the run it completed
  in, then gives the result back to the proxy's export, which returns it (its `task.return`). It
  waits for the job as replay's mock waits for the step, holding the executor until it resumes
  (see [Steps](#steps)). The job may borrow the guest's arguments, e.g. a method's `self`: if the
  guest cancels the call, the job is dropped with it, which cancels the host's call too.
- **Running the executor.** There is no `start` in record mode, so the exports component runs
  the executor while the host's calls of async exports are pending, and stops it when the last
  one returns, which lets it finish its jobs; a call that comes meanwhile keeps it running.
- **Streams and futures, by pumps.** The proxy doesn't pass a stream end through, as the copies
  would then go between the host and the guest without it. It keeps the end it received, creates
  a pair of its own, and passes on that pair's other end. A pump is the executor job between the
  two: it reads from one end, writes what it read to the other, and records each copy, and passes
  on a drop the same way. It reads from the writer only once the reader reads, without
  [read-ahead](#read-ahead-in-pumps): a zero-length write to the reader's end waits for that, as
  it completes once the reader reads, and leaves the read pending. A future is pumped once, for
  its value.

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
  the schedule, and which the proxy can't record. So the proxy, the bundled recorder, which today
  prints each line as it is recorded, and the debug component, whose prints fuzz mode and a
  guest's debug builds use, must buffer their output and write it once the run is over, as the
  spike does, or leave the trace to a host recorder. A buffer is lost if the run traps, and a run
  that doesn't end, e.g. under `wasmtime serve`, has no end to write it at.
- **No epoch deadlines that yield.** wasmtime's epoch interruption ends a guest's time slice at a
  point set by a timer thread, wherever the guest is.
  - **A deadline that traps**, as `wasmtime -W timeout=…` sets, only ends the run, and the trace
    with it. A run that finished before its deadline replays; one that hit it stops at a point
    replay has no timer for, so replay goes on past the end of the trace.
  - **A deadline that yields** (`epoch_deadline_async_yield_and_update`), as embedders use to
    time-slice guests, suspends the guest in the middle of its computation, while its instance
    stays entered. The executor can then run a step, and deliver a host event H, before the
    guest's next wait W. Replay can't interrupt the guest, so it runs that step only once the
    guest waits at W. Step numbers stay the same; what moves is where the step falls relative to
    the guest's computation. The guest sees the difference when another event E is pending at W:
    in record, H and E are both pending, and the runtime returns the lower waitable, say H; in
    replay, only E is.

  A p2 guest isn't affected: it is sequential, and sees the host only through calls, so pausing it
  changes nothing it can observe. [The shim](#portability) would lift this requirement.

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
   performs them when the host gets to them, so `at: n` says they happened in step `n`. For a copy,
   it is the step in which the host made its side, e.g. issued the write the guest then read,
   rather than the one in which the host saw it complete, which may come after the guest reacted
   to it. In replay, the executor counts its steps the same way, and a job holding such a line
   waits until the count reaches `n`. There is no host to wake the executor, so a job waiting for
   a later step makes the executor wake itself, by yielding, and its next run is the next step.
   That run first takes an event that came while it yielded, e.g. a guest's write, as a waiting
   executor woken by the event does in the recording: wit-bindgen would only take it after a
   poll, in another run. So the executor's callback, which wraps wit-bindgen's, makes the first
   poll after a yield a probe, which only lets wit-bindgen take a pending event.
   A task that waits for a step from outside the executor, e.g. the mock of a STARTED call, holds
   the executor from the run it waited for until the task resumes: otherwise a run, e.g. for a
   guest's drop, could come in between, which replay, knowing only the step, would place
   differently, as it waits by waking the executor itself, which changes the order in which
   wasmtime delivers the guest's events.

**Export calls** arrive at the exports component, from the host, so their arrival is a step of its
own. On each arrival, the exports component calls `step` on `proxy:conversion/executor`, a sync
function that increments the counter and returns it, as the export `Call`'s `at`; the executor's
later runs are numbered after it. In replay, `start` makes an export call recorded at step `n` once
the count reaches `n - 1`, by calling `step` and then the export. The wait and the step are one
call, `step-after`, which takes the step in the imports component's task as soon as its wait
resumes, while the executor is still held, as above. As two calls, the executor could run between
the wait's return and `start`'s call of `step`. An arrival can't fall within a run of the
executor: the host only calls an export while no wasm is running, unless something suspends where
the proxy can't see it, which the requirements below rule out. The host's drops of the guest's
resources, which `start` makes too, take a step the same way.

`start` calls only the main component's exports, not the host's own functions, as the real host
doesn't call those through the proxy. A call like `consume` of a body the host made would put
both ends of its streams in the generated components, one in each: copies the guest never sees,
which the imports component's executor can't take over, as wasmtime traps a copy between two
ends of one component unless the payload is numeric.

The exports component's jobs, e.g. reading a stream an export returned, run on the task of
`start`, not in the executor: a line of theirs has the last step, and replay's job waits until
the count is there. That places them less exactly than the executor's runs do; see open
question 8.

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
  record, futures that the trace makes ready in replay. Its work in a step depends only on its
  state and the trace's lines for that step. The same goes for `start`: fuzz mode's waits for
  the steps it chooses before an export call, through `step-after`, as replay's waits for the
  trace's, so that the two make the same tasks.
- **Steps in replay where the host woke the executor in the recording.** Replay has no host, so a
  job that waits for a later step makes the executor yield, and come back as the next step.
  wasmtime runs the ready tasks first, as it did before the host's completion in the recording.
- **No suspension the proxy can't see**: no calls to other components where the recording calls
  the host, no blocking WASI 0.2 calls, and no epoch deadlines that yield, as above.
- **One clock for both components.** The exports component takes its steps from the same
  counter, through `step` and `step-after`, as [Steps](#steps) describes.

### What recording can't reach

The proxy puts a hop, its own task, between the host and the guest. With delivery on arrival:

| | |
|---|---|
| RETURNED | preserved, when the proxy forwards to host functions rather than another component |
| several host results pending at one guest wait | possible, as nothing holds them back. A wit-bindgen guest still takes them one per wakeup, the lowest waitable first, with or without the proxy: its callback receives one event at a time, and its futures don't see the others until their own events |
| a new export call | preserved: forwarded in the same task |
| a guest read or write that completes at once, as the host had data or was reading | only if the pump [reads ahead](#read-ahead-in-pumps); otherwise it blocks first |
| a host result arriving before the guest's next wait | may arrive after it instead, as the executor needs a turn to pass it on: a slower host, which is legal. [Read-ahead](#read-ahead-in-pumps) doesn't help: an async import's result has nothing to read ahead, and a stream's data that arrives just before the wait still waits for the pump. A wit-bindgen guest, whose callback gets one event per wakeup, can't tell the difference when this is the only event, but can when another is pending at the same wait: without the proxy, the lowest waitable of the two comes first, with it, the other one. A guest that calls `waitable-set.poll` also finds nothing pending |

A run that traps, or is killed, during an export call cuts its trace short:

| | |
|---|---|
| the guest's last copies before a trap: its drop of a stream or future, or the value of a future it wrote | may be missing, as the host records them once it sees them complete, after the guest went on, and the guest may trap first. A trace with an export call that has no answer is cut: replay accepts a guest's line about a stream or future that the trace lacks, without checking it, and takes what the guest writes since, so that it goes on to the same trap. The host's own copies, and the guest's writes to its streams, are recorded as the host completes them, so a cut trace has every value and item the guest got, and every item the host took. A trap after every export call returned, e.g. in a task the guest spawned, leaves a trace that looks whole, whose replay then reports the missing line as a divergence |

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
  happen together, so one line describes both. This is record mode's choice:
  - A zero-length write to the reader's end tells the pump that the reader reads, but not how
    much: the pump reads up to its own capacity, 4096 items, and the reader's reads may split the
    chunk, which replay's writes split alike.
  - A host's end may answer a zero-length copy at once, without being ready, as the component
    model allows it; wasmtime's do. The pump then takes a chunk of the guest's as soon as the
    guest writes it, which a host that isn't ready, e.g. under backpressure, wouldn't have.
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

- Recording a wasi-http component under `wasmtime serve -S p3,http,cli --no-logging-prefix`
  needs no change: `cli` for the WASI 0.2 interfaces a component built for `wasm32-wasip2` also
  imports, and no prefix so that the output is the trace. wasmtime reuses an instance for several
  requests of a p3 component, so a trace has an export call for each of them, which replay
  makes in turn; an instance's trace is one recording. Recording `tests/wasi_http_p3.wasm` this
  way replays, for each of its routes but its echo (see [Labels](#labels)). Viceroy is for p2
  components.
- Recording under `proxy-component run` passes the main component's p3 imports through to the
  host, so `run` needs `wasmtime_wasi::p3` and wasmtime-wasi-http's p3 linker, and, for
  `wasi:http/handler`, a way to receive requests, as `--invoke` can't build one.

## Spike results

[`tests/fixtures/spike-p3-race`](../tests/fixtures/spike-p3-race) tests steps in wasmtime 49, with
hand-written components: a guest whose `run` calls `fetch("a")` and `fetch("b")` concurrently and
joins the results in the order its wakeups receive them; a record proxy whose `fetch` forwards to
wasmtime's `wait-for`, with delays from `DELAY_A` and `DELAY_B` or random, and whose executor
counts its steps; a replay proxy that delivers each result in its recorded step; and a driver
that runs the executor alongside the guest. `make -C tests/fixtures/spike-p3-race` records each
scenario, replays it, and requires the same result, the same trace, and the same results in each
of the guest's wakeups.

- **The clock works.** All scenarios replay exactly: each result first (`"AB"`, `"BA"`), each call
  RETURNED, both RETURNED, the guest busy until both results are ready, and random delays.
- **RETURNED needs the host to be host functions.** With the host as a component instead
  (`make -C tests/fixtures/spike-p3-race component-host`), a call the proxy saw RETURNED reached
  the guest as STARTED, and the guest got the two results in separate wakeups, while replay gave
  them together. wasmtime runs the proxy's call into a component as a queued work item, which
  suspends the proxy while it is entered, and holds back the guest's next call into it
  (`do_not_enter`, `concurrent.rs:774`). Calling a host component in replay as well made other
  scenarios diverge, as a component host suspends differently in record and replay. Removing it,
  i.e. forwarding to host functions, fixed it.
- **Sync WASI 0.2 calls change the schedule.** With the proxy and the guest printing each line
  with `println!`, several scenarios diverged; buffering the output fixed them.
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
- **Recorder interface** (`assets/recorder.wit`): `types` mirrors the Rust types; `record`
  records a line; `replay` has `next-host-line`, the next line `start` performs, an export call
  or the host's drop of a resource of the guest; `take-host-line`, the next host line about a task
  or a handle, e.g. the `Return` of a STARTED call, which its mock waits for; `check`, which
  checks a guest line, up to renaming, and returns the host lines that answer it at once, e.g. an
  import's `Return`; `pair`, which pairs a label of a stream or future replay makes; `finish`,
  which checks that replay reached every line; `trace-cut`, whether the trace ends during an
  export call, e.g. in a trap, whose last guest copies may be missing; and, in both, `fail`,
  which reports a panic of a generated component, which has no stderr of its own to print it.
  Waiting for a step is the executor's, not the recorder's. The bundled recorder
  (`assets/recorder*.wasm`) and the host recorder of `run` (`--use-host-recorder`) implement it,
  with `Replay` of `crates/trace`.

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
without an `at`. The induction holds under the requirements of
[Why the step is enough](#why-the-step-is-enough-and-a-gate-isnt-needed), and two more, none of
them proven yet:

- **The executor is the only way in.** Every host event reaches the guest in a run of the
  executor; see [Recording](#the-executor-delivers-and-records-host-events).
- **The trace holds every input**: the values, RETURNED or STARTED, and the step of each host
  line.

Of the requirements, deterministic runtime choices are the one the component model doesn't
guarantee (`Concurrency.md` allows nondeterminism); wasmtime 49 happens to make them, which is why
replay in another runtime needs [the profile or the shim](#portability).

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
- **Fuzzed schedules, replayed**: `make test-fuzz-replay` fuzzes the p2, p3 and p3-record
  fixtures, `wasi_http_p3`, and the spike's guest, and replays each trace, which has to end as
  the fuzzing run did: to the end, or in the guest's trap, e.g. a panic on an error the fuzzed
  host returned, which is what fuzzing finds. A trap in a generated component or the recorder,
  e.g. a divergence, fails it (`tests/run-wasm.sh`).
- **Enumerated scenarios**: for small guests, every placement of host completions across steps,
  rather than random delays, and guests that merge nondeterministically, which are the ones steps
  exist for.
- **The assumptions, checked at run time**: in replay, a step count that differs from the
  recording's, which replay reports; in record mode, a host event that the guest observes outside
  a step.
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
   reimplementing the component model's async runtime inside the guest. Positions by the guest's
   waits, rather than by the executor's steps, also hold when the guest is suspended where the
   proxy can't see it, e.g. by an epoch deadline that yields, or a blocking WASI 0.2 call: replay
   releases each host event so that every wait finds the same events pending as in the
   recording.

Patching the runtime, e.g. a scheduling hook in wasmtime, would also work, but replay must not
depend on runtime patches. wasmtime 49 has the start of its own record and replay
(`Config::rr`), without concurrency yet.

## Phases

1. **Spike, in wasmtime**: done, see [Spike results](#spike-results).
2. **Trace format** in `crates/trace`, and the recorder interface: done.
3. **Replay**, tested on traces that fuzz mode writes, with the checks of
   [Correctness](#checking-the-implementation): done. It replays 100 of 100 fuzz traces of the
   p3 fixture, and 60 of 60 of `wasi_http_p3`.
4. **Record**, with the executor and its pumps, under `wasmtime serve` and `run`. Async imports,
   streams and futures are done: `make test-record-p3` records `tests/fixtures/p3-record`
   under `wasmtime run`, with delays that make its calls RETURNED or STARTED, and stdin copied to
   stderr through their streams, and replays each trace; `make test-record-http-p3` records
   `tests/wasi_http_p3.wasm` under `wasmtime serve`, for several requests, and replays it. Next
   are `proxy-component run` as a host, and the ends the guest hands back (see
   [Labels](#labels)).
5. **Portability across runtimes**: the profile proposal, or the shim.

## Open questions

1. **Steps with several arrivals**, and the host's side of stream copies (wasmtime's
   `StreamProducer::poll_produce`, which is called when the guest reads): whether counting
   steps places them exactly.
2. **[Read-ahead in pumps](#read-ahead-in-pumps)**: whether to read ahead, which makes the
   guest's immediate copies reachable, at the cost of the host's view, and of copy lines that
   record the host's chunks rather than the guest's. Record mode doesn't, for now.
3. **`list<u8>`** as WAVE lists or byte strings.
4. **Which way to portability**: the profile, the shim, or both.
5. **Hosts that are components**, e.g. virtualized WASI composed with the main component: whether
   to support recording them, which needs the guest to always see STARTED, or the replay to
   reproduce the component host's suspensions.
6. **Guests that see several events at once**: bindings that drain pending events, or call
   `waitable-set.poll`, unlike wit-bindgen's; the spike covers only wit-bindgen guests.
7. **The recorder's output**: buffering it to the end, or only a host recorder; and where to
   write a buffer when the run traps, or doesn't end, as under `wasmtime serve`.
8. **The steps of the exports component's jobs**, e.g. reading the stream `serve` returned,
   which run on the task of `start`, not in the executor, so that their lines have the last step
   rather than their own. They replay in every test so far. To place them exactly, the exports
   component could hand its ends of the guest's streams and futures to the imports component's
   executor, with a function per payload type.
