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

mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "trace",
        additional_derives: [serde::Serialize, serde::Deserialize, PartialEq],
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
            && self.event.task().is_some_and(|id| id.table == Table::Imports)
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
    /// Panics on a trace line that doesn't parse, e.g. of another version of the format.
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
        Trace(lines.collect())
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

/// A trace being replayed: which of its lines replay has performed or checked.
pub struct Replay {
    lines: Vec<Line>,
    done: Vec<bool>,
}

impl Replay {
    pub fn new(trace: Trace) -> Self {
        let done = vec![false; trace.0.len()];
        Replay {
            lines: trace.0,
            done,
        }
    }
    fn pending(&self) -> impl Iterator<Item = (usize, &Line)> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(idx, _)| !self.done[*idx])
    }
    /// The next line the host performs on its own: in trace order, the first host line that
    /// doesn't answer a guest call, e.g. an export call, or the host's drop of a guest resource.
    /// `None` at the end of the trace.
    ///
    /// Panics if a guest line comes before it, which the guest hasn't done.
    pub fn next_host_line(&mut self) -> Option<Line> {
        let (idx, line) = self
            .pending()
            .find(|(_, line)| line.by == Party::Guest || !line.answers_guest())?;
        if line.by == Party::Guest {
            panic!("the trace has `{line}`, which the guest didn't do");
        }
        let line = line.clone();
        self.done[idx] = true;
        Some(line)
    }
    /// Checks a guest line against the trace, and returns the host lines that answer it at once,
    /// e.g. the `Return` of an import call. A guest line is matched by its task, for the lines of
    /// a call, and otherwise as the trace's next guest line.
    ///
    /// Panics on a mismatch, with the expected and the actual line.
    pub fn check(&mut self, actual: &Line) -> Vec<Line> {
        assert_eq!(actual.by, Party::Guest, "`{actual}` isn't a guest line");
        let task = actual.event.task();
        let found = self.pending().find(|(_, line)| match task {
            Some(task) => line.event.task() == Some(task),
            None => line.by == Party::Guest,
        });
        let Some((idx, expected)) = found else {
            panic!("the guest did `{actual}`, which the trace doesn't have");
        };
        if expected.by != Party::Guest || expected.event != actual.event {
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
            self.done[idx] = true;
            answers.push(line.clone());
            if !matches!(line.event, Event::Started(_)) {
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
        Line { by, at: None, event }
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
    #[should_panic(expected = "the trace has `guest: call c1 get(1)`, which the guest didn't do")]
    fn replay_missing_guest_line() {
        let mut replay = Replay::new(p2_trace());
        replay.next_host_line();
        replay.next_host_line();
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
        // The `Return` comes at its step, not in answer to the call.
        assert_eq!(replay.check(&trace.0[0]), vec![trace.0[1].clone()]);
        assert_eq!(replay.next_host_line(), Some(late));
    }
}
