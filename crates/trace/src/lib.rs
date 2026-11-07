//! The trace of a run, which record mode writes and replay mode reads: the events the proxy
//! observed, as `docs/p3-record-replay.md` (Trace format) defines them. A trace is stored as one
//! JSON line per `Line`; other lines in the text, e.g. the guest's own output, are ignored.
//!
//! The types are generated from `proxy:recorder/types` (`assets/recorder.wit`), with serde, so
//! that the WIT defines them once: the bundled recorder maps the interface to this crate, with
//! wit-bindgen's `with` option, and a host converts its own bindings' types with `convert`, as
//! they serialize alike. Values are WAVE text, e.g. `ok(#response-3)`, whose type comes from
//! where they appear.
use std::fmt;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod renaming;
pub use renaming::Labels;
use renaming::Renaming;

mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "trace",
        additional_derives: [serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash],
    });
}
pub use bindings::proxy::recorder::types::*;

/// The lines the proxy observed, in the order it observed them.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(transparent)]
pub struct Trace(pub Vec<Line>);

/// Converts between this crate's types and another binding's types of `proxy:recorder/types`,
/// e.g. those of wasmtime's `bindgen!`, which derive serde with `additional_derives`.
pub fn convert<T: Serialize, U: DeserializeOwned>(value: T) -> U {
    serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}

impl Event {
    /// The stream, future or resource the event is about, for the events of a handle.
    pub fn handle(&self) -> Option<&Label> {
        match self {
            Event::Write(chunk) | Event::Read(chunk) => Some(&chunk.stream),
            Event::Value(value) => Some(&value.future),
            Event::Drop(label) => Some(label),
            _ => None,
        }
    }
    /// The task the event is about, for the events of a call.
    pub fn task(&self) -> Option<TaskId> {
        match self {
            Event::Call(Call { id, .. })
            | Event::Started(id)
            | Event::Return(ReturnValue { id, .. })
            | Event::Cancel(id)
            | Event::Cancelled(id) => Some(*id),
            _ => None,
        }
    }
}

impl Line {
    /// Whether the line is the host's answer to a guest call, which the host gives while handling
    /// the call: a `Started`, or the `Return` or `Cancelled` of a call that didn't start, of an
    /// import. Replay performs it as part of answering the call, rather than on its own.
    pub fn answers_guest(&self) -> bool {
        self.by == Party::Host
            && self.at.is_none()
            && matches!(
                self.event,
                Event::Started(_) | Event::Return(_) | Event::Cancelled(_)
            )
            && self
                .event
                .task()
                .is_some_and(|id| id.table == Table::Imports)
    }
    /// Whether the line is one the host performs from the exports component, which replay's
    /// `start` performs: an export call, or the host's drop of a resource of the guest. The
    /// host's drops of its own resources, in the imports table, the guest doesn't see; replay has
    /// nothing to do for them.
    pub fn is_start_line(&self) -> bool {
        self.by == Party::Host
            && match &self.event {
                Event::Call(_) => true,
                // Not a stream's or future's end, which the job that keeps it drops.
                Event::Drop(label) => {
                    label.table == Table::Exports
                        && !label.name.starts_with("#stream-")
                        && !label.name.starts_with("#future-")
                }
                _ => false,
            }
    }
    /// Whether the line is about `key`: a line of the task, or of the stream or future.
    pub fn is_about(&self, key: &LineKey) -> bool {
        match (key, &self.event) {
            (LineKey::Task(task), event) => event.task() == Some(*task),
            (LineKey::Handle(label), event) => event.handle() == Some(label),
        }
    }
    /// The exit status of a call to `wasi:cli/exit`, which ends the run.
    pub fn exit_status(&self) -> Option<i32> {
        match &self.event {
            Event::Call(Call { func, args, .. }) if func.starts_with("wasi:cli/exit") => {
                let is_err = args.first().is_some_and(|arg| arg.starts_with("err"));
                Some(if is_err { 1 } else { 0 })
            }
            _ => None,
        }
    }
    /// The stored form of the line, one line of text.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Trace {
    /// Reads a trace from its stored form, ignoring the lines that aren't trace lines, e.g. the
    /// guest's output: those that aren't JSON objects with a `by` and an `event`.
    ///
    /// Panics on a trace line that doesn't parse, e.g. of another version of the format, and on
    /// text without trace lines, e.g. a trace printed as text.
    pub fn parse(text: &str) -> Self {
        let lines = text.lines().filter_map(|text| {
            let value: serde_json::Value = serde_json::from_str(text).ok()?;
            let object = value.as_object()?;
            if !object.contains_key("by") || !object.contains_key("event") {
                return None;
            }
            match serde_json::from_value(value) {
                Ok(line) => Some(line),
                Err(err) => panic!("invalid trace line `{text}`: {err}"),
            }
        });
        let trace = Trace(lines.collect());
        assert!(
            !trace.0.is_empty() || text.trim().is_empty(),
            "no trace lines: was it printed with `PROXY_TRACE=text`?"
        );
        trace
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.table {
            Table::Exports => 't',
            Table::Imports => 'c',
        };
        write!(f, "{prefix}{}", self.id)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.table {
            Table::Imports => write!(f, "{}", self.name),
            Table::Exports => write!(f, "{} (exports)", self.name),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Call(Call {
                id,
                is_async,
                func,
                args,
            }) => {
                let is_async = if *is_async { "async " } else { "" };
                write!(f, "call {id} {is_async}{func}({})", args.join(", "))
            }
            Event::Started(id) => write!(f, "started {id}"),
            Event::Return(ReturnValue { id, value }) => {
                write!(f, "return {id} {}", value.as_deref().unwrap_or("()"))
            }
            Event::Cancel(id) => write!(f, "cancel {id}"),
            Event::Cancelled(id) => write!(f, "cancelled {id}"),
            Event::Write(Chunk { stream, items }) => write!(f, "write {stream} {items}"),
            Event::Read(Chunk { stream, items }) => write!(f, "read {stream} {items}"),
            Event::Value(FutureValue { future, value }) => write!(f, "value {future} {value}"),
            Event::Drop(handle) => write!(f, "drop {handle}"),
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let by = match self.by {
            Party::Host => "host",
            Party::Guest => "guest",
        };
        match self.at {
            Some(at) => write!(f, "{by} @{at}: {}", self.event),
            None => write!(f, "{by}: {}", self.event),
        }
    }
}

/// A trace being replayed: which of its lines replay has performed or checked, and how its
/// labels pair with replay's (see `renaming`). The lines `Replay` returns have replay's labels,
/// where it has paired them, and those it takes, the trace's.
pub struct Replay {
    lines: Vec<Line>,
    done: Vec<bool>,
    renaming: Renaming,
    /// Whether the trace is cut short (see `is_cut`).
    cut: bool,
}

impl Replay {
    pub fn new(trace: Trace) -> Self {
        let done = vec![false; trace.0.len()];
        let renaming = Renaming::new(&trace.0);
        let answered = |id: TaskId| {
            trace.0.iter().any(|line| {
                matches!(&line.event, Event::Return(ret) if ret.id == id)
                    || matches!(&line.event, Event::Cancelled(task) if *task == id)
            })
        };
        let cut = trace.0.iter().any(|line| match &line.event {
            Event::Call(call) => call.id.table == Table::Exports && !answered(call.id),
            _ => false,
        });
        Replay {
            lines: trace.0,
            done,
            renaming,
            cut,
        }
    }
    /// Whether the trace is cut short: an export call has no answer, e.g. the run trapped during
    /// it. The host records some copies once it sees them complete, after the guest went on, so
    /// a cut trace may lack the last of those: the guest's drops of streams and futures, and the
    /// values of the futures it wrote. `check` accepts a guest's line about one that it lacks.
    pub fn is_cut(&self) -> bool {
        self.cut
    }
    fn pending(&self) -> impl Iterator<Item = (usize, &Line)> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(idx, _)| !self.done[*idx])
    }
    /// The next line replay's `start` performs, in trace order: an export call, or the host's
    /// drop of a resource of the guest (see `Line::is_start_line`). `None` once there are none.
    /// The lines before it that are still pending may happen while `start` waits for its step;
    /// `finish` reports those that never do.
    pub fn next_host_line(&mut self) -> Option<Line> {
        self.take(|line| line.is_start_line())
    }
    /// The next host line about `key`, in replay's labels, e.g. the `Return` of a STARTED call,
    /// which the import's mock waits for, or a copy of a stream, which a job performs.
    pub fn take_host_line(&mut self, key: &LineKey) -> Option<Line> {
        let key = match key {
            LineKey::Handle(label) => LineKey::Handle(self.renaming.to_trace(label)?),
            key => key.clone(),
        };
        self.take(|line| line.by == Party::Host && line.is_about(&key))
    }
    fn take(&mut self, matches: impl Fn(&Line) -> bool) -> Option<Line> {
        let idx = self.pending().find(|(_, line)| matches(line))?.0;
        Some(self.hand(idx))
    }
    /// Hands line `idx` to replay, in its labels.
    fn hand(&mut self, idx: usize) -> Line {
        self.done[idx] = true;
        self.renaming.to_replay(&self.lines[idx])
    }
    /// Pairs the trace's label of a stream or future replay makes, from the value of the host
    /// line that hands it on, with its own, of the end it keeps, e.g. the writer of a stream a
    /// mock returns: the trace's label first appears in a host line, which `check` doesn't pair.
    ///
    /// Panics if either is paired with another, or no line hands it on.
    pub fn pair(&mut self, trace: &Label, replay: &Label) {
        assert!(
            self.renaming.pair_next(trace, replay),
            "replay's {replay} can't be the one the trace's {trace} hands on"
        );
    }
    /// Checks that replay reached every line, once it is done: all but the host's drops of its
    /// own resources, which replay has nothing to do for.
    ///
    /// Panics on the first line it didn't reach.
    pub fn finish(&self) {
        let missed = self.pending().find(|(_, line)| {
            line.by == Party::Guest || line.is_start_line() || !matches!(line.event, Event::Drop(_))
        });
        if let Some((_, line)) = missed {
            panic!("the trace has `{line}`, which replay didn't reach");
        }
    }
    /// Checks a guest line against the trace, up to the pairing of labels, and returns the host
    /// lines that answer it at once, e.g. the `Return` of an import call. A guest line is matched
    /// by its task, for the lines of a call, and by its handle, for those of a stream, a future
    /// or a resource.
    ///
    /// Panics on a mismatch, with the expected and the actual line.
    pub fn check(&mut self, actual: &Line) -> Vec<Line> {
        assert_eq!(actual.by, Party::Guest, "`{actual}` isn't a guest line");
        let (task, handle) = (actual.event.task(), actual.event.handle());
        // The trace's label of the handle, if replay's is paired, and otherwise a binding of
        // replay's number, e.g. a mock's, which carries the trace's. A pair whose trace label
        // ended, in a line replay has done, is of another handle, which had the number before.
        let paired = handle.and_then(|label| {
            let trace = self.renaming.to_trace(label)?;
            let live = self.renaming.end_of(&trace).is_none_or(|end| !self.done[end]);
            live.then_some(trace)
        });
        let found = self.pending().find(|(_, line)| match (task, handle) {
            (Some(task), _) => line.event.task() == Some(task),
            (None, Some(replay)) => {
                line.by == Party::Guest
                    && line.event.handle().is_some_and(|trace| match &paired {
                        Some(paired) => trace == paired,
                        None => Renaming::binds(trace, replay),
                    })
            }
            (None, None) => line.by == Party::Guest,
        });
        let Some((idx, expected)) = found else {
            let late = |label: &Label| {
                ["#stream-", "#future-"].iter().any(|kind| label.name.starts_with(kind))
            };
            if self.cut && handle.is_some_and(late) {
                return Vec::new();
            }
            panic!("the guest did `{actual}`, which the trace doesn't have");
        };
        let expected = expected.clone();
        if expected.by != Party::Guest || !self.renaming.matches(idx, &expected, actual) {
            panic!("the trace has `{expected}`, but the guest did `{actual}`");
        }
        self.done[idx] = true;
        let Some(task) = task else {
            return Vec::new();
        };
        // The task's next lines, while they answer the call.
        let mut answers = Vec::new();
        for idx in idx + 1..self.lines.len() {
            let line = &self.lines[idx];
            if self.done[idx] || line.event.task() != Some(task) {
                continue;
            }
            if !line.answers_guest() {
                break;
            }
            let started = matches!(line.event, Event::Started(_));
            answers.push(self.hand(idx));
            if !started {
                break;
            }
        }
        answers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(table: Table, id: u32) -> TaskId {
        TaskId { table, id }
    }
    fn line(by: Party, event: Event) -> Line {
        Line {
            by,
            at: None,
            event,
        }
    }
    fn call(id: TaskId, func: &str, args: &[&str]) -> Event {
        Event::Call(Call {
            id,
            is_async: false,
            func: func.to_string(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
        })
    }
    fn ret(id: TaskId, value: Option<&str>) -> Event {
        Event::Return(ReturnValue {
            id,
            value: value.map(str::to_string),
        })
    }
    /// The trace a recorder writes of `lines`, with their labels' bindings named apart.
    fn recorded(lines: &[Line]) -> Trace {
        let mut labels = Labels::default();
        Trace(lines.iter().map(|line| labels.label(line.clone())).collect())
    }

    /// An export call `t1`, during which the guest calls an import `c1`, then the host drops a
    /// resource of the guest.
    fn p2_trace() -> Trace {
        let (t1, c1) = (task(Table::Exports, 1), task(Table::Imports, 1));
        Trace(vec![
            line(Party::Host, call(t1, "run", &["#request-1"])),
            line(Party::Guest, call(c1, "get", &["1"])),
            line(Party::Host, ret(c1, Some("\"a\""))),
            line(Party::Guest, ret(t1, None)),
            line(
                Party::Host,
                Event::Drop(Label {
                    table: Table::Exports,
                    name: "#session-1".to_string(),
                }),
            ),
        ])
    }

    #[test]
    fn stored_form() {
        let trace = p2_trace();
        let text: Vec<_> = trace.0.iter().map(Line::to_json).collect();
        assert_eq!(
            text[0],
            r##"{"by":"Host","at":null,"event":{"Call":{"id":{"table":"Exports","id":1},"is_async":false,"func":"run","args":["#request-1"]}}}"##
        );
        let text = format!("guest output\n{}\n", text.join("\n"));
        assert_eq!(Trace::parse(&text), trace);
        let step = Line {
            at: Some(4),
            ..trace.0[2].clone()
        };
        assert_eq!(Trace::parse(&step.to_json()).0, vec![step]);
    }

    #[test]
    #[should_panic(expected = "invalid trace line")]
    fn stored_form_mismatch() {
        Trace::parse(r##"{"by":"Guest","at":null,"event":{"Drop":{"handle":"#x-1"}}}"##);
    }

    #[test]
    #[should_panic(expected = "no trace lines")]
    fn stored_form_as_text() {
        let text: Vec<_> = p2_trace().0.iter().map(Line::to_string).collect();
        Trace::parse(&text.join("\n"));
    }

    #[test]
    fn display() {
        let lines: Vec<_> = p2_trace().0.iter().map(Line::to_string).collect();
        assert_eq!(
            lines,
            [
                "host: call t1 run(#request-1)",
                "guest: call c1 get(1)",
                "host: return c1 \"a\"",
                "guest: return t1 ()",
                "host: drop #session-1 (exports)",
            ]
        );
    }

    #[test]
    fn replay() {
        let trace = p2_trace();
        let mut replay = Replay::new(trace.clone());
        assert_eq!(replay.next_host_line(), Some(trace.0[0].clone()));
        assert_eq!(replay.check(&trace.0[1]), vec![trace.0[2].clone()]);
        assert_eq!(replay.check(&trace.0[3]), vec![]);
        assert_eq!(replay.next_host_line(), Some(trace.0[4].clone()));
        assert_eq!(replay.next_host_line(), None);
    }

    #[test]
    #[should_panic(
        expected = "the trace has `guest: call c1 get(1)`, but the guest did `guest: call c1 get(2)`"
    )]
    fn replay_mismatch() {
        let mut replay = Replay::new(p2_trace());
        replay.next_host_line();
        replay.check(&line(
            Party::Guest,
            call(task(Table::Imports, 1), "get", &["2"]),
        ));
    }

    #[test]
    #[should_panic(expected = "the trace has `guest: call c1 get(1)`, which replay didn't reach")]
    fn replay_missing_guest_line() {
        let mut replay = Replay::new(p2_trace());
        // The guest's lines may still come while `start` waits for the next line's step.
        assert_eq!(replay.next_host_line(), Some(p2_trace().0[0].clone()));
        assert_eq!(replay.next_host_line(), Some(p2_trace().0[4].clone()));
        assert_eq!(replay.next_host_line(), None);
        replay.finish();
    }

    #[test]
    fn host_drops_of_mocks() {
        let mock = Label {
            table: Table::Imports,
            name: "#counter-1".to_string(),
        };
        let trace = Trace(vec![line(Party::Host, Event::Drop(mock))]);
        let mut replay = Replay::new(trace);
        // Neither `start`'s, nor needed.
        assert_eq!(replay.next_host_line(), None);
        replay.finish();
    }

    #[test]
    fn replay_renames_labels() {
        let session = |name: &str| Label {
            table: Table::Exports,
            name: name.to_string(),
        };
        let (t1, t2) = (task(Table::Exports, 1), task(Table::Exports, 2));
        let trace = recorded(&[
            line(Party::Host, call(t1, "new", &[])),
            line(Party::Guest, ret(t1, Some("#session-3"))),
            line(Party::Host, Event::Drop(session("#session-3"))),
            line(Party::Host, call(t2, "new", &[])),
            line(Party::Guest, ret(t2, Some("#session-3"))),
        ]);
        assert_eq!(trace.0[4].to_string(), "guest: return t2 #session-3-2");
        let mut replay = Replay::new(trace);
        replay.next_host_line();
        // Replay's table numbers the session differently.
        replay.check(&line(Party::Guest, ret(t1, Some("#session-1"))));
        let drop = line(Party::Host, Event::Drop(session("#session-1")));
        assert_eq!(replay.next_host_line(), Some(drop));
        // Once dropped, both labels may name another session.
        replay.next_host_line();
        replay.check(&line(Party::Guest, ret(t2, Some("#session-2"))));
        replay.finish();
    }

    #[test]
    fn replay_ends_labels_handed_on() {
        let (t1, t2, t3) = (
            task(Table::Exports, 1),
            task(Table::Exports, 2),
            task(Table::Exports, 3),
        );
        let trace = recorded(&[
            line(Party::Host, call(t1, "new", &[])),
            line(Party::Guest, ret(t1, Some("#session-3"))),
            // The host hands the session back, which frees its number.
            line(Party::Host, call(t2, "close", &["#session-3"])),
            line(Party::Guest, ret(t2, None)),
            line(Party::Host, call(t3, "new", &[])),
            line(Party::Guest, ret(t3, Some("#session-3"))),
        ]);
        let mut replay = Replay::new(trace);
        replay.next_host_line();
        replay.check(&line(Party::Guest, ret(t1, Some("#session-1"))));
        let close = replay.next_host_line().unwrap();
        assert_eq!(close.event, call(t2, "close", &["#session-1"]));
        replay.check(&line(Party::Guest, ret(t2, None)));
        replay.next_host_line();
        // The recording reused the number, and replay didn't, or the other way round.
        replay.check(&line(Party::Guest, ret(t3, Some("#session-2"))));
        replay.finish();
    }

    #[test]
    fn replay_finds_reused_labels() {
        // The guest hands the counter on to the host, which returns another with its number, as
        // record mode's trace has it.
        let (c1, c2) = (task(Table::Imports, 1), task(Table::Imports, 2));
        let counter = Label {
            table: Table::Imports,
            name: "#counter-2".to_string(),
        };
        let lines = [
            line(Party::Guest, call(c1, "new", &[])),
            line(Party::Host, ret(c1, Some("#counter-2"))),
            line(Party::Guest, call(c2, "merge", &["#counter-2"])),
            line(Party::Host, ret(c2, Some("#counter-2"))),
            line(Party::Guest, Event::Drop(counter.clone())),
        ];
        let trace = recorded(&lines);
        assert_eq!(trace.0[4].to_string(), "guest: drop #counter-2-3");
        // Replay's own labels, which it reuses too.
        let mut replay = Replay::new(trace);
        assert_eq!(replay.check(&lines[0]), vec![lines[1].clone()]);
        assert_eq!(replay.check(&lines[2]), vec![lines[3].clone()]);
        replay.check(&lines[4]);
        replay.finish();
    }

    #[test]
    #[should_panic(expected = "the trace has `guest: return t2 #session-3`")]
    fn replay_checks_labels() {
        let (t1, t2) = (task(Table::Exports, 1), task(Table::Exports, 2));
        let trace = Trace(vec![
            line(Party::Host, call(t1, "new", &[])),
            line(Party::Guest, ret(t1, Some("#session-3"))),
            line(Party::Host, call(t2, "get", &[])),
            line(Party::Guest, ret(t2, Some("#session-3"))),
        ]);
        let mut replay = Replay::new(trace);
        replay.next_host_line();
        replay.check(&line(Party::Guest, ret(t1, Some("#session-1"))));
        replay.next_host_line();
        // The trace's `#session-3` is replay's `#session-1`, not another session.
        replay.check(&line(Party::Guest, ret(t2, Some("#session-2"))));
    }

    #[test]
    fn cut_traces_lack_the_guests_last_copies() {
        let t1 = task(Table::Exports, 1);
        let stream = Label {
            table: Table::Imports,
            name: "#stream-3".to_string(),
        };
        let drop = line(Party::Guest, Event::Drop(stream));
        // The run trapped during `t1`, before the host saw the guest's drop of its stream.
        let mut replay = Replay::new(Trace(vec![line(Party::Host, call(t1, "run", &[]))]));
        assert!(replay.is_cut());
        replay.next_host_line();
        assert_eq!(replay.check(&drop), vec![]);
        // A whole trace has every line.
        let whole = Trace(vec![
            line(Party::Host, call(t1, "run", &[])),
            line(Party::Guest, ret(t1, None)),
        ]);
        let mut replay = Replay::new(whole);
        assert!(!replay.is_cut());
        replay.next_host_line();
        let check = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| replay.check(&drop)));
        assert!(check.is_err());
    }

    #[test]
    fn started_then_return() {
        let c1 = task(Table::Imports, 1);
        let late = Line {
            at: Some(3),
            ..line(Party::Host, ret(c1, Some("5")))
        };
        let async_call = Event::Call(Call {
            id: c1,
            is_async: true,
            func: "wait".to_string(),
            args: vec![],
        });
        let trace = Trace(vec![
            line(Party::Guest, async_call),
            line(Party::Host, Event::Started(c1)),
            late.clone(),
        ]);
        let mut replay = Replay::new(trace.clone());
        // The `Return` comes at its step, not in answer to the call, and the mock takes it.
        assert_eq!(replay.check(&trace.0[0]), vec![trace.0[1].clone()]);
        assert_eq!(replay.next_host_line(), None);
        assert_eq!(replay.take_host_line(&LineKey::Task(c1)), Some(late));
        replay.finish();
    }
}
