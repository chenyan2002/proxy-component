# Record and replay of WASI p3 components

proxy-component records a run of a component and replays it without the host. 
This is the design for components that use WASI p3, the component
model's async: async functions, streams, futures and error-contexts. It covers what a trace
records, how the proxy records a run without changing it, how replay reproduces it, and what
makes replay portable across runtimes. [Status](#status) says what is implemented.

## Overview

The proxy is two generated components around the main component, the guest. The imports
component stands between the guest and the host: in record mode, it forwards each import to the
host and records it, and in fuzz, dialog and replay modes it plays the host. The exports
component stands between the host and the guest's exports: in record mode the host calls them
through it, and in the other modes, its `start` function makes the calls, as the trace.

Every mode of the proxy has the same structure, so that a run is the same in record and replay.
In particular, the imports component runs all of the host's side of the run in one task, the
executor, in whose runs a trace's positions are counted (see [Steps](#steps)).

## Where nondeterminism comes from

Component model concurrency is cooperative: within an instance one thread runs at a time, and it
observes other threads, or the host, only at defined points. Two kinds of choices decide a run:

- **The host's timing**: when an async import returns, and whether at once (RETURNED) or after
  starting a subtask (STARTED); when and how much the host reads from or writes to a stream or
  future; and when the host calls an export.
- **The runtime's choices**, which the component model leaves open: which of several pending
  events a wait returns, which ready thread resumes, and the like.

A trace records the first. The second must be the same in record and replay: wasmtime makes its
choices without randomness, from the state of the whole store, which replay has to reproduce
closely enough.

## wasmtime and wit-bindgen

The design depends on how wasmtime (49) schedules async work, and on how wit-bindgen (0.63) 
drives a task.

### wasmtime

- **Run queues.** wasmtime keeps a high-priority queue, e.g. delivering an event to a task that
  waits for it, or starting a call, and a low-priority one, for a task that yielded; it takes from
  the first while there is work in it. So after the executor yields to wake itself, every task with
  a pending event, e.g. the guest, runs before the executor's next run.
- **Which pending event a wait returns.** A waitable set keeps its pending events in a
  `BTreeSet<Waitable>`, ordered by kind (host task, guest task, stream or future end) and then by
  index in a table the whole store shares, which reuses freed slots. A task gets the lowest one
  first, one per callback. The index isn't the guest's handle: it depends on every task and resource
  in the whole shared store, including the proxy's and the recorder's. This makes the replay difficult
  to replicate the exact same scheduling, because the replay composition uses different import and export
  components. See [Answers in the same step](#answers-in-the-same-step).
- **Events are only delivered to a waiting task.** An event for a task that yielded, rather than
  waits, stays in its set until the task waits, or polls the set itself.
- **Copies within one instance.** A copy between two ends of one instance traps unless the payload
  is numeric ("cannot read from and write to intra-component future/stream with non-numeric
  payload"), as the component model requires. It's checked when the copy happens, so a component
  can't tell beforehand; see [Ends that come back](#labels).
- **Zero-length copies** ask whether the other side is ready: a zero-length write completes once
  the reader reads, and leaves the read pending for the next write; a zero-length read likewise
  waits for a write. A host's end may answer at once, without being ready, which wasmtime's do.
  Futures have no such copies.
- **A call into a component** runs as a queued work item, which suspends the caller while it stays
  entered, and holds back calls into it (`do_not_enter`); a sync WASI 0.2 call, e.g. a print, can
  suspend an instance the same way. Either changes the schedule where the proxy can't see it.
- **`wasmtime serve`** reuses an instance for many requests of a p3 component, so one instance's
  trace has a call for each.

### wit-bindgen

- **A task's callback loop.** wit-bindgen runs a task's futures in one callback: it delivers the
  callback's event, if any, polls the futures that were woken, and then, if one of them woke the
  task again, polls the task's waitable set itself, delivers an event it finds, and polls again;
  otherwise it yields (`YIELD`), or waits on the set (`WAIT`). It polls only woken futures, so a
  future that wants another poll must wake itself.
- **After a yield**, wasmtime resumes the task without an event, and wit-bindgen polls its futures
  before it takes the events that came meanwhile, which it gets only in a later poll. A task woken
  by an event takes it before polling. For the executor, whose polls are steps, that difference
  would move an event to the next step in replay, where the executor wakes itself, but not in the
  recording, where the event woke it. So the executor's callback wraps wit-bindgen's: resumed after
  a yield, its first poll is a probe that only wakes the executor, so that wit-bindgen takes a
  pending event, if any, before the run. wit-bindgen's generated glue is patched to call the
  wrapper.
- **Waking another task**, e.g. a sync mock queuing a job for the executor, goes through
  wit-bindgen's inter-task wakeup, a stream that the sleeping task reads.
- **Spawned futures** run in the task of the export that spawned them, which lives until they end;
  the exports component's own jobs run on the task of `start`, not in the executor.
- **A pending call moves between tasks.** The proxy's export polls the host's call once, for
  RETURNED, and hands it to the executor otherwise; the executor's poll moves its waitable to the
  executor's task.
- **Cancellation.** Dropping a pending import call cancels it; the callee's export task gets the
  request, its future is dropped, and wit-bindgen acknowledges with `task.cancel`. So the proxy
  always answers a cancel with `cancelled`, and a callee's choice between returning and
  acknowledging isn't visible to its caller.
- **Streams and futures.** A write of no items is a zero-length write, and a read with no capacity
  a zero-length read. A future's writer dropped unwritten writes a default value, which the proxy
  never relies on.

## Trace format

A trace is the list of lines the proxy observed, in the order it observed them. Its types are
defined once, in WIT ([`assets/recorder.wit`](../assets/recorder.wit)); values are WAVE text,
e.g. `ok(#response-3)`, whose type comes from where they appear.

```wit
enum party { host, guest }

// The generated component that numbers a handle or a call: the imports component, for the
// guest's imports, and the exports component, for its exports.
enum table { imports, exports }

// A call, numbered in order by the component that sees it: `t1, t2, …` for the host's calls of
// exports, `c1, c2, …` for the guest's calls of imports.
record task-id { table: table, id: u32 }

// A handle, by its table and its name, e.g. `#stream-7`; see Labels.
record label { table: table, name: string }

record call { id: task-id, is-async: bool, %func: string, args: list<string> }
record return-value { id: task-id, value: option<string> }
record chunk { %stream: label, items: string }
record future-value { %future: label, value: string }

variant event {
  call(call),             // by the caller
  started(task-id),       // the callee blocked, so the caller got a subtask
  %return(return-value),  // by the callee
  cancel(task-id),        // the caller asked to cancel a pending call
  cancelled(task-id),     // the callee acknowledged it, instead of returning
  write(chunk),           // the host wrote the items, which the guest read
  read(chunk),            // the host read the items, which the guest wrote
  value(future-value),    // a future's value, by its writer
  drop(label),            // a handle was dropped, by whoever dropped it
}

// One event, who did it, and, for a host event the guest observes outside its calls, the step in
// which it happened; see Steps.
record line { by: party, at: option<u32>, event: event }
```

The lines of a call (`call`, `started`, `return`, `cancel`, `cancelled`) belong to its task, and
those of a stream or future (`write`, `read`, `value`, `drop`) to its label. Lines of different
tasks or handles may come in any order: what orders them is each task's and handle's own sequence,
and the steps.

An example, for `GET /wait?ms=250` on a wasi-http handler, in the text form, where `<--` puts
data into a stream or future and `-->` takes it out:

```
host @1: t1.call async wasi:http/handler.handle(#request-1)
guest: c1.call wasi:http/types/request.get-method(#borrow-request-1)
host: c1.return get
guest: c2.call async wasi:clocks/monotonic-clock.wait-for(250000000)
host: c2.started
host @4: c2.return ()
guest: c3.call wasi:http/types/response.new(#fields-2, some(#stream-5), #future-6)
host: c3.return (#response-3, #future-7)
guest: t1.return ok(#response-3)
host @6: #stream-5 --> [97, 115, 107, …]
guest: drop #stream-5
```

The guest returns the response before it writes its body, which the host then reads.

Some properties the types don't show:

- **Copies.** A copy completes once both sides have acted, so it is one line, the items one
  rendezvous moved; copies are lazy, so a stream nobody reads has no `write`. The host's side waits
  until the other side is ready, with a zero-length read or write, so that each copy completes as
  the host issues it, and is recorded before the guest goes on.
- **Futures.** A future can't wait for its reader, so the host's value is recorded as the host
  writes it; the guest may drop its end instead, which isn't recorded. The guest's value is
  recorded when the host takes it.
- **Cancellation.** The guest cancels an import by dropping its pending call. The proxy
  acknowledges it at once, so the trace has the guest's `cancel` and the host's `cancelled`; in
  record mode the host's call is cancelled with it. Only a STARTED call can be cancelled.
- **No caller identity.** The component model doesn't say which of the guest's tasks made an
  import call, so a trace can't tell which guest lines a host line caused, only their order.

## Labels

A label names a handle in one of the two generated components' tables, by its type and number,
e.g. `#fields-3`. Neither table is the guest's, which also holds handles the proxy never sees,
and the two can hold the same number, so a label carries its table. Labels only need to be
consistent: replay treats them as names.

- **Host resources** are labeled by the host's handle in record mode, and in the other modes by a
  number the mock gets when it is made.
- **Streams and futures** are labeled by the end the proxy holds for the stream's lifetime: the
  end it received, or, for one it made, its writer. A stream first appears in the value that
  brings it across.

**Bindings.** A number names a handle until the handle leaves the table: at a `drop`, a future's
`value`, or when the host hands it on, owned, to the guest. The number can then name another
handle, as a variable can be bound again. So the recorder names each binding apart as it records
each line, from the earlier lines alone: the first keeps the number, e.g. `#stream-7`, the next
are `#stream-7-2`, `#stream-7-3`, and so on. A label then names one handle, and a trace cut short
names its bindings as the whole trace would. A guest's drop can be recorded before the host's
line that brought the handle in, as the host records a copy once it sees it complete, so an end
of a label no line has mentioned belongs to the binding that line starts.

**Renaming.** Replay's handle numbers differ from the recording's, as each mode makes handles of
its own. Replay pairs each trace label with its own label for the same value, one to one, from
the first line that has both, and requires every later line to agree; a replay label is paired
again once the trace's label has ended. A stream replay makes itself, from a value it hands out,
is paired explicitly. So replay checks that labels are used consistently, not their numbers.

**Ends that come back.** When the guest hands back a stream or future the proxy gave it, the
proxy gets a new handle, and nothing tells it that the end is its own:

- It gets a new label: the trace stays consistent, but doesn't show the two as one.
- It can't be read by the proxy, if its payload isn't numeric: that would copy between two ends
  of one instance, which the component model forbids, and wasmtime traps. E.g. an HTTP echo that
  passes a request's trailers future on to its response can't be recorded. Making the proxy's
  pairs in another component instance would avoid it.

## Recording

Recording runs the composition in the real host. The imports component forwards every
interaction with the host through its executor:

- **Async imports.** The proxy polls the host's call once: if the result is ready, it returns it
  at once, so the guest sees RETURNED. Otherwise the call is STARTED, and an executor job awaits
  the host's subtask, and records its `return` in the step it completed in.
- **Streams and futures** pass through pumps. The proxy keeps the end it receives, makes a pair of
  its own, and hands on the pair's other end; an executor job copies between the two, records each
  copy and drop, and reads from the writer only once the reader reads, without read-ahead.
- **The executor runs** while the host's calls of the guest's exports are pending, as there is no
  `start` in record mode, and finishes its jobs when the last one returns.

So nothing the host does reaches the guest outside a run of the executor, which is what lets a step
mark when each host event happened. Putting the executor in between must not change the run, which
these rules keep:

- **Delivery on arrival.** The executor passes each host result on as soon as it arrives.
- **RETURNED is preserved**, as above. This needs the host to be the runtime's own functions: a
  host that is another component suspends the proxy as it calls it, so that the guest sees
  STARTED, in a way replay can't reproduce.
- **Nothing suspends where the proxy can't see it**: no blocking WASI 0.2 calls in the proxy or the
  recorder, e.g. printing, and no epoch deadlines that yield, which pause the guest mid-computation
  at a point replay has no timer for. A deadline that traps only ends the run.

## Steps

The executor is a single task that runs the proxy's jobs. Each of its runs is a **step**, and so is
each arrival of an export call; one counter, from 1, numbers them in both components. A host line
the guest observes outside its own calls, e.g. a STARTED call's `return`, a copy, a host's drop, an
export call, has `at`: the step it happened in. A guest line, or a host line that answers a guest
call before it returns, e.g. a `return` of a RETURNED call, has none: the guest's call places it.

**Replay** counts its steps the same way, and performs each host line in its step. With no host to
wake the executor, a job waiting for a later step makes the executor wake itself; the next run
first takes an event that came meanwhile, e.g. a guest's write, as an executor woken by that event
does in the recording (see [wit-bindgen](#wit-bindgen)). A task that waits for a step from outside
the executor, e.g. a mock of a STARTED call, or an export call waiting to arrive, holds the
executor until it resumes: otherwise another run could come in between, which replay, knowing only
the step, would place differently.

**Why the step is enough, and a gate isn't needed.** The trace order alone isn't a schedule:
released in trace order, two results could reach the guest before or after a different one of its
waits. component-ui's design holds every result until the guest is idle, which excludes schedules,
turns every RETURNED call into a STARTED one, and needs to know when the guest is idle, which the
proxy can't see. Instead, every host event reaches the guest in a step; if the runtime's choices
are deterministic, everything between two steps follows from what happened before, so "event H in
step n" means the same in record and replay. This requires:

- the same executor in record and replay, with only the leaves different: host calls, or the
  trace's lines;
- deterministic runtime choices, given the same state;
- nothing suspending where the proxy can't see it, as above;
- at most one event pending for the guest at once, see below.

A spike, [`tests/fixtures/spike-p3-race`](../tests/fixtures/spike-p3-race), confirmed the clock in
wasmtime, with a guest that joins two results in the order its wakeups receive them.

### Answers in the same step

Steps place each host event, but not the order of two events that reach the guest in one step,
e.g. two calls returning in the same run of the executor. When both are pending at one of the
guest's waits, wasmtime gives the guest the lowest waitable first, by its index in a table the
whole store shares, not by the guest's own handle. That index depends on every task and handle
the store made before, also the proxy's and the recorder's, which differ between record and replay
by design. So the guest may take the other event first, while every step agrees, and diverge,
e.g. a `select` of two such calls.

The proxy doesn't see which event the guest took, so the trace can't record it; see
[Ordering answers in the same step](#ordering-answers-in-the-same-step) for ways to make it
replayable.

## What recording can't reach

The proxy puts its own task between the host and the guest, and observes only its side:

| | |
|---|---|
| RETURNED | preserved, with the runtime's own host functions |
| several results pending at one guest wait | possible, but which the guest takes first isn't replayed; see above |
| a guest copy that completes at once | not reached without read-ahead: the guest's copies block first |
| a host result arriving before the guest's next wait | may arrive after it, as the executor needs a run to pass it on: a slower host, which is legal |
| the last copies before a trap | the guest's drops, and the values of futures it wrote, which the host sees only after the guest went on, may be missing: see below |
| an end the guest hands back | traps, unless its payload is numeric; see [Labels](#labels) |
| the host's cancel of an export | not recorded yet: the proxy would have to see whether the guest returned or acknowledged it |

Whatever schedule happens is recorded completely; the proxy only makes the host look slower.
Covering other schedules is exploration, which fuzz mode does.

**Traces cut short.** A run that traps, or is killed, during an export call leaves a trace whose
last guest copies may be missing. Replay treats a trace with an unanswered export call as cut: it
accepts a guest's line about a stream or future that the trace lacks, and lets the run go on to
the same trap. The data the guest got, and the host took, is always recorded, as the host's side
of a copy completes it.

**Read-ahead** would make a guest's immediate copies reachable: a pump that keeps a read pending on
one side has data ready for the other. But the host is then read earlier, and more, than the guest
asked, which changes what it sees, e.g. a producer that produces only when read, and a copy line
is no longer one rendezvous of the guest's. Record mode doesn't read ahead.

## Replay

Replay runs the composition with no host. The imports component answers every import from the
trace, the exports component's `start` makes the export calls, and the executor performs each host
line in its step. Every guest line is checked against the trace, up to renaming; a mismatch is a
divergence, reported with the expected and the actual line, and a stall shows up as a deadlock.
At the end, replay checks that it reached every line.

The recorder, a component bundled with the proxy or implemented by the host, reads and writes the
trace, and pairs labels; waiting for steps is the executor's. Fuzz mode writes the same format, so
that its runs replay too.

## Correctness

Replay is sound if every run that recording sees replays exactly; coverage, recording every
schedule the composition could have, isn't a goal (see
[What recording can't reach](#what-recording-cant-reach)).

**Soundness, by induction on steps.** If record and replay are in the same state when step `n`
begins, and step `n` performs the same host lines, they are in the same state when step `n + 1`
begins; between steps, the guest's lines are checked, and its calls answered by the lines without
an `at`. This holds if the executor is the only way in, the trace holds every input, and the
requirements of [Steps](#steps) hold. The trace doesn't hold one input wasmtime's choices depend
on, the store-wide order of pending events, so the induction holds only while the guest finds at
most one.

**Fork-join guests**, without determinacy races, give the same result in every schedule, so
recording their values would be enough. A component model guest isn't one in general: it can
observe the order completions arrive in, as a `select` does, which is what steps record.

**Checking the implementation:**

- Replay checks itself, for each trace: a wrong replay shows up as a divergence or a stall.
- Fuzzed schedules are replayed: fuzz mode explores the host's timing, and each trace has to
  replay, or end in the same trap of the guest's.
- Recordings under `wasmtime run` and `wasmtime serve` are replayed.
- Further: a fixpoint test, replaying while recording and comparing the traces; enumerating every
  placement of completions for small guests; and a model of the spec, whose reference
  implementation marks its nondeterministic choices, to check the design apart from wasmtime.

## Portability

Replay in the runtime that recorded needs the same runtime version and configuration, and the
proxy, which wasmtime is deterministic enough for, but for
[answers in the same step](#answers-in-the-same-step). Replay in another runtime needs the
runtime's choices to be the same in both. Two ways:

1. **A deterministic scheduling profile** in the component model, as core Wasm has for NaNs: fix
   each choice the spec leaves open, e.g. the lowest waitable first, by the waiting instance's own
   handle, the oldest ready thread first, and when a host completion is delivered relative to
   ready tasks. It fixes the choices rather than recording them; runtimes would adopt it behind a
   flag, as they do async.
2. **A shim inside the guest**: rewrite the guest component to put a core module between its core
   instance and its async built-ins, which reports what each wait returned to the proxy. Replay
   then releases exactly what each recorded wait saw, also when the guest is suspended where the
   proxy can't see it. It observes the built-ins, and doesn't reimplement them.

Patching the runtime would also work, but replay must not depend on runtime patches.

## Status

1. **Trace format and recorder interface**: done.
2. **Replay**, tested on fuzzed traces: done.
3. **Record**: async imports, streams and futures, and the guest's cancels, under `wasmtime run`
   and `wasmtime serve`: done.
4. **Portability across runtimes**: the profile, or the shim.

## Open problems

### Ordering answers in the same step

When two events reach the guest in one step, which it takes first depends on wasmtime's store-wide
indexes, which differ between record and replay (see
[Answers in the same step](#answers-in-the-same-step)). Fuzzing a guest that races two calls,
keeping the first, diverges in about 3 of 100 runs; any guest that merges results, e.g. with
`select`, is exposed, in record mode too, when two host results arrive before the executor's next
run.

- **One answer per run.** The executor releases at most one event for the guest per run, and wakes
  itself for the next; wasmtime then runs the guest, at high priority, before the executor's next
  run, so the guest never finds two of the proxy's events pending, and steps order them. It's an
  executor change: a release queue, and a rule for which event goes first, e.g. the trace's order.
  The cost: recording can't reach several results pending at one wait, which a guest that drains
  them would notice, and the second result reaches the guest a run later.
- **The shim** records what each of the guest's waits returned, which makes the order exact, at
  the cost of rewriting the guest (see [Portability](#portability)).
- **A deterministic profile** ordering pending events by the waiting instance's own handles would
  make wasmtime's choice a function of the guest's own history, and need nothing recorded.
- **Making the store's history the same** in record and replay isn't practical: the leaves differ,
  host calls against the trace's lines, and the recorder is called differently.

### Ends the guest hands back

A stream or future the proxy gave the guest, coming back, has its writer in the same instance, so
the proxy can't read it unless its payload is numeric (see [Labels](#labels)). Passing a request's
body and trailers on to a response, common in HTTP handlers, traps the recording. Options:

- **Pairs in another instance.** If a separate component instance made the pairs and wrote to them,
  a returning end would be read across instances. It needs a function per payload type, to make a
  pair and pump into it there, and the steps of that instance's copies would be placed only as
  exactly as the exports component's jobs are (see below).
- **Recognizing the end.** If the proxy knew a returning end was its own, it could pass it on
  rather than read it. Nothing in the component model identifies an end across components, and
  handle numbers are reused; it would need a runtime or spec addition.

### The exports component's jobs

The exports component's own jobs, e.g. reading a stream an export returned, run on the task of
`start`, not in the executor, so their lines have the last step rather than their own, and replay
places them only by waiting until the count is there. Everything replays so far. Giving each copy
a step of its own made it worse, as two tasks then took steps in an order the runtime decides.
Handing these ends to the imports component's executor, with a function per payload type, would
place them exactly.

### The host's cancels of exports

The host may cancel a pending export call, e.g. `wasmtime serve` when a client goes away. The proxy
sees the cancel, but not whether the guest then returned or acknowledged it, as wit-bindgen hides
the callee's choice; and replay's `start` would have to cancel the call in its step. Not recorded
yet.

### Traces cut short

A trace is cut when the run trapped or was killed during an export call; then the guest's last
drops, and the values of futures it wrote, may be missing, and replay accepts their absence. A
trap after every export call returned, e.g. in a task the guest spawned, leaves a trace that looks
whole, whose replay reports the missing line as a divergence. Recording the run's end, e.g. by the
host, would tell the two apart.

### The recorder's output

The bundled recorder prints each line as it records it, a blocking WASI 0.2 call, which can
suspend an instance where the proxy can't see it. No test has shown a divergence from it, but
nothing rules it out. Buffering to the end loses the trace on a trap, and a run under
`wasmtime serve` doesn't end; a host recorder avoids both, but needs a host that implements it;
writing through a WASI p3 stream from the executor would make output part of the steps.

### Read-ahead in pumps

Without read-ahead, a guest's copy never completes at once, which some guests may behave
differently on. With it, the host is read earlier and more than the guest asked, which a host that
produces only when read would see, and a copy line is no longer one rendezvous of the guest's.
Record mode doesn't read ahead.

### Hosts and guests the design doesn't cover

- **Hosts that are components**, e.g. virtualized WASI composed with the guest, suspend the proxy
  as it calls them, so that the guest sees STARTED where the proxy saw RETURNED; replay would have
  to reproduce their suspensions.
- **Guests that drain several pending events**, or poll their waitable sets, unlike wit-bindgen's;
  one answer per run would leave them nothing to drain.
- **Sync exports** can't run the executor in record mode, so host work they start, e.g. a stream
  they return, makes no progress until an async export runs it.
- **`proxy-component run`** doesn't host p3 components yet: it needs wasmtime's p3 WASI and HTTP
  linkers, and a way to call a handler.

### Which way to portability

Replay in another runtime needs its choices to match: a deterministic profile, agreed in the
component model and adopted by runtimes, or a shim, which works without anyone's agreement but
rewrites the guest. Which one, or both, is open.

### Smaller ones

- `list<u8>` items are WAVE lists; byte strings would be shorter.
- No caller identity: a trace can't tell which guest lines a host line caused, which limits
  deriving other schedules from it, e.g. for systematic exploration; the shim could add it.
