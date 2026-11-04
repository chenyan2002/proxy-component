mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    async fn start() -> () {
        let __executor = wit_bindgen::spawn_local(proxy::conversion::executor::run());
        /// Calls an async export, and records that the guest started it, if it blocks.
        #[allow(dead_code)]
        async fn call_export<T>(
            id: TaskId,
            call: impl std::future::Future<Output = T>,
        ) -> T {
            let mut call = std::pin::pin!(call);
            let mut started = false;
            std::future::poll_fn(|cx| {
                    let poll = call.as_mut().poll(cx);
                    if poll.is_pending() && !started {
                        started = true;
                        proxy::recorder::record::record(
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
        let __buf = proxy::util::debug::get_random();
        let mut u = Unstructured::new(&__buf);
        for _ in 0..10 {
            let idx = u.int_in_range(1..=9usize).unwrap();
            match idx {
                1usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let url: String = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
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
                    proxy::recorder::record::record(
                        &trace_return(
                            Party::Guest,
                            __id,
                            Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                        ),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                2usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let port: u16 = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&port)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
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
                    proxy::recorder::record::record(
                        &trace_return(
                            Party::Guest,
                            __id,
                            Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                        ),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                3usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let s: wit_bindgen::rt::async_support::StreamReader<u8> = Fuzz::fuzz(
                            &mut u,
                        )
                        .unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
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
                    proxy::recorder::record::record(
                        &trace_return(
                            Party::Guest,
                            __id,
                            Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                        ),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                4usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let url: String = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:p3/host@0.1.0.fetch",
                                true,
                                __params,
                            )
                        },
                    );
                    let res = call_export(__id, test::p3::host::fetch(url)).await;
                    proxy::recorder::record::record(
                        &trace_return(
                            Party::Guest,
                            __id,
                            Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                        ),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                5usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let ms: u64 = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&ms)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:p3/host@0.1.0.sleep",
                                true,
                                __params,
                            )
                        },
                    );
                    let res = call_export(__id, test::p3::host::sleep(ms)).await;
                    proxy::recorder::record::record(
                        &trace_return(Party::Guest, __id, None),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                6usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let err: wit_bindgen::rt::async_support::ErrorContext = Fuzz::fuzz(
                            &mut u,
                        )
                        .unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&err)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:p3/host@0.1.0.report",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::p3::host::report(err);
                    proxy::recorder::record::record(
                        &trace_return(Party::Guest, __id, None),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                7usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let done: wit_bindgen::rt::async_support::FutureReader<
                        ::core::result::Result<(), test::p3::host::ErrorCode>,
                    > = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&done)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:p3/host@0.1.0.finish",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::p3::host::finish(done);
                    proxy::recorder::record::record(
                        &trace_return(Party::Guest, __id, None),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                8usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let contents: ::core::option::Option<
                        wit_bindgen::rt::async_support::StreamReader<u8>,
                    > = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(
                            wasm_wave::to_string(&ToValue::to_value(&contents)).unwrap(),
                        );
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "new test:p3/host/body@0.1.0",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::p3::host::Body::new(contents);
                    proxy::recorder::record::record(
                        &trace_return(
                            Party::Guest,
                            __id,
                            Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                        ),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                9usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let this: test::p3::host::Body = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&this)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:p3/host/body@0.1.0.consume",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::p3::host::Body::consume(this);
                    proxy::recorder::record::record(
                        &trace_return(
                            Party::Guest,
                            __id,
                            Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                        ),
                    );
                    proxy::conversion::executor::host_drops(true);
                    Fuzz::consume(res, &mut u);
                    SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                    proxy::conversion::executor::host_drops(false);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
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
impl ToRust<MockedResource> for Value {
    fn to_rust(&self) -> MockedResource {
        let label = self.unwrap_handle();
        let (name, handle) = label
            .rsplit_once('-')
            .and_then(|(name, handle_str)| {
                handle_str.parse::<u32>().ok().map(|h| (name.to_string(), h))
            })
            .expect("invalid handle label");
        MockedResource { handle, name }
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
fn host_stream<T: wit_stream::StreamPayload + 'static>() -> (
    wit_bindgen::StreamWriter<T>,
    wit_bindgen::StreamReader<T>,
    String,
) {
    let (writer, reader) = wit_stream::new::<T>();
    label_by_writer(reader.handle(), writer.handle());
    let label = handle_label(&writer);
    (writer, reader, label)
}
/// A future the host writes, which it always does: its writer, its reader, to hand
/// out, and its label.
fn host_future<T: wit_future::FuturePayload + 'static>() -> (
    wit_bindgen::FutureWriter<T>,
    wit_bindgen::FutureReader<T>,
    String,
) {
    let (writer, reader) = wit_future::new::<
        T,
    >(|| unreachable!("the value is always written"));
    label_by_writer(reader.handle(), writer.handle());
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
        let (mut writer, reader, name) = host_stream::<T>();
        let label = Label { table: TRACE_TABLE, name };
        let chunks: Vec<Vec<T>> = Fuzz::fuzz(u)?;
        spawn(async move {
            let mut chunks = chunks.into_iter().filter(|chunk| !chunk.is_empty());
            while let Some(chunk) = chunks.next() {
                let items = wasm_wave::to_string(&chunk.to_value()).unwrap();
                let at = trace_at();
                let rest = writer.write_all(chunk).await;
                if !rest.is_empty() {
                    proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                    host_dropping(move || drop((rest, chunks.collect::<Vec<_>>())));
                    return;
                }
                let chunk = Chunk {
                    stream: label.clone(),
                    items,
                };
                proxy::recorder::record::record(&host_line_at(at, Event::Write(chunk)));
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
                let at = trace_at();
                let (result, items) = self.read(Vec::with_capacity(capacity)).await;
                if let wit_bindgen::StreamResult::Dropped = result {
                    proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                    return;
                }
                let wave = wasm_wave::to_string(&items.to_value()).unwrap();
                let chunk = Chunk {
                    stream: label.clone(),
                    items: wave,
                };
                proxy::recorder::record::record(&host_line_at(at, Event::Read(chunk)));
                host_dropping(move || drop(items));
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
        let (writer, reader, name) = host_future::<T>();
        let label = Label { table: TRACE_TABLE, name };
        let wave = wasm_wave::to_string(&value.to_value()).unwrap();
        spawn(async move {
            let at = trace_at();
            match writer.write(value).await {
                Ok(()) => {
                    let value = FutureValue {
                        future: label,
                        value: wave,
                    };
                    proxy::recorder::record::record(
                        &host_line_at(at, Event::Value(value)),
                    );
                }
                Err(unwritten) => {
                    proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                    host_dropping(move || drop(unwritten));
                }
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
    proxy::conversion::executor::step()
}
#[allow(dead_code)]
fn host_dropping<R>(f: impl FnOnce() -> R) -> R {
    proxy::conversion::executor::host_drops(true);
    let res = f();
    proxy::conversion::executor::host_drops(false);
    res
}
thread_local! {
    #[allow(dead_code)] static MOVING : std::cell::Cell < bool > = const {
    std::cell::Cell::new(false) };
}
thread_local! {
    static MOCK_LABEL : std::cell::Cell < u32 > = const { std::cell::Cell::new(0) };
    static HOST_DROP_AT : std::cell::Cell < Option < u32 >> = const {
    std::cell::Cell::new(None) };
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
