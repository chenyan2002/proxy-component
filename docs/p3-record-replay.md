# Record and replay of WASI p3 components

proxy-component records a component's interactions with its host and replays them without that
host. WASI p3 adds async calls, streams, futures, and error-contexts. Replaying their values is
not enough: the guest can observe when operations complete and which completion arrives first.

This document describes the current design, its invariants, and the conditions under which
replay is correct. **Exact replay is not established for all p3 guests**; in particular,
[multiple pending events](#answers-in-the-same-step) can produce a different observation order.
The implementation currently targets wasmtime 49 and wit-bindgen 0.63.

## Architecture

```text
host ↔ exports proxy ↔ guest ↔ imports proxy ↔ host
```

The imports proxy forwards guest imports while recording and substitutes host behavior while
replaying. The exports proxy forwards host calls while recording; replay's `start` drives the
guest exports from the trace. Fuzz and dialog modes use the same infrastructure with generated
or user-chosen host behavior. The recorder component, or a host implementation of its interface,
stores events; replay's recorder interface checks events and supplies recorded answers.

### Why p3 needs more machinery than p2

For the synchronous p2 interfaces handled here, a call boundary supplies an observation point:
record the arguments and result, then substitute the result during replay. Even readiness from
`poll` is returned through an intercepted call.

In p3, work can outlive its introducing call. A sync import can return a stream that must be
written later; an async import can remain pending while other calls run. Stream/future operations
are canonical built-ins, not imports the proxy can intercept. To virtualize their contents, the
proxy creates a wrapper stream or future in place of the original. During recording, a pump
connects the wrapper to the real stream or future, forwarding and recording the transferred values.

The wrapper creates two separate requirements:

- **Run work after the call returns.** Pumps need a long-lived async task to drive them, even
  when a sync function created the wrapper. This is ordinary background-work management.
- **Reproduce when results become visible.** Replay must deliver pending results and stream
  contents at the corresponding points in the guest's execution, not merely reproduce their
  values. This requires a record/replay scheduling policy.

The imports proxy's executor currently serves both purposes: it drives the deferred jobs and
gives their progress a shared logical clock. The runtime still schedules component tasks; the
executor drives futures inside one of those tasks. The exports proxy's own jobs run separately,
a [limitation](#limitations).

A custom scheduler is not intrinsic to p3. It is our current readiness-placement mechanism
without runtime instrumentation. A deterministic runtime could replace part or all of that
policy, but must control external readiness as well as task ordering; see [Portability](#portability).
The [proposed executor protocol](executor-protocol.md) specifies a possible refactor and its
verification obligations, not the implementation described here.

## Trace format

A trace is a sequence of observations, not a total execution schedule. The authoritative types
are in [`assets/recorder.wit`](../assets/recorder.wit), implemented by
[`crates/trace`](../crates/trace). Each line is `(by, at, event)`:

- `by` is `host` or `guest`, describing the event's party, not the component recording it.
- `at`, when present, is a logical step, not elapsed time; see [Steps](#steps).
- Values use WAVE text, with types determined by their position, e.g. `ok(#response-3)`.

| Events | Meaning and identity |
|---|---|
| `call`, `started`, `return`, `cancel`, `cancelled` | A call's lifetime, identified by `task-id = (table, id)` |
| `write`, `read` | Items transferred from host to guest, or guest to host, identified by stream label |
| `value` | A future's value; `by` identifies its writer |
| `drop` | A handle's end, attributed to the party that dropped it |

The `table` is the generated component's: `imports` or `exports`, not the guest's handle table.
The numeric `id` is local to that component: `c1, c2, …` for imports and `t1, t2, …` for exports.
Replay follows each task's or handle's sequence and the step constraints; unrelated lines need
not be consumed in file order. The trace has no identity for the guest task that caused an import
call.

For example, an HTTP handler can return its response before the host reads its body:

```text
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

Here `<--` denotes host-produced contents and `-->` host-consumed contents. The example's body
list is abbreviated.

## Labels

A label names one lifetime of a handle in a generated component's table. For example, handle 7
can first refer to stream A and later be reused for stream B. The trace must distinguish these
lifetimes even though the handle number is the same. We call each lifetime a **binding**.

Host resources use host handle numbers in record mode and mock numbers in other modes. Streams
and futures use the proxy's retained end: the end received, or the writer of a pair it creates.
Neither numbering scheme is the guest's numbering.

**Lifetime invariant.** Ownership is linear, but mentions are not: a binding may have many
borrows or stream transfers. It ends on a drop, a future's value, or an owned transfer out of
the table in a host call/return. Borrowing does not end it. Reusing a handle creates a new
binding, not a rename of the old one.

The recorder names successive bindings `#stream-7`, `#stream-7-2`, and so on. Distinguishing the
lifetimes is necessary for this naming scheme; the suffix spelling is a convention, not an
additional resource property. Fresh trace IDs would be another possible representation. The
current convention retains the raw handle number and assigns names from the observed prefix,
so truncating a trace does not change earlier names.

**Observation order.** Transferring a value and recording the transfer are distinct actions.
A pump may record a completed transfer only when its task resumes, after the guest has already
acted on a resource in that value. The trace can therefore contain a drop of `#body-2` before
the transfer line that first explains where `#body-2` came from. The resource existed before
the drop; only the line describing its arrival is late.

This is not the recorder sorting same-step events into a different order: it appends observations
as they reach it. Guest drops have no `at`, so this ordering does not even require two lines with
the same clock value. The recorder associates such an early drop with the subsequently recorded
arrival, retaining one label for that lifetime rather than treating the arrival as handle reuse.

**Renaming invariant.** Correspondence between recorded and replay bindings is a table-scoped
partial bijection: two distinct live bindings cannot map to the same replay binding. Every
owned or borrowed occurrence must respect the established correspondence and retain its borrow
annotation. Replay may reuse a raw handle after the recorded binding it represented has ended.
Streams/futures created by replay are paired explicitly when needed.

The implementation is [`crates/trace/src/renaming.rs`](../crates/trace/src/renaming.rs).
This is correspondence of table bindings, not recovery of underlying object identity: an end
handed back by the guest receives a new handle without revealing that it was the proxy's own.

## Recording

### Calls and cancellation

An async import is polled once in the calling task. If ready, its result is returned immediately
(RETURNED). Otherwise the proxy records STARTED and moves the pending future to an executor job;
its completion is recorded with that run's step. wit-bindgen moves the call's waitable to the
executor task when that task polls it. The first-poll path preserves immediate completion for
the runtime's native host functions, not necessarily for hosts implemented as components.

The trace distinguishes a normal `return` from cancellation of a pending guest import. On the
normal path, the proxy disarms its cancellation guard and records the result. If the pending
wrapper is dropped instead, its guard records `cancel` followed by `cancelled`; wit-bindgen
acknowledges cancellation with `task.cancel`. Dropping the wrapper also drops its pending host
call, cancelling that work.

Thus `cancelled` records the proxy's acknowledgement, not an inspection of whether the underlying
host callee acknowledged or returned after the cancellation request. Host cancellation of guest
exports is a separate, unimplemented recording path.

### Stream and future transfers

The proxy keeps the end it receives, creates a pair, and hands the pair's readable end onward.
A pump copies between them, converting payloads and recording transfers and drops.

**Copy invariant.** A stream line describes one completed rendezvous, not an attempted read or
write. Record mode does not read ahead: it waits for the guest side to be ready before issuing
the matching host operation. Zero-length reads/writes provide this readiness handshake on the
guest side; native host ends may answer zero-length operations immediately. A stream nobody
reads has no recorded host write. Recording a completed transfer does not imply its line precedes
every action on its payload; see [Observation order](#labels).

Futures have no equivalent reader-readiness handshake. A host-produced value is recorded when
written; the guest may instead drop its reader, which is not recorded. A guest-produced value
is recorded when the host takes it. The proxy does not rely on the default value wit-bindgen
writes when a future writer is dropped unwritten.

Instrumentation changes the reachable schedules: pumps make guest copies block, and the extra
task can delay host results. Read-ahead would admit some immediate copies but would also read
earlier or more than requested. The goal is replay of the instrumented execution, not preservation
of every schedule of the uninstrumented component.

## Steps

The current executor polls its existing jobs once, then polls newly queued jobs until no more
are queued. All share its task and waker. A blocked operation joins that task's waitable set;
sync submissions wake it through wit-bindgen's inter-task wakeup stream.

**Clock invariant.** Starting from zero:

```text
step = executor runs + explicitly counted external steps
```

Export arrivals take external steps. Probes and polls blocked by a handoff do not count as runs.
A host event delivered outside an immediate guest-call answer carries `at`, such as an export
arrival, a STARTED call's return, a copy, or a host drop. Guest lines and immediate answers have
no `at`: their call or operation sequence places them. An event's `at` need not itself advance
the clock; several events can share a step.

Record mode starts the executor while async exports are outstanding and requests draining when
the last returns. Another export can keep or reopen the running driver; it finishes once its
jobs finish. Replay/fuzz `start` drives it for the run; dialog drives and drains it per export.
Sync exports alone cannot keep the executor running.

### Completion handoff

**Handoff invariant.** A wait completed in an executor run holds admission of the next run until
its waiting task resumes or is dropped. Other jobs in the current run still get polled:

```text
complete(waiter, step) < resume_or_drop(waiter) < next executor run
```

This lets a STARTED call return, or an export's arrival take its step, without another run
overtaking that continuation. It is not a promise that the guest has consumed every delivered
event or finished all computation caused by it.

### Equal event-delivery boundaries

**Boundary invariant.** A self-woken run must get the same opportunity to receive a pending
event before polling jobs as an event-woken run. Otherwise replay, which advances itself toward
recorded steps, would observe a completion one run later than recording.

wit-bindgen delivers a callback's event before polling tasks. After a yield, however, the callback
has no event: it polls tasks first, then polls the waitable set if the task self-woke. The executor
therefore wraps the generated callback: the first poll after a yield is a **probe**, which only
self-wakes. wit-bindgen can then deliver an event before the real run. If no event is found and
it returns Yield, the wrapper re-enters once without another probe.

wasmtime prioritizes event-woken tasks over yielded tasks, allowing guest work to run before
the executor's next self-woken run. Neither this priority rule nor the probe controls which of
several guest events is selected. The callback adapter and the priority behavior are runtime
dependencies, not consequences of the clock equation.

## Replay

The imports proxy answers calls and performs transfers from the trace; the exports proxy's
`start` drives export calls and host drops of guest resources. Immediate answers are returned
at their call boundary. Deferred executor work waits for its recorded step, self-waking while
below it and reporting divergence if it has passed it. External step-taking continuations use
the handoff to keep their placement adjacent to the run they awaited.

Guest lines are checked against the corresponding task/handle sequence up to label renaming.
Mismatches report expected and actual events; `finish` checks for unconsumed lines, except host
drops of its own resources, which require no replay action. These checks detect differences
visible to the trace, not every possible internal guest difference. A stall may surface as a
runtime deadlock. Cut traces use a weaker policy, described below.

## Correctness

Distinguish three claims: recording observes the instrumented execution; replay reproduces that
execution's guest-visible interactions; instrumentation covers the original component's possible
schedules. The design aims at the first two, subject to the gaps below, not the third.

### Conditional replay argument

Record and replay cannot have literally identical runtime states: host subtasks, proxy objects,
and recorder activity differ. Let `R` relate states with corresponding guest computation,
pending guest-visible operations, consumed per-operation trace prefixes, and logical steps,
with resource bindings related by the label bijection.

The proof obligations are:

1. **Complete observations:** all relevant external inputs and readiness outcomes are recorded;
   there are no unobserved suspensions that affect execution.
2. **Matching progress:** corresponding jobs/calls can perform each recorded operation at its
   prescribed boundary, preserving RETURNED/STARTED and stream/future transfer semantics.
3. **Corresponding runtime choices:** scheduling and event selection preserve `R`, despite
   private allocation differences. Determinism on each runtime's full state alone is insufficient.
4. **Guest determinism:** given corresponding inputs and scheduling observations, guest execution
   preserves the relation. Unrecorded sources of guest nondeterminism are outside the claim.

Under these hypotheses, use induction over observation boundaries, grouping executor work by
step. Initially the guests and empty trace prefixes correspond. Immediate calls receive matching
answers; an executor run performs matching recorded effects; label correspondence extends or
ends consistently. The boundary and handoff invariants prevent these effects moving across the
specified continuations. Corresponding runtime choices then let guest execution reach related
states at the next boundary. Thus each observed prefix, and complete termination when reached,
agrees up to renaming. Eventual progress additionally requires terminating polls, fair scheduling,
and completion or cancellation of outstanding work.

This is a conditional proof argument, **not a proof that the implementation satisfies all four
hypotheses**. Equal step numbers are necessary placement checks, not sufficient evidence for `R`.
The separately scheduled exports jobs and the following counterexample are important gaps.

### Answers in the same step

Two results can become pending before one guest wait. wasmtime selects the lowest waitable by
kind and then by a store-wide index, not by the guest's own handle. Proxy and recorder allocations
differ between modes, so recording can select A while replay selects B even when both steps and
values agree. A guest using `select` can then diverge. The proxy does not see which event the
guest selected, so the current trace cannot repair this choice. The race fixture has exhibited
this divergence; at most one pending relevant event avoids this particular ambiguity but does
not establish the other proof obligations.

Restricting releases to one guest-visible event per run is a possible scheduling restriction,
not a complete implemented fix: one future poll may perform multiple effects, and the guest
may observe accumulated events or drain/poll its set. A release policy would need to control
actual observations and would exclude some recordings. Runtime ordering or observation capture
offers a less restrictive path; see [Portability](#portability).

### Verification

Existing checks include source snapshots, record/replay under `wasmtime run` and `serve`, and
replaying fuzzed traces. The [race spike](../tests/fixtures/spike-p3-race) exercises clock and
completion ordering; `make test-record-p3`, `make test-record-http-p3`, and `make test-fuzz-replay`
cover representative calls, streams, and cancellations. Passing runs are evidence, not a general
proof; in particular, trace checking cannot certify an observation it never records.

The [executor verification plan](executor-protocol.md#7-verification-plan) separates a transition
model, Rust implementation conformance and memory safety, runtime-adapter tests, and end-to-end
replay. Exhaustive small schedules and a record-during-replay fixpoint test would strengthen the
evidence. Models must retain nondeterministic event selection to expose, not hide, the known gap.

## Limitations

| Boundary | Current limitation and possible direction |
|---|---|
| Exports proxy jobs | Pumps run on the task driving the export, not the imports executor. Their events use the last step and replay waits until that count, rather than giving them an exact executor run. Moving typed work into the executor could unify placement; independent counters would not. |
| Returning stream/future ends | The proxy cannot recognize its own end returned through the guest. Copying it against its writer in the same instance traps for non-numeric payloads. A separate pair-producing instance or a runtime identity facility could avoid this, with additional scheduling work. |
| Host cancellation of exports | Not recorded. wit-bindgen hides the guest's return-versus-acknowledgement choice; replay would also need to issue cancellation at its recorded point. |
| Hidden suspension | Blocking WASI 0.2 calls and yielding epoch deadlines can suspend where the trace has no placement information. A trapping deadline only ends execution. |
| Bundled recorder | It prints through blocking WASI 0.2 calls, violating the no-hidden-suspension assumption in principle. No divergence has been attributed to this yet. A suitable host recorder or executor-driven async output could remove the risk; buffering until exit loses crash traces and does not suit long-lived servers. |
| Component-based hosts | Calling into another component can suspend even when the proxy's first poll sees a result, so the guest can see STARTED instead of RETURNED. This additional suspension is not reproduced. |
| Guest wait/poll behavior | General guests can drain or poll waitable sets rather than use wit-bindgen's callback pattern. Replaying those readiness observations requires more than the current event lines. |
| Sync exports and CLI hosting | Record mode needs an async export to drive queued work. `proxy-component run` does not yet provide p3 WASI/HTTP hosting. |

### Traces cut short

An unanswered export marks a trace as cut, for example after a trap or termination during that
call. Drops and guest-written future values observed only after the guest progresses may be
absent. Replay accepts otherwise missing guest stream/future lines in this case, aiming to reach
the same trap; it does not prove the missing suffix matched.

A trap after all exports returned can look like a complete trace and instead produce a missing
line divergence. An explicit run-end record would distinguish these cases. `wasmtime serve`
reuses instances across requests, so a trace may contain many export calls without a normal exit.

Other format limitations are verbosity of `list<u8>` WAVE values and the absence of causal caller
identity, which limits deriving alternative schedules for systematic exploration.

## Portability

A deterministic scheduler chooses predictably from its state and ready events. Replay must also
make the corresponding events ready at the corresponding observation points. Stable tie-breaking
cannot help if recording has only B ready but replay eagerly makes both A and B ready.

Three approaches provide different amounts of support:

- **Deterministic scheduling profile:** specify guest-relative event ordering, relevant task
  ordering, and delivery boundaries independently of private proxy allocations. It can resolve
  runtime-choice ambiguity; a readiness-placement mechanism such as executor steps still remains.
- **Guest observation shim:** instrument async built-ins to record wait/poll results and control
  their replay. It works without runtime adoption but rewrites the guest and needs sufficient
  coverage of readiness and scheduling observations.
- **Runtime-assisted recording/replay:** capture external readiness and relevant choices at actual
  guest boundaries, then inject them during replay. This could remove proxy steps, handoff barriers,
  and callback probes; ordinary async work, conversion, and possibly pumps would remain.

The current design requires neither runtime patches nor guest rewriting. Using the same runtime
version/configuration is its baseline, not a guarantee: the store-index counterexample applies
even there. Cross-runtime replay requires an agreed observation contract or recording the choices
that differ. See the [runtime alternative](executor-protocol.md#runtime-support-that-could-replace-the-scheduling-policy)
for the contract needed to replace custom scheduling.

## Status

Implemented: the trace/recorder interfaces, replay and fuzz generation, and recording of async
imports, stream/future transfers, and guest import cancellation. Tests exercise command and HTTP
hosting, including reused server instances. General exact replay and cross-runtime portability
remain conditional on the unresolved observation and scheduling issues above. The executor
protocol refactor is a proposal; its finite-batch scheduling rule is not current behavior.
