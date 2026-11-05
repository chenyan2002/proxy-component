//! The pairing of a trace's labels with replay's. A label is a handle's number in a generated
//! component's table, and the tables of the recording and of replay differ: besides the values
//! the trace labels, they hold waitable sets and subtasks, which each mode makes its own. So
//! replay compares lines up to renaming: it pairs each label of the trace with the label replay
//! sees for the same value, one to one, per table, from the first line that has both, and requires
//! every later line to agree. A pair ends when its handle leaves the table, which frees its number:
//! at a `Drop`, a future's `Value`, or when the host hands it on, owned, to the guest, in an export
//! call's arguments or an import's result. The rule is the same for every label, whatever it names.
use std::collections::HashMap;

use crate::{Event, Label, Line, Party, Table};

#[derive(Default)]
pub(crate) struct Renaming {
    to_replay: HashMap<(Table, String), String>,
    to_trace: HashMap<(Table, String), String>,
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

impl Renaming {
    /// Pairs the trace's label `trace` with replay's `replay`, or returns false if either is
    /// paired with another.
    fn pair(&mut self, table: Table, trace: &str, replay: &str) -> bool {
        let ((trace_borrow, trace), (replay_borrow, replay)) = (handle_of(trace), handle_of(replay));
        if trace_borrow != replay_borrow {
            return false;
        }
        let key = |label: &str| (table, label.to_string());
        match (self.to_replay.get(&key(trace)), self.to_trace.get(&key(replay))) {
            (Some(paired), _) => paired == replay,
            (None, Some(_)) => false,
            (None, None) => {
                self.to_replay.insert(key(trace), replay.to_string());
                self.to_trace.insert(key(replay), trace.to_string());
                true
            }
        }
    }
    /// Ends the pair of the trace's label `trace`, whose handle has ended.
    fn unpair(&mut self, table: Table, trace: &str) {
        let (_, trace) = handle_of(trace);
        if let Some(replay) = self.to_replay.remove(&(table, trace.to_string())) {
            self.to_trace.remove(&(table, replay));
        }
    }
    /// Whether the trace's WAVE value `trace` is replay's `replay`, up to the pairing, which it
    /// extends.
    fn unify(&mut self, table: Table, trace: &str, replay: &str) -> bool {
        let (trace, replay) = (tokens(trace), tokens(replay));
        trace.len() == replay.len()
            && trace.iter().zip(&replay).all(|tokens| match tokens {
                (Token::Text(trace), Token::Text(replay)) => trace == replay,
                (Token::Label(trace), Token::Label(replay)) => self.pair(table, trace, replay),
                _ => false,
            })
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
    fn to_replay_wave(&self, table: Table, wave: &str) -> String {
        Self::rename(wave, |handle| self.to_replay.get(&(table, handle.to_string())).cloned())
    }
    fn to_replay_label(&self, label: &Label) -> Label {
        Label {
            table: label.table,
            name: self.to_replay_wave(label.table, &label.name),
        }
    }
    /// Replay's label, e.g. a job's for the stream it keeps, as the trace's.
    pub(crate) fn to_trace_label(&self, label: &Label) -> Label {
        let name = Self::rename(&label.name, |handle| {
            self.to_trace.get(&(label.table, handle.to_string())).cloned()
        });
        Label {
            table: label.table,
            name,
        }
    }
    /// A line of the trace, with the labels replay has paired its own with renamed to those, so
    /// that replay can perform it, e.g. drop the guest's resource the trace's label names.
    pub(crate) fn to_replay(&self, line: &Line) -> Line {
        let table = table_of(&line.event);
        let wave = |wave: &String| self.to_replay_wave(table, wave);
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
                stream: self.to_replay_label(&chunk.stream),
                items: wave(&chunk.items),
            }),
            Event::Read(chunk) => Event::Read(crate::Chunk {
                stream: self.to_replay_label(&chunk.stream),
                items: wave(&chunk.items),
            }),
            Event::Value(value) => Event::Value(crate::FutureValue {
                future: self.to_replay_label(&value.future),
                value: wave(&value.value),
            }),
            Event::Drop(label) => Event::Drop(self.to_replay_label(label)),
            event => event.clone(),
        };
        Line {
            event,
            ..line.clone()
        }
    }
    /// Whether the trace's line `trace` is replay's `replay`, up to the pairing, which it extends.
    pub(crate) fn matches(&mut self, trace: &Line, replay: &Line) -> bool {
        if trace.by != replay.by {
            return false;
        }
        let table = table_of(&trace.event);
        let mut label = |trace: &Label, replay: &Label| {
            trace.table == replay.table && self.pair(trace.table, &trace.name, &replay.name)
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
                        .all(|(trace, replay)| self.unify(table, trace, replay))
            }
            (Event::Return(trace), Event::Return(replay)) => {
                trace.id == replay.id
                    && match (&trace.value, &replay.value) {
                        (Some(trace), Some(replay)) => self.unify(table, trace, replay),
                        (None, None) => true,
                        _ => false,
                    }
            }
            (Event::Write(trace), Event::Write(replay))
            | (Event::Read(trace), Event::Read(replay)) => {
                label(&trace.stream, &replay.stream) && self.unify(table, &trace.items, &replay.items)
            }
            (Event::Value(trace), Event::Value(replay)) => {
                label(&trace.future, &replay.future) && self.unify(table, &trace.value, &replay.value)
            }
            (Event::Drop(trace), Event::Drop(replay)) => label(trace, replay),
            (trace, replay) => trace == replay,
        }
    }
    /// Ends the pairs of the handles that leave the table with the trace's line: a dropped one, a
    /// future whose value was taken, or those the host hands on, owned, to the guest.
    ///
    /// The party of a call or return says which way its handles move. A host line hands them from
    /// a generated component to the guest: an export call's arguments, or an import's result, whose
    /// labels end. A guest line hands them to a generated component: an import call's arguments,
    /// or an export's result, which come in as new handles, with new labels, which `unify` pairs.
    /// The guest's own handles aren't labels.
    pub(crate) fn end(&mut self, line: &Line) {
        let table = table_of(&line.event);
        let mut hand_on = |wave: &str| {
            for token in tokens(wave) {
                if let Token::Label(label) = token
                    && !handle_of(label).0
                {
                    self.unpair(table, label);
                }
            }
        };
        match &line.event {
            Event::Drop(label) => self.unpair(label.table, &label.name),
            Event::Value(value) => self.unpair(value.future.table, &value.future.name),
            Event::Call(call) if line.by == Party::Host => call.args.iter().for_each(|arg| hand_on(arg)),
            Event::Return(ret) if line.by == Party::Host => ret.value.iter().for_each(|value| hand_on(value)),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn unify_pairs_one_to_one() {
        let mut renaming = Renaming::default();
        let table = Table::Exports;
        assert!(renaming.unify(table, "ok(#session-3)", "ok(#session-1)"));
        // The pairs hold, both ways, and for borrows.
        assert!(renaming.unify(table, "#borrow-session-3", "#borrow-session-1"));
        assert!(!renaming.unify(table, "#session-3", "#session-2"));
        assert!(!renaming.unify(table, "#session-4", "#session-1"));
        assert!(!renaming.unify(table, "#session-3", "#borrow-session-1"));
        // Per table.
        assert!(renaming.unify(Table::Imports, "#session-3", "#session-3"));
        // Until the handle ends.
        renaming.unpair(table, "#session-3");
        assert!(renaming.unify(table, "#session-3", "#session-2"));
        assert!(!renaming.unify(table, "\"a\"", "\"b\""));
    }

    #[test]
    fn rename_to_replay() {
        let mut renaming = Renaming::default();
        assert!(renaming.unify(Table::Exports, "#session-3", "#session-1"));
        assert_eq!(
            renaming.to_replay_wave(Table::Exports, "[#borrow-session-3, #session-5]"),
            "[#borrow-session-1, #session-5]"
        );
        let replay = Label {
            table: Table::Exports,
            name: "#session-1".to_string(),
        };
        assert_eq!(renaming.to_trace_label(&replay).name, "#session-3");
    }
}
