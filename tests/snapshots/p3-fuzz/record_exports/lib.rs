mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    async fn start() -> () {
        static PANIC_HOOK: std::sync::Once = std::sync::Once::new();
        PANIC_HOOK
            .call_once(|| std::panic::set_hook(
                Box::new(|info| proxy::recorder::record::fail(&info.to_string())),
            ));
        let __executor = wit_bindgen::spawn_local(proxy::conversion::executor::run());
        let __buf = proxy::util::debug::get_random();
        let mut u = Unstructured::new(&__buf);
        for _ in 0..10 {
            let idx = u.int_in_range(1..=3usize).unwrap();
            match idx {
                1usize => {
                    let res = {
                        let __runs = u.int_in_range(0..=3u8).unwrap() as u32;
                        let __at = proxy::conversion::executor::step_after(
                                proxy::conversion::executor::current_step() + __runs,
                            )
                            .await;
                        let mut __params: Vec<String> = Vec::new();
                        let url: String = Fuzz::fuzz(&mut u).unwrap();
                        __params
                            .push(
                                wasm_wave::to_string(&ToValue::to_value(&url)).unwrap(),
                            );
                        let __id = trace_task_id(Table::Exports);
                        proxy::recorder::record::record(
                            &Line {
                                at: Some(__at),
                                ..trace_call(
                                    Party::Host,
                                    __id,
                                    "test:p3/api@0.1.0.handle",
                                    true,
                                    __params,
                                )
                            },
                        );
                        let res = call_export(__id, test::p3::api::handle(url)).await;
                        trace_guest(
                            &trace_return(
                                Party::Guest,
                                __id,
                                Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                            ),
                        );
                        proxy::conversion::executor::host_drops(true);
                        res
                    };
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                }
                2usize => {
                    let res = {
                        let __runs = u.int_in_range(0..=3u8).unwrap() as u32;
                        let __at = proxy::conversion::executor::step_after(
                                proxy::conversion::executor::current_step() + __runs,
                            )
                            .await;
                        let mut __params: Vec<String> = Vec::new();
                        let port: u16 = Fuzz::fuzz(&mut u).unwrap();
                        __params
                            .push(
                                wasm_wave::to_string(&ToValue::to_value(&port)).unwrap(),
                            );
                        let __id = trace_task_id(Table::Exports);
                        proxy::recorder::record::record(
                            &Line {
                                at: Some(__at),
                                ..trace_call(
                                    Party::Host,
                                    __id,
                                    "test:p3/api@0.1.0.serve",
                                    true,
                                    __params,
                                )
                            },
                        );
                        let res = call_export(__id, test::p3::api::serve(port)).await;
                        trace_guest(
                            &trace_return(
                                Party::Guest,
                                __id,
                                Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                            ),
                        );
                        proxy::conversion::executor::host_drops(true);
                        res
                    };
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                }
                3usize => {
                    let res = {
                        let __runs = u.int_in_range(0..=3u8).unwrap() as u32;
                        let __at = proxy::conversion::executor::step_after(
                                proxy::conversion::executor::current_step() + __runs,
                            )
                            .await;
                        let mut __params: Vec<String> = Vec::new();
                        let s: wit_bindgen::rt::async_support::StreamReader<u8> = Fuzz::fuzz(
                                &mut u,
                            )
                            .unwrap();
                        __params
                            .push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
                        let __id = trace_task_id(Table::Exports);
                        proxy::recorder::record::record(
                            &Line {
                                at: Some(__at),
                                ..trace_call(
                                    Party::Host,
                                    __id,
                                    "test:p3/api@0.1.0.echo",
                                    false,
                                    __params,
                                )
                            },
                        );
                        let res = test::p3::api::echo(s);
                        trace_guest(
                            &trace_return(
                                Party::Guest,
                                __id,
                                Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                            ),
                        );
                        proxy::conversion::executor::host_drops(true);
                        res
                    };
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                }
                _ => unreachable!(),
            }
        }
        join_spawned().await;
        proxy::conversion::executor::stop();
        __executor.await;
    }
}
impl ValueTyped for test::p3::host::ErrorCode {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let cases = vec![
            ("timeout", None), ("other", Some(< ::core::option::Option < String > as
            ValueTyped > ::value_type()))
        ];
        Type::variant(cases).unwrap()
    }
}
impl ToValue for test::p3::host::ErrorCode {
    fn to_value(&self) -> Value {
        let ty = test::p3::host::ErrorCode::value_type();
        match self {
            Self::Timeout => Value::make_variant(&ty, "timeout", None),
            Self::Other(e) => Value::make_variant(&ty, "other", Some(e.to_value())),
        }
            .unwrap()
    }
}
impl ValueTyped for test::p3::host::Body {
    fn value_type() -> Type {
        Type::handle("body")
    }
}
impl<'a> ValueTyped for &'a test::p3::host::Body {
    fn value_type() -> Type {
        Type::handle("body")
    }
}
impl ToValue for test::p3::host::Body {
    fn to_value(&self) -> Value {
        let label = format!(
            "{}-{}", "body",
            proxy::conversion::conversion::get_handle_mock_test_p3_host_body(self)
        );
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for &'a test::p3::host::Body {
    fn to_value(&self) -> Value {
        let label = format!(
            "borrow-{}-{}", "body",
            proxy::conversion::conversion::get_handle_mock_test_p3_host_body(self)
        );
        Value::make_handle(label.into())
    }
}
#[allow(unused_imports)]
use wasm_wave::{
    wasm::WasmValue, value::{Value, Type, convert::{ToRust, ToValue, ValueTyped}},
};
use std::{alloc::Layout, cell::RefCell};
#[allow(dead_code)]
struct ScopedAlloc {
    ptrs: Vec<(*mut u8, Layout, fn(*mut u8))>,
}
thread_local! {
    static SCOPED_ALLOC : RefCell < ScopedAlloc > = RefCell::new(ScopedAlloc::new());
}
#[allow(dead_code)]
impl ScopedAlloc {
    fn new() -> Self {
        Self { ptrs: Vec::new() }
    }
    fn alloc<T>(&mut self, value: T) -> &'static T {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        fn drop_ptr<T>(ptr: *mut u8) {
            drop(unsafe { Box::from_raw(ptr as *mut T) });
        }
        self.ptrs.push((ptr as *mut u8, Layout::new::<T>(), drop_ptr::<T>));
        unsafe { &*ptr }
    }
    fn clear(&mut self) {
        for (ptr, _layout, drop_fn) in self.ptrs.drain(..) {
            drop_fn(ptr);
        }
    }
}
impl Drop for ScopedAlloc {
    fn drop(&mut self) {
        self.clear();
    }
}
#[derive(Default, Debug)]
struct MockedResource {
    handle: u32,
    name: String,
}
impl ValueTyped for MockedResource {
    fn value_type() -> Type {
        Type::handle("mocked-resource")
    }
}
impl ToValue for MockedResource {
    fn to_value(&self) -> Value {
        let label = format!("{}-{}", self.name, self.handle);
        Value::make_handle(label.into())
    }
}
impl ValueTyped for &MockedResource {
    fn value_type() -> Type {
        Type::handle("mocked-resource")
    }
}
impl ToValue for &MockedResource {
    fn to_value(&self) -> Value {
        let label = format!("borrow-{}-{}", self.name, self.handle);
        Value::make_handle(label.into())
    }
}
impl Fuzz for test::p3::host::ErrorCode {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(2usize)?;
        match idx {
            0usize => Ok(test::p3::host::ErrorCode::Timeout),
            1usize => Ok(test::p3::host::ErrorCode::Other(Fuzz::fuzz(u)?)),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables)]
    fn consume(self, u: &mut Unstructured<'_>) {
        match self {
            test::p3::host::ErrorCode::Timeout => {}
            test::p3::host::ErrorCode::Other(x) => Fuzz::consume(x, u),
        }
    }
}
impl Fuzz for test::p3::host::Body {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(proxy::conversion::conversion::get_mock_test_p3_host_body(0))
    }
}
impl<'a> Fuzz for &'a test::p3::host::Body {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                Ok(
                    alloc
                        .alloc(
                            proxy::conversion::conversion::get_mock_test_p3_host_body(0),
                        ),
                )
            })
    }
}
#[allow(unused_imports)]
use arbitrary::{Arbitrary, Unstructured, Result};
trait Fuzz: Sized {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self>;
    #[allow(dead_code)]
    fn consume(self, _u: &mut Unstructured<'_>) {}
}
thread_local! {
    static FUZZ_SEED : u64 = { let random = proxy::util::debug::get_random();
    u64::from_le_bytes(random[..8].try_into().unwrap()) };
}
/// Fuzz data drawn from `text`, e.g. a call's arguments: the same for the same text in
/// a run, so that the host answers alike, and different in another run. The text's
/// hash seeds splitmix64, which makes as much data as `get-random` gives.
#[allow(dead_code)]
fn fuzz_data(text: &str) -> Vec<u8> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    FUZZ_SEED.with(|seed| seed.hash(&mut hasher));
    text.hash(&mut hasher);
    let mut state = hasher.finish();
    (0..128)
        .flat_map(|_| {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            (z ^ (z >> 31)).to_le_bytes()
        })
        .collect()
}
impl Fuzz for MockedResource {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self {
            handle: next_mock_label(),
            name: "mocked-resource".to_string(),
        })
    }
}
macro_rules! impl_fuzz_primitive {
    ($($t:ty),*) => {
        $(impl Fuzz for $t { fn fuzz(u : & mut Unstructured < '_ >) -> Result < Self > {
        u.arbitrary() } })*
    };
}
impl_fuzz_primitive!(
    u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool, char, String, ()
);
impl<T: Fuzz> Fuzz for Option<T> {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(if u.arbitrary()? { Some(Fuzz::fuzz(u)?) } else { None })
    }
    fn consume(self, u: &mut Unstructured<'_>) {
        if let Some(x) = self {
            x.consume(u);
        }
    }
}
impl<T: Fuzz> Fuzz for Vec<T> {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let mut res = Vec::new();
        while u.arbitrary()? {
            res.push(Fuzz::fuzz(u)?);
        }
        Ok(res)
    }
    fn consume(self, u: &mut Unstructured<'_>) {
        for x in self {
            x.consume(u);
        }
    }
}
impl<T: Fuzz, E: Fuzz> Fuzz for core::result::Result<T, E> {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(if u.arbitrary()? { Ok(Fuzz::fuzz(u)?) } else { Err(Fuzz::fuzz(u)?) })
    }
    fn consume(self, u: &mut Unstructured<'_>) {
        match self {
            Ok(x) => x.consume(u),
            Err(x) => x.consume(u),
        }
    }
}
macro_rules! impl_fuzz_tuple {
    ($($T:ident),+) => {
        impl < $($T : Fuzz),+ > Fuzz for ($($T,)+) { fn fuzz(u : & mut Unstructured < '_
        >) -> Result < Self > { Ok(($($T ::fuzz(u) ?,)+)) } #[allow(non_snake_case)] fn
        consume(self, u : & mut Unstructured < '_ >) { let ($($T,)+) = self; $($T
        .consume(u);)+ } }
    };
}
impl_fuzz_tuple!(T1);
impl_fuzz_tuple!(T1, T2);
impl_fuzz_tuple!(T1, T2, T3);
impl_fuzz_tuple!(T1, T2, T3, T4);
impl_fuzz_tuple!(T1, T2, T3, T4, T5);
impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6);
impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
thread_local! {
    static READER_LABELS : std::cell::RefCell < std::collections::HashMap < u32, u32 >> =
    std::cell::RefCell::new(std::collections::HashMap::new());
}
/// Labels a stream or future this component made by the handle of its `writer`, which it
/// keeps for the stream's lifetime, also where its `reader` is printed: in the value that
/// hands the reader out, after which the reader moves on, and its handle can be reused.
/// So the entry is gone once the reader is printed.
fn label_by_writer(reader: u32, writer: u32) {
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        wasm_wave::value::convert::set_handle_label(|_kind, handle| {
            READER_LABELS.with_borrow_mut(|labels| labels.remove(&handle))
        })
    });
    READER_LABELS
        .with_borrow_mut(|labels| {
            labels.remove(&writer);
            labels.insert(reader, writer);
        });
}
/// The label of a stream or future, e.g. `#stream-7`, by the end this component keeps.
fn handle_label(end: &impl ToValue) -> String {
    wasm_wave::to_string(&end.to_value()).unwrap()
}
/// A stream the host writes: its writer, its reader, to hand out, and its label.
#[allow(dead_code)]
fn host_stream<T: wit_stream::StreamPayload + 'static>(
    label_reader: bool,
) -> (wit_bindgen::StreamWriter<T>, wit_bindgen::StreamReader<T>, String) {
    let (writer, reader) = wit_stream::new::<T>();
    if label_reader {
        label_by_writer(reader.handle(), writer.handle());
    }
    let label = handle_label(&writer);
    (writer, reader, label)
}
/// A future the host writes, which it always does: its writer, its reader, to hand
/// out, and its label.
#[allow(dead_code)]
fn host_future<T: wit_future::FuturePayload + 'static>(
    label_reader: bool,
) -> (wit_bindgen::FutureWriter<T>, wit_bindgen::FutureReader<T>, String) {
    let (writer, reader) = wit_future::new::<
        T,
    >(|| unreachable!("the value is always written"));
    if label_reader {
        label_by_writer(reader.handle(), writer.handle());
    }
    let label = handle_label(&writer);
    (writer, reader, label)
}
/// The label of a stream or future, by the end this component keeps.
fn label_of(end: &impl ToValue) -> Label {
    Label {
        table: TRACE_TABLE,
        name: handle_label(end),
    }
}
/// A host line, in step `at`, in which its job made it, e.g. issued the write the
/// guest read; or, by default, in the current one.
fn host_line_at(at: u32, event: Event) -> Line {
    Line {
        by: Party::Host,
        at: Some(at),
        event,
    }
}
fn host_line(event: Event) -> Line {
    host_line_at(trace_at(), event)
}
fn guest_line(event: Event) -> Line {
    Line {
        by: Party::Guest,
        at: None,
        event,
    }
}
impl<T> Fuzz for wit_bindgen::StreamReader<T>
where
    T: Fuzz + ToValue + ValueTyped + wit_stream::StreamPayload,
{
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let (mut writer, reader, name) = host_stream::<T>(true);
        let label = Label { table: TRACE_TABLE, name };
        let chunks: Vec<Vec<T>> = Fuzz::fuzz(u)?;
        spawn(async move {
            let mut chunks = chunks.into_iter().filter(|chunk| !chunk.is_empty());
            while let Some(chunk) = chunks.next() {
                let waves: Vec<_> = chunk
                    .iter()
                    .map(|item| wasm_wave::to_string(&item.to_value()).unwrap())
                    .collect();
                let (mut items, mut offset) = (chunk, 0);
                while !items.is_empty() {
                    let (ready, _) = writer.write(Vec::new()).await;
                    if let wit_bindgen::StreamResult::Dropped = ready {
                        proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                        host_dropping(move || drop((items, chunks.collect::<Vec<_>>())));
                        return;
                    }
                    let (at, count) = (trace_at(), items.len());
                    let (result, rest) = writer.write(items).await;
                    let rest = rest.into_vec();
                    let written = count - rest.len();
                    if written > 0 {
                        let items = format!(
                            "[{}]", waves[offset..offset + written].join(", ")
                        );
                        let chunk = Chunk {
                            stream: label.clone(),
                            items,
                        };
                        proxy::recorder::record::record(
                            &host_line_at(at, Event::Write(chunk)),
                        );
                        offset += written;
                    }
                    if let wit_bindgen::StreamResult::Dropped = result {
                        proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                        host_dropping(move || drop((rest, chunks.collect::<Vec<_>>())));
                        return;
                    }
                    items = rest;
                }
            }
            drop(writer);
            proxy::recorder::record::record(&host_line(Event::Drop(label)));
        });
        Ok(reader)
    }
    fn consume(mut self, u: &mut Unstructured<'_>) {
        let label = label_of(&self);
        let reads: Option<u32> = match u.int_in_range(0..=9u8).unwrap_or(0) {
            8 => Some(u.int_in_range(1..=4).unwrap_or(1)),
            9 => Some(0),
            _ => None,
        };
        let capacity = u.int_in_range(1..=4096usize).unwrap_or(4096);
        spawn(async move {
            let mut count = 0;
            while reads != Some(count) {
                let (ready, _) = self.read(Vec::new()).await;
                if let wit_bindgen::StreamResult::Dropped = ready {
                    proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                    return;
                }
                let at = trace_at();
                let (result, items) = self.read(Vec::with_capacity(capacity)).await;
                if !items.is_empty() {
                    let wave = wasm_wave::to_string(&items.to_value()).unwrap();
                    let chunk = Chunk {
                        stream: label.clone(),
                        items: wave,
                    };
                    proxy::recorder::record::record(
                        &host_line_at(at, Event::Read(chunk)),
                    );
                    host_dropping(move || drop(items));
                }
                if let wit_bindgen::StreamResult::Dropped = result {
                    proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                    return;
                }
                count += 1;
            }
            drop(self);
            proxy::recorder::record::record(&host_line(Event::Drop(label)));
        });
    }
}
impl<T> Fuzz for wit_bindgen::FutureReader<T>
where
    T: Fuzz + ToValue + ValueTyped + wit_future::FuturePayload,
{
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let value = T::fuzz(u)?;
        let (writer, reader, name) = host_future::<T>(true);
        let label = Label { table: TRACE_TABLE, name };
        let wave = wasm_wave::to_string(&value.to_value()).unwrap();
        spawn(async move {
            let line = FutureValue {
                future: label.clone(),
                value: wave,
            };
            proxy::recorder::record::record(&host_line(Event::Value(line)));
            if let Err(unwritten) = writer.write(value).await {
                proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                host_dropping(move || drop(unwritten));
            }
        });
        Ok(reader)
    }
    fn consume(self, u: &mut Unstructured<'_>) {
        let label = label_of(&self);
        let read = u.int_in_range(0..=9u8).unwrap_or(0) != 9;
        spawn(async move {
            if read {
                let value = self.await;
                let wave = wasm_wave::to_string(&value.to_value()).unwrap();
                let line = guest_line(
                    Event::Value(FutureValue {
                        future: label,
                        value: wave,
                    }),
                );
                proxy::recorder::record::record(&line);
                host_dropping(move || drop(value));
            } else {
                drop(self);
                proxy::recorder::record::record(&host_line(Event::Drop(label)));
            }
        });
    }
}
impl Fuzz for wit_bindgen::rt::async_support::ErrorContext {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self::new(&String::fuzz(u)?))
    }
    fn consume(self, _u: &mut Unstructured<'_>) {
        let name = wasm_wave::to_string(&self.to_value()).unwrap();
        proxy::recorder::record::record(&trace_drop(Party::Host, TRACE_TABLE, name));
    }
}
use bindings::proxy::recorder::types::*;
thread_local! {
    static TASK_ID : std::cell::Cell < u32 > = const { std::cell::Cell::new(0) };
}
#[allow(dead_code)]
fn trace_task_id(table: Table) -> TaskId {
    let id = TASK_ID
        .with(|id| {
            id.set(id.get() + 1);
            id.get()
        });
    TaskId { table, id }
}
#[allow(dead_code)]
fn trace_call(
    by: Party,
    id: TaskId,
    func: &str,
    is_async: bool,
    args: Vec<String>,
) -> Line {
    let call = Call {
        id,
        is_async,
        func: func.to_string(),
        args,
    };
    Line {
        by,
        at: None,
        event: Event::Call(call),
    }
}
#[allow(dead_code)]
fn trace_return(by: Party, id: TaskId, value: Option<String>) -> Line {
    let ret = ReturnValue { id, value };
    Line {
        by,
        at: None,
        event: Event::Return(ret),
    }
}
#[allow(dead_code)]
fn trace_drop(by: Party, table: Table, name: String) -> Line {
    let label = Label { table, name };
    Line {
        by,
        at: None,
        event: Event::Drop(label),
    }
}
/// The value of the `return` among the host's answers to a call.
#[allow(dead_code)]
fn trace_return_value(answers: Vec<Line>) -> Option<String> {
    answers
        .into_iter()
        .find_map(|line| match line.event {
            Event::Return(ret) => Some(ret.value),
            _ => None,
        })
        .expect("the trace doesn't answer the call")
}
#[allow(dead_code)]
const TRACE_TABLE: Table = Table::Exports;
#[allow(dead_code)]
fn trace_at() -> u32 {
    proxy::conversion::executor::current_step()
}
#[allow(dead_code)]
async fn trace_reach(at: Option<u32>) {
    if let Some(at) = at {
        proxy::conversion::executor::wait_until_step(at).await;
    }
}
#[allow(dead_code)]
fn host_dropping<R>(f: impl FnOnce() -> R) -> R {
    proxy::conversion::executor::host_drops(true);
    let res = f();
    proxy::conversion::executor::host_drops(false);
    res
}
/// A line of the guest, which record and fuzz modes record, and replay checks.
#[allow(dead_code)]
fn trace_guest(line: &Line) {
    proxy::recorder::record::record(line);
}
/// Calls an async export, and reports that the guest started it, if it blocks.
#[allow(dead_code)]
async fn call_export<T>(id: TaskId, call: impl std::future::Future<Output = T>) -> T {
    let mut call = std::pin::pin!(call);
    let mut started = false;
    std::future::poll_fn(|cx| {
            let poll = call.as_mut().poll(cx);
            if poll.is_pending() && !started {
                started = true;
                trace_guest(
                    &Line {
                        by: Party::Guest,
                        at: None,
                        event: Event::Started(id),
                    },
                );
            }
            poll
        })
        .await
}
thread_local! {
    #[allow(dead_code)] static MOVING : std::cell::Cell < bool > = const {
    std::cell::Cell::new(false) };
}
thread_local! {
    static MOCK_LABEL : std::cell::Cell < u32 > = const { std::cell::Cell::new(0) };
}
/// The label of a new mock.
#[allow(dead_code)]
fn next_mock_label() -> u32 {
    MOCK_LABEL.set(MOCK_LABEL.get() + 1);
    MOCK_LABEL.get()
}
thread_local! {
    static SPAWNED : std::cell::RefCell < Vec < wit_bindgen::JoinHandle < () >> > = const
    { std::cell::RefCell::new(Vec::new()) };
}
#[allow(dead_code)]
fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
    let handle = wit_bindgen::spawn_local(job);
    SPAWNED.with_borrow_mut(|spawned| spawned.push(handle));
}
#[allow(dead_code)]
async fn join_spawned() {
    loop {
        let spawned = SPAWNED.with_borrow_mut(std::mem::take);
        if spawned.is_empty() {
            break;
        }
        for handle in spawned {
            handle.await;
        }
    }
}
