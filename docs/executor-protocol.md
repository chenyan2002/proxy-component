# Proposed executor protocol

Status: design, not the implemented scheduler and not yet verified. The existing executor is
in `assets/executor.rs`; its record/replay requirements are in
[`p3-record-replay.md`](p3-record-replay.md#steps).

The objective is to control the order in which executor-managed asynchronous operations make
progress, with an explicit clock and cancellation contract. It is not to make the component
model runtime deterministic. Labels play no role in scheduling.

## Why an executor, and could the runtime replace it?

### What changes from p2

For the synchronous p2 interactions this tool handles, a call boundary supplies a natural
observation point. The proxy records the arguments, calls the host, records the result, and
returns. Replay substitutes the recorded result at the same boundary. Even `poll` exposes its
readiness decision as the result of an intercepted call. This does not mean p2 hosts have no
concurrency; it means the relevant observation is exposed through a synchronous interface.

In p3 an interaction can outlive the call that introduced it:

- An async import can return immediately or remain pending while other work runs.
- A sync function can return a stream whose contents will be produced much later.
- Stream and future operations are canonical built-ins, not import calls the proxy can wrap.
  To virtualize their contents, the proxy supplies the other end, using pumps in record mode
  and generated producers/consumers in replay, fuzz, and dialog modes.
- Several calls, copies, cancellations, and export invocations can overlap, with their order
  observable to the guest.

The proxy therefore needs both **a home for work that outlives a call** and **a way to reproduce
when that work becomes observable**. These are different requirements. Only the second calls
for a record/replay scheduling policy.

### What the current executor buys

The executor is a user-space driver of Rust futures inside a component-model task, not a
replacement for the runtime scheduler. The runtime still delivers events and chooses which
component tasks run; wit-bindgen schedules futures inside its tasks.

The custom executor supplies:

1. A long-lived async task to drive jobs submitted by sync imports, which cannot themselves
   await the work or use this tool's async task-spawning path.
2. A common logical clock and admission policy for deferred import results and pumps. A trace
   can say "perform this in round n" rather than merely "this happened after that line".
3. A completion barrier so a new round cannot overtake a task receiving the previous round's
   result, plus lifecycle management for the driver.

If jobs were independently spawned and merely awaited their recorded payloads, the runtime
would still schedule them, but the trace would no longer say when each should become ready.
Making replay's results ready immediately can change a STARTED call into a RETURNED call,
change which branch wins a race, or let a stream advance earlier. Preserving trace line order
alone does not specify which of the guest's waits observes a completion.

Thus **a custom scheduler is not intrinsic to p3**. It is the current solution to recording
readiness placement without runtime instrumentation. Ordinary p3 applications can rely on
their runtime and SDK; exact replay introduces an additional requirement.

### Why an ordinary deterministic scheduler is insufficient

A scheduler can be a deterministic function of its full state and ready inputs:

```text
next = schedule(runtime_state, ready_events)
```

That is weaker than replay determinism. Record and replay can have different ready-event
sets because the real host completes at different times from a trace reader. They can also
have different internal runtime states because their proxy jobs, host subtasks, recorder calls,
and handle allocations differ. A deterministic tie-break by a store-wide handle index can
therefore make a different choice, as the existing same-step race demonstrates.

For example, if the guest races calls A and B, a runtime that always chooses the lowest ready
handle is deterministic. Recording can observe only B ready and choose B; replay can make both
ready and choose A. Stable tie-breaking cannot repair the difference in readiness placement.

The required property is correspondence of **guest-observable histories**, not identical
internal schedules. Scheduler decisions must either respect the correspondence between the
recording and replay states, despite hidden bookkeeping, or the decisions must themselves be
recorded and replayed. Alpha-renaming trace labels does not change the runtime's handle indexes.

### Runtime support that could replace the scheduling policy

A runtime-backed design could record and replay at the guest's actual async observation
boundaries instead of introducing executor rounds. Its contract would need:

- **External event capture and controlled replay delivery:** results, stream transfers,
  export arrivals, and cancellation, associated with reproducible logical operations and
  observation points rather than wall-clock times or proxy-local rounds.
- **Readiness semantics:** preserve immediate versus pending calls, backpressure, and the
  observations of waits and polls, including multiple pending events. This may require
  readiness-state information beyond the single event selected by a wait.
- **Scheduling correspondence:** specify relevant event/task ordering independently of hidden
  proxy allocations, or record and replay those choices. Determinism of Rust/SDK task scheduling
  is part of the argument too; fixing only component-runtime task order is not sufficient.
- **Task lifetime support:** provide or retain a driver that can own work originating in sync
  functions and outliving them, with defined cancellation and shutdown behavior.

With that contract the generated proxy could largely return to forwarding calls and running
pumps as ordinary async work. Custom step waits, the completion barrier, and the callback probe
could move out of generated code, because the runtime would enforce observation order. Payload
conversion, resource virtualization, and any still-needed pumps would remain. If the runtime
also intercepts those operations, more of the proxy could disappear; that is a different
instrumentation architecture, not just a different scheduler.

A smaller deterministic scheduling profile can help without providing full runtime replay. For
example, a guest-relative event ordering rule can address the current store-index tie-break
problem while the proxy still controls readiness using rounds. It does not by itself remove
the need to record when external events become visible.

Another option is to deliver external input only when the guest is quiescent. With suitable
ordering rules this can simplify deterministic execution, but it restricts the recorded
schedules and must still account for immediate calls and external input order. It is not a
transparent replacement for recording arbitrary host timing, and the proxy cannot generally
observe guest quiescence on its own.

### Architectural choice

| Available support | What remains in the proxy |
|---|---|
| Current runtime and current trace | Work driver, round scheduling, handoffs, callback adaptation, pumps |
| Deterministic ordering profile only | Work driver and readiness-placement mechanism; some ordering assumptions become simpler |
| Runtime observation capture and replay delivery | Ordinary async work and conversion; custom round scheduling may be unnecessary |

The executor protocol below is for the first case. It should not be mistaken for a requirement
of p3 or the ideal architecture if runtime changes are allowed. A useful runtime experiment is
a two-call race, run with perturbed proxy allocations and immediate/pending completion patterns:
does the proposed runtime contract reproduce the same guest observations without proxy steps?
That tests the needed contract more directly than replacing one deterministic ready queue with
another.

## 1. Scope and assumptions

There are three participants:

- **Scheduler:** owns jobs and admits finite rounds of polling.
- **Waiters:** tasks outside the scheduler that receive results of its jobs.
- **Runtime adapter:** translates notifications and wit-bindgen callbacks into opportunities
  to run, with event delivery ordered before the corresponding round.

A job is a future. A poll can perform several observable operations, not just complete one
operation. Immediately ready imports retain their first-poll fast path outside the scheduler.
The exports component's separately scheduled pumps are outside this protocol for now.

The model assumes single-threaded cooperative execution, no recursive entry into a scheduler
round, and that each individual poll returns. A job poll is one transition in this model;
notifications and submissions it generates are included in that transition. This abstraction
must be checked against runtime behavior, not assumed to follow from Rust's type system.

## 2. State

At a round boundary the scheduler has:

| State | Meaning |
|---|---|
| `mode` | `Idle`, `Active`, `Draining`, or `Failed` |
| `step` | Number of admitted rounds plus externally counted steps since initialization |
| `jobs` | Ordered sequence of unfinished jobs retained from earlier rounds |
| `submitted` | Ordered sequence of jobs not yet admitted to a round |
| `requested` | A request for another round is outstanding |
| `boundary` | `Required` or `Established`, supplied by the runtime adapter |
| `handoffs` | Set of completed external waits not yet acknowledged or cancelled |

During a round there are also `batch`, the unpolled jobs of that round, and `survivors`, the
jobs already polled and still pending. `jobs` is temporarily moved into the round, not copied.
Job and waiter identifiers in the model are fresh identities, not component handle numbers.
Counters in an implementation must fail on overflow rather than wrap.

Each external wait has one completion cell:

```text
Pending(job) -> Ready(value, completion_step, token) -> Taken
     |                         |
     +------ cancellation -----+--------------------> Cancelled
```

The scheduler has exclusive polling access to a pending job; the waiter has cancellation and
result-consumption authority. The cell is shared bookkeeping, not a second independently
owned future. An ordinary detached job has no completion cell or handoff.

## 3. Scheduler transitions

Transitions below are atomic at the model's granularity. Invoking a future, destructor, or
waker while holding a mutable borrow of global scheduler state is not an implementation of
an atomic transition: these calls must happen outside that borrow.

### Notifications and submission

`RequestRound` sets `requested = true` and arranges for the driver to be polled if present.
Repeated requests before admission coalesce. A notification is not itself a clock step.

`Submit(job)` appends a fresh job to `submitted`, then requests a round. Submission while
idle is allowed: a sync import can create work before an async export starts the driver.
Submission while draining is allowed: existing jobs can create more work.

Operation readiness and a job's request to reach a later step request rounds. A raw driver
poll and a probe-only reschedule do not. The adapter must preserve this distinction.

### Admission

`BeginRound` requires:

```text
mode in {Active, Draining}
no round currently executing
requested
boundary = Established
handoffs = empty
```

It performs:

```text
step := step + 1
requested := false
boundary := Required
batch := jobs ++ submitted
jobs := []
submitted := []
survivors := []
```

An empty round is legal when explicitly requested, for example to advance the replay clock.
There is no automatic clock jump: intervening rounds may enable external continuations.

### Polling

`PollNext` removes the first job from `batch` and polls it exactly once:

- Pending: append it to `survivors`.
- Ready, detached job: retire it.
- Ready, external wait: remove its future from the completion cell, store its value and the
  current step, create one token in `handoffs`, and notify its waiter.
- Already cancelled: retire its scheduling entry without polling its future.

Submissions during a poll go to `submitted`, never to the current `batch`. A completion does
not interrupt the current round; its token prevents admission of the next round.

`EndRound` requires an empty `batch` and sets `jobs := survivors`. Nonempty `submitted` leaves
a round requested. Pending jobs alone do not request rounds: they need readiness or an
explicit clock-advance request. Requests made during the round are not cleared at its end.

Thus, excluding cancelled entries:

```text
next round's batch = pending(current batch) ++ submissions before next admission
```

This finite-batch rule differs from today's executor, which polls newly spawned jobs in the
same round until no more are queued. It is a separately testable scheduling change, not just
a representation change. Compatibility with existing traces is not established.

### Handoff and cancellation

`TakeResult(waiter)` requires `Ready`. It consumes the value and token exactly once and marks
the cell `Taken`. The token is removed from `handoffs`.

`CancelWaiter(waiter)` has two cases:

- Pending: destroy the future before its borrowed inputs can become invalid, mark the cell
  `Cancelled`, and request cleanup of its scheduling entry.
- Ready: discard the value, consume the token, and mark the cell `Cancelled`.

Releasing the last token re-notifies the driver if a round is requested or draining needs a
completion check. It does not invent a round request. A notification received while held must
remain effective after release. Lifecycle checks, like probe polls, can wake the driver without
requesting a round.

The acknowledgement is the waiting task taking its result, not the guest finishing all work
caused by that result. Cooperative execution lets that task continue until its next suspension;
the protocol does not hold the executor across an arbitrary later await.

### External steps

`ExternalStep` increments `step` without polling jobs. It is allowed only outside an executing
round and outside `Failed`. Outstanding handoffs do not forbid it: an acknowledged
continuation may need to count an export's arrival while another continuation is still held.
Idle external steps must also be supported if the caller counts an arrival before starting
the driver; this does not by itself start the driver or poll jobs.

`step_after(target)` waits until `step >= target`, then performs `ExternalStep` without an
intervening suspension. Its result is the then-current step plus one, not necessarily
`target + 1`. A replay caller requiring an exact step must check it.

An exact-step job (`reach_step`) requests another round when below its target, succeeds at
its target, and reports divergence above it. Relative waits must define whether the first
poll counts: the current `wait_for_runs(k)` completes on its `(k + 1)`th job poll. Changing that
convention is another scheduling change and must not slip into this refactor unnoticed.

### Lifecycle

- `Start` in `Idle`: create the sole driver, enter `Active`, and request its initial round.
- `Start` in `Active`: reuse the driver; do not create a second one.
- `Start` in `Draining`: reuse the driver and return to `Active`.
- `Stop` in `Active`: enter `Draining` and notify the driver to check completion.
- `Stop` in `Idle` or `Draining`: no additional effect.
- At a round boundary, `Draining` with no jobs, submissions, or handoffs becomes `Idle`.
  This clears `requested` and resets the adapter boundary. The check does not require a new
  clock step. The clock survives restart.
- Unexpected driver cancellation or failure enters `Failed`; restarting a partially lost
  execution is not allowed. Component teardown and destruction of remaining jobs require an
  explicit implementation path, not an accidental consequence of dropping a local vector.

Waiter cancellation is normal protocol behavior. Driver failure is not normal waiter
cancellation. Liveness claims below exclude failed executions.

## 4. Runtime adapter protocol

`boundary = Established` means an event-delivery opportunity has occurred before admission.
It does **not** mean that the waitable set is empty or that every pending event was delivered.

At a callback entered with an operation event, wit-bindgen delivers the event before polling
its tasks. At a callback entered after a yield (`EVENT_NONE`), it polls tasks first; only if
they return pending and self-wake does it poll the waitable set. This asymmetry is why merely
using the same future in record and replay is insufficient.

The adapter protocol is:

```text
event-bearing entry:
    deliver event -> establish boundary -> attempt scheduler progress

yield-resume entry:
    request probe -> probe poll (no jobs, no step; self-wake)
      -> runtime delivers a pending event -> establish boundary -> attempt progress
      OR
      -> runtime finds no event and returns Yield
         -> one direct callback re-entry -> establish boundary -> attempt progress
```

The direct re-entry must not request another probe, or an empty waitable set would livelock.
The initial driver poll may use an initial boundary; there was no previous executor yield to
normalize. Each new admitted round consumes its boundary. Subsequent delivery opportunities
inside the same runtime callback can establish subsequent boundaries.

When held, a probe can still run, but no round is admitted. Probe notifications are bookkeeping,
not operation readiness. Cancellation callbacks follow the runtime's cancellation path, not
the event-bearing readiness rule.

The current two flags, `PROBE` and `PROBED`, implement a version of this handshake. The new
adapter needs explicit phases and tests against the pinned wit-bindgen implementation. In
particular, an event may wake a different task without polling the driver immediately;
callback entry, task polling, and round admission are not interchangeable transitions.

## 5. Safety properties

The following must hold after every transition:

1. **Clock accounting:** `step = admitted_rounds + external_steps`. Probes, notifications,
   held polls, and lifecycle checks do not advance it. During a round all jobs see its step.
2. **Single driver:** at most one driver and at most one executing round exist.
3. **Job conservation:** every submitted job has exactly one scheduling location (submitted,
   unpolled batch, survivor, retained jobs) until retired. Cancelled tombstones contain no
   pollable future. No job is polled twice in a round or after cancellation/completion.
4. **Handoff conservation:** `handoffs` corresponds one-to-one with `Ready` completion cells.
   Each token is created once and consumed once. No round begins with an outstanding token.
5. **No lost demand:** a request remains set until admission consumes it. Requests during a
   round or while held survive until a later admission, unless draining finishes or execution
   fails. Registering the driver's waker cannot lose a concurrent/reentrant notification.
6. **Boundary ordering:** every admitted round has a preceding adapter boundary; probes poll
   no jobs. Synthetic scheduling requests do not bypass event-delivery normalization.
7. **Finite batches:** admissions during a round do not enlarge its batch. With terminating
   polls, every round finishes even if jobs continually submit more jobs.

For an external wait completed in round `n`, these imply the useful happens-before relation:

```text
complete(waiter, n) < acknowledge_or_cancel(waiter) < begin(next round)
```

External steps may occur between these actions, so the next round's clock value need not be
`n + 1`. No ordering between different waiters' acknowledgements is promised.

## 6. Conditional liveness

Assume that the runtime eventually services persistent notifications, each poll returns,
and every ready waiter eventually resumes or is cancelled:

- A requested round in a live driver eventually runs once all handoffs are released and an
  adapter boundary is available, unless draining finishes first.
- A retained ready job is eventually polled; submissions cannot indefinitely precede it.
- Once all jobs terminate and all handoffs are released, a draining executor becomes idle.

An external operation that never completes, a guest that never services its stream, or a
waiter that is never resumed can prevent progress. No scheduler can prove unconditional
termination for such inputs. Safety properties do not require these fairness assumptions.

## 7. Verification plan

### A. Executable transition model

Write a small pure reference model with scripted jobs, no Rust futures, and the state above.
Enumerate enabled transitions for a bounded set of jobs, waiters, submissions, and events.
Check every safety invariant after each transition and report the shortest counterexample.
Include event delivery, empty probes, cancellation before/after completion, notifications
during polling and while held, stop/reopen, and driver failure.

Use a finite-state model checker such as TLC if checking fair-cycle liveness: state the
fairness constraints explicitly and distinguish deadlock from a permitted environment stall.
Bounded exhaustive exploration establishes properties only within its bounds. General safety
needs an induction argument: initial state satisfies the invariants and every transition
preserves them. An unbounded liveness proof additionally needs the stated fairness assumptions.

### B. Rust implementation conformance

Drive the real scheduler with manually polled futures and counting wakers. For each model
action, compare an audit log of `BeginRound`, `PollJob`, `Complete`, `Take`, `Cancel`,
`ExternalStep`, and lifecycle changes. Adapter-only activity is a stuttering transition:
it may change implementation bookkeeping but must not add semantic actions.

Generate action sequences and shrink failures. Dedicated tests should include two completions
in one round, cancellation of one while another remains held, a notification before waker
registration, self-spawning jobs, empty requested rounds, and reopening while draining.

Run cancellation/drop tests under Miri where supported. This does not prove the existing
borrowed-future lifetime erasure sound. A safety argument relying only on `Drop` is incomplete
if forgetting a waiter can leave a pollable future borrowing expired inputs. Prefer owned jobs
where possible; retaining lifetime erasure requires a separate safety audit, including leaked
waiters and teardown. Do not treat the protocol proof as a Rust memory-safety proof.

### C. Runtime adapter conformance

Use a small Wasm fixture to record callback entries, probes, event deliveries where observable,
job polls, and steps. Exercise event-bearing resumes, self-woken resumes with/without a pending
event, inter-task wakeups, multiple queued events, and cancellation. Verify the boundary rule
and absence of probe-only rounds. Repeat on wit-bindgen/runtime upgrades.

Native scheduler tests alone cannot establish this contract: they do not model wit-bindgen's
internal task queue, waitable-set polling, or callback return codes.

### D. End-to-end record/replay

Run the existing p3 fixtures with immediate and delayed imports, overlapping exports, stream
and future transfers, cancellation races, and repeated run/stop cycles. Add a nested-spawn
fixture specifically for the finite-batch change. Compare semantic events, steps, termination,
and traps, not just the final value. Regenerate source snapshots after intended code changes.

Test the representation refactor against old traces. Test the finite-batch scheduling change
separately with newly recorded traces, and explicitly decide whether to reject/version old
traces or retain the old admission policy. Structural simplification does not require changing
that policy if the scheduling change proves unsuitable.

## 8. What would be proved

Scheduler correctness means its executions satisfy the transition system's safety properties
and conditional liveness. Implementation correctness additionally requires refinement to that
model, a valid runtime adapter, and Rust memory safety.

Exact replay is a further claim. Informally, equality of the already visible histories must
imply equality of the next visible operation when the host leaf is replaced by a recorded one.
Equal step numbers alone do not prove this: record and replay must create corresponding work
in time for the same rounds, and the guest must observe corresponding runtime choices.

In particular, the documented multiple-events-at-one-guest-wait problem remains. The bounded
model should be able to exhibit two legal guest observation orders when that runtime choice is
left nondeterministic. That is a counterexample to an unconditional exact-replay claim, not a
reason to hide the choice behind a deterministic test double.

The acceptance claim is therefore limited: the scheduler implements this protocol, and replay
equivalence holds only under separately stated runtime and observation assumptions. Neither
the proposed finite-batch rule nor the label scheme removes that limitation.
