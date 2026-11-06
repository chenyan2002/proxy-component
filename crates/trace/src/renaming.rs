//! The labels of a trace, and their pairing with replay's. A label is a handle's number in a
//! generated component's table, which names the handle until it leaves the table: at a `Drop`, a
//! future's `Value`, or when the host hands it on, owned, to the guest, in an export call's
//! arguments or an import's result. After that, the number may name another handle, as a
//! variable may be bound again. So the recorder names each binding apart, as it records the lines
//! (see `Labels`): the first keeps the number, e.g. `#stream-7`, and the next are `#stream-7-2`,
//! `#stream-7-3`, and so on. A trace's label then names one handle, from its first line to the
//! line that ends it.
//!
//! The tables of the recording and of replay differ: besides the values the trace labels, they
//! hold waitable sets and subtasks, which each mode makes its own. So replay compares lines up to
//! renaming: it pairs each label of the trace with the label replay sees for the same value, one
//! to one, per table, from the first line that has both, and requires every later line to agree.
//! Replay's labels are its own handle numbers, which it reuses too: a replay label can be paired
//! again once the trace's label it was paired with has ended. The rule is the same for every
//! label, whatever it names.
use std::collections::HashMap;

use crate::{Event, Label, Line, Party, Table};

/// A trace label's handle in a table, e.g. `(imports, "stream-7")`.
type Handle = (Table, String);

/// Names the bindings of labels apart, as a recorder records the lines (see the module's doc).
#[derive(Default)]
pub struct Labels {
    /// The number of bindings of each handle that have ended.
    ended: HashMap<Handle, u32>,
}

impl Labels {
    /// `line`, with each label renamed to its binding's, and the bindings it ends ended.
    pub fn label(&mut self, line: Line) -> Line {
        let ends = ends_of(&line);
        let line = map_labels(&line, |table, handle| {
            let ended = *self.ended.get(&(table, handle.to_string()))?;
            Some(format!("{handle}-{}", ended + 1))
        });
        for handle in ends {
            *self.ended.entry(handle).or_insert(0) += 1;
        }
        line
    }
}

/// The handle a binding's label names, and which binding of its number it is, as `Labels` names
/// them, e.g. `("stream-7", 2)` for `stream-7-2`, and `("stream-7", 1)` for `stream-7`. A handle
/// ends with its number, and a WIT name's words don't start with a digit.
fn binding_of(label: &str) -> (&str, u32) {
    let is_number = |text: &str| !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    match label.rsplit_once('-') {
        Some((handle, binding))
            if is_number(binding) && handle.rsplit_once('-').is_some_and(|(_, n)| is_number(n)) =>
        {
            (handle, binding.parse().unwrap())
        }
        _ => (label, 1),
    }
}

/// The binding of a handle's number after `label`'s, e.g. `stream-7-2` after `stream-7`.
fn next_binding(label: &str) -> String {
    let (handle, binding) = binding_of(label);
    format!("{handle}-{}", binding + 1)
}

pub(crate) struct Renaming {
    /// The line each label of the trace ends in.
    end_of: HashMap<Handle, usize>,
    to_replay: HashMap<Handle, String>,
    to_trace: HashMap<Handle, String>,
}

/// A WAVE value, cut into its labels, e.g. `#counter-1` or `#borrow-counter-1`, and the text
/// between them, which may have a `#` in a string or a char.
#[derive(Debug, PartialEq)]
enum Token<'a> {
    Text(&'a str),
    Label(&'a str),
}

fn tokens(wave: &str) -> Vec<Token<'_>> {
    let bytes = wave.as_bytes();
    let mut tokens = Vec::new();
    let (mut start, mut idx) = (0, 0);
    while idx < bytes.len() {
        match bytes[idx] {
            quote @ (b'"' | b'\'') => {
                idx += 1;
                while idx < bytes.len() && bytes[idx] != quote {
                    idx += if bytes[idx] == b'\\' { 2 } else { 1 };
                }
                idx += 1;
            }
            b'#' => {
                if start < idx {
                    tokens.push(Token::Text(&wave[start..idx]));
                }
                let end = idx
                    + 1
                    + bytes[idx + 1..]
                        .iter()
                        .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'-')
                        .count();
                tokens.push(Token::Label(&wave[idx..end]));
                (start, idx) = (end, end);
            }
            _ => idx += 1,
        }
    }
    if start < bytes.len() {
        tokens.push(Token::Text(&wave[start..]));
    }
    tokens
}

/// A label, e.g. `#borrow-counter-1`, as its borrow, and the handle it names, `counter-1`.
fn handle_of(label: &str) -> (bool, &str) {
    let label = label.trim_start_matches('#');
    match label.strip_prefix("borrow-") {
        Some(handle) => (true, handle),
        None => (false, label),
    }
}

/// The table a line's labels are in: its task's, or its handle's.
fn table_of(event: &Event) -> Table {
    match event {
        Event::Write(chunk) | Event::Read(chunk) => chunk.stream.table,
        Event::Value(value) => value.future.table,
        Event::Drop(label) => label.table,
        event => event.task().unwrap().table,
    }
}

/// The handles a line's labels name, and the owned ones among those in its values.
fn handles_of(line: &Line) -> (Vec<Handle>, Vec<Handle>) {
    let table = table_of(&line.event);
    let (mut all, mut owned) = (Vec::new(), Vec::new());
    let mut wave = |wave: &String| {
        for token in tokens(wave) {
            if let Token::Label(label) = token {
                let (borrow, handle) = handle_of(label);
                all.push((table, handle.to_string()));
                if !borrow {
                    owned.push((table, handle.to_string()));
                }
            }
        }
    };
    let mut fields = Vec::new();
    match &line.event {
        Event::Call(call) => call.args.iter().for_each(&mut wave),
        Event::Return(ret) => ret.value.iter().for_each(&mut wave),
        Event::Write(chunk) | Event::Read(chunk) => {
            fields.push(&chunk.stream);
            wave(&chunk.items);
        }
        Event::Value(value) => {
            fields.push(&value.future);
            wave(&value.value);
        }
        Event::Drop(label) => fields.push(label),
        _ => {}
    }
    all.extend(
        fields
            .iter()
            .map(|label| (label.table, handle_of(&label.name).1.to_string())),
    );
    (all, owned)
}

/// The handles that leave the table with a line: a dropped one, a future whose value was taken,
/// or those the host hands on, owned, to the guest.
///
/// The party of a call or return says which way its handles move. A host line hands them from
/// a generated component to the guest: an export call's arguments, or an import's result, whose
/// labels end. A guest line hands them to a generated component: an import call's arguments, or
/// an export's result, which come in as new handles. The guest's own handles aren't labels.
fn ends_of(line: &Line) -> Vec<Handle> {
    let handle = |label: &Label| (label.table, handle_of(&label.name).1.to_string());
    match &line.event {
        Event::Drop(label) => vec![handle(label)],
        Event::Value(value) => vec![handle(&value.future)],
        Event::Call(_) | Event::Return(_) if line.by == Party::Host => handles_of(line).1,
        _ => Vec::new(),
    }
}

/// `wave` with its labels renamed by `rename`, of the handle a label names, if it has one.
fn rename(wave: &str, rename: impl Fn(&str) -> Option<String>) -> String {
    tokens(wave)
        .into_iter()
        .map(|token| match token {
            Token::Text(text) => text.to_string(),
            Token::Label(label) => match (handle_of(label), rename(handle_of(label).1)) {
                ((true, _), Some(handle)) => format!("#borrow-{handle}"),
                ((false, _), Some(handle)) => format!("#{handle}"),
                (_, None) => label.to_string(),
            },
        })
        .collect()
}

/// `line` with its labels renamed by `rename`, of a handle in a table, if it has a new name.
fn map_labels(line: &Line, rename: impl Fn(Table, &str) -> Option<String>) -> Line {
    let table = table_of(&line.event);
    let wave = |wave: &String| self::rename(wave, |handle| rename(table, handle));
    let label = |label: &Label| Label {
        table: label.table,
        name: self::rename(&label.name, |handle| rename(label.table, handle)),
    };
    let event = match &line.event {
        Event::Call(call) => Event::Call(crate::Call {
            args: call.args.iter().map(wave).collect(),
            ..call.clone()
        }),
        Event::Return(ret) => Event::Return(crate::ReturnValue {
            value: ret.value.as_ref().map(wave),
            ..ret.clone()
        }),
        Event::Write(chunk) => Event::Write(crate::Chunk {
            stream: label(&chunk.stream),
            items: wave(&chunk.items),
        }),
        Event::Read(chunk) => Event::Read(crate::Chunk {
            stream: label(&chunk.stream),
            items: wave(&chunk.items),
        }),
        Event::Value(value) => Event::Value(crate::FutureValue {
            future: label(&value.future),
            value: wave(&value.value),
        }),
        Event::Drop(handle) => Event::Drop(label(handle)),
        event => event.clone(),
    };
    Line {
        event,
        ..line.clone()
    }
}

impl Renaming {
    /// Panics if a label of the trace names a handle after its end: a trace whose bindings
    /// `Labels` didn't name apart.
    pub(crate) fn new(lines: &[Line]) -> Self {
        let mut labels = Labels::default();
        let mut end_of = HashMap::new();
        for (idx, line) in lines.iter().enumerate() {
            assert!(
                labels.label(line.clone()) == *line,
                "the trace has `{line}`, with a label after the line that ended it"
            );
            for handle in ends_of(line) {
                end_of.insert(handle, idx);
            }
        }
        Renaming {
            end_of,
            to_replay: HashMap::new(),
            to_trace: HashMap::new(),
        }
    }
    /// Whether the trace's label of `handle` has ended before line `idx`.
    fn has_ended(&self, handle: &Handle, idx: usize) -> bool {
        self.end_of.get(handle).is_some_and(|end| *end < idx)
    }
    /// Pairs the trace's label `trace` with replay's `replay`, at line `idx`, or returns false if
    /// either is paired with another, which is still live.
    fn pair(&mut self, idx: usize, table: Table, trace: &str, replay: &str) -> bool {
        let ((trace_borrow, trace), (replay_borrow, replay)) =
            (handle_of(trace), handle_of(replay));
        if trace_borrow != replay_borrow {
            return false;
        }
        let trace = (table, trace.to_string());
        if let Some(paired) = self.to_replay.get(&trace) {
            return paired == replay;
        }
        let replay = (table, replay.to_string());
        if let Some(other) = self.to_trace.get(&replay) {
            let other = (table, other.clone());
            if !self.has_ended(&other, idx) {
                return false;
            }
            // Replay reused the number of a handle that ended, as the trace says.
            self.to_replay.remove(&other);
        }
        self.to_replay.insert(trace.clone(), replay.1.clone());
        self.to_trace.insert(replay, trace.1);
        true
    }
    /// Pairs the trace's label of a stream or future replay makes, from the value of the line that
    /// hands it on, with replay's, of the end it keeps: the label's next binding, e.g. the writer
    /// of the stream a mock returns, whose number labeled the reader in that line. Returns false
    /// if no line hands it on, or either is paired with another.
    pub(crate) fn pair_next(&mut self, trace: &Label, replay: &Label) -> bool {
        let handle = (trace.table, handle_of(&trace.name).1.to_string());
        let Some(end) = self.end_of.get(&handle).copied() else {
            return false;
        };
        trace.table == replay.table
            && self.pair(end + 1, trace.table, &next_binding(&handle.1), &replay.name)
    }
    /// Whether the trace's WAVE value `trace`, in line `idx`, is replay's `replay`, up to the
    /// pairing, which it extends.
    fn unify(&mut self, idx: usize, table: Table, trace: &str, replay: &str) -> bool {
        let (trace, replay) = (tokens(trace), tokens(replay));
        trace.len() == replay.len()
            && trace.iter().zip(&replay).all(|tokens| match tokens {
                (Token::Text(trace), Token::Text(replay)) => trace == replay,
                (Token::Label(trace), Token::Label(replay)) => self.pair(idx, table, trace, replay),
                _ => false,
            })
    }
    /// The trace's label that replay's label, e.g. a job's for the stream it keeps, is paired
    /// with, if any.
    pub(crate) fn to_trace(&self, label: &Label) -> Option<Label> {
        let (_, handle) = handle_of(&label.name);
        let trace = self.to_trace.get(&(label.table, handle.to_string()))?;
        let name = rename(&label.name, |_| Some(trace.clone()));
        Some(Label {
            table: label.table,
            name,
        })
    }
    /// Whether the trace's label `trace` is of a binding of replay's label `replay`'s number, e.g.
    /// a mock's, which carries the trace's number.
    pub(crate) fn binds(trace: &Label, replay: &Label) -> bool {
        let (trace_borrow, trace_handle) = handle_of(&trace.name);
        let (replay_borrow, replay_handle) = handle_of(&replay.name);
        trace.table == replay.table
            && trace_borrow == replay_borrow
            && binding_of(trace_handle).0 == replay_handle
    }
    /// The line the trace's label `label` ends in, if it ends.
    pub(crate) fn end_of(&self, label: &Label) -> Option<usize> {
        let handle = (label.table, handle_of(&label.name).1.to_string());
        self.end_of.get(&handle).copied()
    }
    /// `line` of the trace, with the labels replay has paired its own with renamed to those, so
    /// that replay can perform it, e.g. drop the guest's resource the trace's label names.
    pub(crate) fn to_replay(&self, line: &Line) -> Line {
        map_labels(line, |table, handle| {
            self.to_replay.get(&(table, handle.to_string())).cloned()
        })
    }
    /// Whether line `idx` of the trace, `trace`, is replay's `replay`, up to the pairing, which it
    /// extends.
    pub(crate) fn matches(&mut self, idx: usize, trace: &Line, replay: &Line) -> bool {
        if trace.by != replay.by {
            return false;
        }
        let table = table_of(&trace.event);
        let label = |this: &mut Self, trace: &Label, replay: &Label| {
            trace.table == replay.table && this.pair(idx, trace.table, &trace.name, &replay.name)
        };
        match (&trace.event, &replay.event) {
            (Event::Call(trace), Event::Call(replay)) => {
                trace.id == replay.id
                    && trace.is_async == replay.is_async
                    && trace.func == replay.func
                    && trace.args.len() == replay.args.len()
                    && trace
                        .args
                        .iter()
                        .zip(&replay.args)
                        .all(|(trace, replay)| self.unify(idx, table, trace, replay))
            }
            (Event::Return(trace), Event::Return(replay)) => {
                trace.id == replay.id
                    && match (&trace.value, &replay.value) {
                        (Some(trace), Some(replay)) => self.unify(idx, table, trace, replay),
                        (None, None) => true,
                        _ => false,
                    }
            }
            (Event::Write(trace), Event::Write(replay))
            | (Event::Read(trace), Event::Read(replay)) => {
                label(self, &trace.stream, &replay.stream)
                    && self.unify(idx, table, &trace.items, &replay.items)
            }
            (Event::Value(trace), Event::Value(replay)) => {
                label(self, &trace.future, &replay.future)
                    && self.unify(idx, table, &trace.value, &replay.value)
            }
            (Event::Drop(trace), Event::Drop(replay)) => label(self, trace, replay),
            (trace, replay) => trace == replay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Call, ReturnValue, TaskId};

    #[test]
    fn tokens_skip_strings_and_chars() {
        assert_eq!(
            tokens(r#"(#counter-1, "a #counter-2 \" #x-3", '#', #borrow-pollable-4)"#),
            [
                Token::Text("("),
                Token::Label("#counter-1"),
                Token::Text(r#", "a #counter-2 \" #x-3", '#', "#),
                Token::Label("#borrow-pollable-4"),
                Token::Text(")"),
            ]
        );
        assert_eq!(tokens("#stream-7"), [Token::Label("#stream-7")]);
        assert_eq!(tokens("42"), [Token::Text("42")]);
    }

    fn guest_return(id: u32, value: &str) -> Line {
        Line {
            by: Party::Guest,
            at: None,
            event: Event::Return(ReturnValue {
                id: TaskId {
                    table: Table::Exports,
                    id,
                },
                value: Some(value.to_string()),
            }),
        }
    }
    fn host_call(id: u32, args: &[&str]) -> Line {
        Line {
            by: Party::Host,
            at: None,
            event: Event::Call(Call {
                id: TaskId {
                    table: Table::Exports,
                    id,
                },
                is_async: false,
                func: "f".to_string(),
                args: args.iter().map(|arg| arg.to_string()).collect(),
            }),
        }
    }

    #[test]
    fn unify_pairs_one_to_one() {
        let lines = [guest_return(1, "#session-3"), guest_return(2, "#session-3")];
        let mut renaming = Renaming::new(&lines);
        let table = Table::Exports;
        assert!(renaming.unify(0, table, "ok(#session-3)", "ok(#session-1)"));
        // The pairs hold, both ways, and for borrows.
        assert!(renaming.unify(1, table, "#borrow-session-3", "#borrow-session-1"));
        assert!(!renaming.unify(1, table, "#session-3", "#session-2"));
        assert!(!renaming.unify(1, table, "#session-4", "#session-1"));
        assert!(!renaming.unify(1, table, "#session-3", "#borrow-session-1"));
        // Per table.
        assert!(renaming.unify(1, Table::Imports, "#session-3", "#session-3"));
        assert!(!renaming.unify(1, table, "\"a\"", "\"b\""));
    }

    #[test]
    fn labels_name_bindings_apart() {
        // The host hands the session back, which ends its label, and the number names another.
        let mut labels = Labels::default();
        let lines: Vec<_> = [
            guest_return(1, "#session-3"),
            host_call(2, &["#borrow-session-3", "#session-3"]),
            guest_return(3, "#session-3"),
            host_call(4, &["#session-3"]),
            guest_return(5, "#session-3"),
        ]
        .into_iter()
        .map(|line| labels.label(line))
        .collect();
        assert_eq!(lines[1], host_call(2, &["#borrow-session-3", "#session-3"]));
        assert_eq!(lines[2], guest_return(3, "#session-3-2"));
        assert_eq!(lines[3], host_call(4, &["#session-3-2"]));
        assert_eq!(lines[4], guest_return(5, "#session-3-3"));
        assert_eq!(next_binding("session-3"), "session-3-2");
        assert_eq!(next_binding("session-3-2"), "session-3-3");
        assert_eq!(next_binding("error-context-1"), "error-context-1-2");
        assert_eq!(binding_of("session-3-2"), ("session-3", 2));
        assert_eq!(binding_of("error-context-1"), ("error-context-1", 1));
    }

    #[test]
    fn replay_labels_pair_again_once_the_trace_ends_theirs() {
        let lines = [
            guest_return(1, "#session-3"),
            host_call(2, &["#session-3"]),
            guest_return(3, "#session-3-2"),
        ];
        let table = Table::Exports;
        // Whichever side reuses its number.
        let mut renaming = Renaming::new(&lines);
        assert!(renaming.unify(0, table, "#session-3", "#session-1"));
        assert_eq!(renaming.to_replay(&lines[1]), host_call(2, &["#session-1"]));
        assert!(renaming.unify(2, table, "#session-3-2", "#session-2"));
        let mut renaming = Renaming::new(&lines);
        assert!(renaming.unify(0, table, "#session-3", "#session-1"));
        assert!(!renaming.unify(1, table, "#session-3-2", "#session-1"));
        assert!(renaming.unify(2, table, "#session-3-2", "#session-1"));
    }

    #[test]
    #[should_panic(expected = "with a label after the line that ended it")]
    fn labels_after_their_end() {
        Renaming::new(&[host_call(1, &["#session-3"]), guest_return(2, "#session-3")]);
    }

    #[test]
    fn to_trace_label() {
        let lines = [guest_return(1, "#session-3")];
        let mut renaming = Renaming::new(&lines);
        assert!(renaming.unify(0, Table::Exports, "#session-3", "#session-1"));
        let replay = Label {
            table: Table::Exports,
            name: "#borrow-session-1".to_string(),
        };
        assert_eq!(renaming.to_trace(&replay).unwrap().name, "#borrow-session-3");
    }
}
