mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::conversion::conversion::Guest for Stub {
    fn get_mock_test_p3_host_body(
        handle: u32,
    ) -> exports::proxy::conversion::conversion::TestP3HostBody {
        let _ = handle;
        exports::proxy::conversion::conversion::TestP3HostBody::new(MockedResource {
            handle: next_mock_label(),
            name: "body".to_string(),
        })
    }
    fn get_handle_mock_test_p3_host_body(
        x: exports::proxy::conversion::conversion::TestP3HostBodyBorrow<'_>,
    ) -> u32 {
        x.get::<MockedResource>().handle
    }
}
impl exports::proxy::conversion::executor::Guest for Stub {
    async fn run() -> () {
        static PANIC_HOOK: std::sync::Once = std::sync::Once::new();
        PANIC_HOOK
            .call_once(|| std::panic::set_hook(
                Box::new(|info| proxy::recorder::record::fail(&info.to_string())),
            ));
        run_executor().await
    }
    fn stop() -> () {
        stop_executor()
    }
    fn step() -> u32 {
        next_step()
    }
    fn current_step() -> u32 {
        current_step()
    }
    async fn wait_until_step(step: u32) -> () {
        wait_until_step(step).await
    }
    async fn arrive_after(step: u32) -> u32 {
        arrive_after(step).await
    }
    fn host_drops(dropping: bool) -> () {
        MOVING.with(|moving| moving.set(dropping))
    }
}
impl exports::test::p3::host::Guest for Stub {
    type Body = MockedResource;
    #[allow(unused_variables)]
    async fn fetch(
        url: String,
    ) -> ::core::result::Result<
        exports::test::p3::host::Body,
        exports::test::p3::host::ErrorCode,
    > {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "test:p3/host@0.1.0.fetch",
                true,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        MOVING.with(|moving| moving.set(false));
        let __runs = __host.int_in_range(0..=3u8).unwrap_or(0);
        let __at = if __runs > 0 {
            proxy::recorder::record::record(
                &Line {
                    by: Party::Host,
                    at: None,
                    event: Event::Started(__id),
                },
            );
            Some(wait_for_runs(__runs as u32).await)
        } else {
            None
        };
        let __buf = __params.join(",") + "test:p3/host@0.1.0.fetch";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(res_str))
            },
        );
        res
    }
    #[allow(unused_variables)]
    async fn sleep(ms: u64) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&ms)).unwrap());
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "test:p3/host@0.1.0.sleep",
                true,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        MOVING.with(|moving| moving.set(false));
        let __runs = __host.int_in_range(0..=3u8).unwrap_or(0);
        let __at = if __runs > 0 {
            proxy::recorder::record::record(
                &Line {
                    by: Party::Host,
                    at: None,
                    event: Event::Started(__id),
                },
            );
            Some(wait_for_runs(__runs as u32).await)
        } else {
            None
        };
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, None)
            },
        );
    }
    #[allow(unused_variables)]
    fn report(err: wit_bindgen::rt::async_support::ErrorContext) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&err)).unwrap());
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "test:p3/host@0.1.0.report",
                false,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        Fuzz::consume(err, &mut __host);
        MOVING.with(|moving| moving.set(false));
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, None)
            },
        );
    }
    #[allow(unused_variables)]
    fn finish(
        done: wit_bindgen::rt::async_support::FutureReader<
            ::core::result::Result<(), exports::test::p3::host::ErrorCode>,
        >,
    ) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&done)).unwrap());
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "test:p3/host@0.1.0.finish",
                false,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        Fuzz::consume(done, &mut __host);
        MOVING.with(|moving| moving.set(false));
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, None)
            },
        );
    }
}
impl exports::test::p3::host::GuestBody for MockedResource {
    #[allow(unused_variables)]
    fn new(
        contents: ::core::option::Option<
            wit_bindgen::rt::async_support::StreamReader<u8>,
        >,
    ) -> Self {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&contents)).unwrap());
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "new test:p3/host/body@0.1.0",
                false,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        Fuzz::consume(contents, &mut __host);
        MOVING.with(|moving| moving.set(false));
        let __at: Option<u32> = None;
        let __buf = __params.join(",") + "new test:p3/host/body@0.1.0";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = MockedResource {
            name: "body".to_string(),
            ..Fuzz::fuzz(&mut u).unwrap()
        };
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(res_str))
            },
        );
        res
    }
    #[allow(unused_variables)]
    async fn size(&self) -> u64 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "test:p3/host/body@0.1.0.size",
                true,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        MOVING.with(|moving| moving.set(false));
        let __runs = __host.int_in_range(0..=3u8).unwrap_or(0);
        let __at = if __runs > 0 {
            proxy::recorder::record::record(
                &Line {
                    by: Party::Host,
                    at: None,
                    event: Event::Started(__id),
                },
            );
            Some(wait_for_runs(__runs as u32).await)
        } else {
            None
        };
        let __buf = __params.join(",") + "test:p3/host/body@0.1.0.size";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(res_str))
            },
        );
        res
    }
    #[allow(unused_variables)]
    fn consume(
        this: exports::test::p3::host::Body,
    ) -> (
        wit_bindgen::rt::async_support::StreamReader<u8>,
        wit_bindgen::rt::async_support::FutureReader<
            ::core::result::Result<
                ::core::option::Option<exports::test::p3::host::Body>,
                exports::test::p3::host::ErrorCode,
            >,
        >,
    ) {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&this)).unwrap());
        let __id = trace_task_id(Table::Imports);
        proxy::recorder::record::record(
            &trace_call(
                Party::Guest,
                __id,
                "test:p3/host/body@0.1.0.consume",
                false,
                __params.clone(),
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        MOVING.with(|moving| moving.set(true));
        drop((this,));
        MOVING.with(|moving| moving.set(false));
        let __at: Option<u32> = None;
        let __buf = __params.join(",") + "test:p3/host/body@0.1.0.consume";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(res_str))
            },
        );
        res
    }
    unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
        if let Some(res) = *unsafe { Box::from_raw(handle) } {
            let name = wasm_wave::to_string(&res.to_value()).unwrap();
            let by = if MOVING.with(|moving| moving.get()) {
                Party::Host
            } else {
                Party::Guest
            };
            let line = trace_drop(by, Table::Imports, name);
            proxy::recorder::record::record(&line);
        }
        None
    }
}
impl ValueTyped for exports::test::p3::host::ErrorCode {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let cases = vec![
            ("timeout", None), ("other", Some(< ::core::option::Option < String > as
            ValueTyped > ::value_type()))
        ];
        Type::variant(cases).unwrap()
    }
}
impl ToValue for exports::test::p3::host::ErrorCode {
    fn to_value(&self) -> Value {
        let ty = exports::test::p3::host::ErrorCode::value_type();
        match self {
            Self::Timeout => Value::make_variant(&ty, "timeout", None),
            Self::Other(e) => Value::make_variant(&ty, "other", Some(e.to_value())),
        }
            .unwrap()
    }
}
impl ValueTyped for exports::test::p3::host::Body {
    fn value_type() -> Type {
        Type::handle("body")
    }
}
impl<'a> ValueTyped for exports::test::p3::host::BodyBorrow<'a> {
    fn value_type() -> Type {
        Type::handle("body")
    }
}
impl ToValue for exports::test::p3::host::Body {
    fn to_value(&self) -> Value {
        let handle = self.get::<MockedResource>().handle;
        let label = format!("{}-{}", "body", handle);
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for exports::test::p3::host::BodyBorrow<'a> {
    fn to_value(&self) -> Value {
        let handle = self.get::<MockedResource>().handle;
        let label = format!("borrow-{}-{}", "body", handle);
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
impl Fuzz for exports::test::p3::host::ErrorCode {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(2usize)?;
        match idx {
            0usize => Ok(exports::test::p3::host::ErrorCode::Timeout),
            1usize => Ok(exports::test::p3::host::ErrorCode::Other(Fuzz::fuzz(u)?)),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables)]
    fn consume(self, u: &mut Unstructured<'_>) {
        match self {
            exports::test::p3::host::ErrorCode::Timeout => {}
            exports::test::p3::host::ErrorCode::Other(x) => Fuzz::consume(x, u),
        }
    }
}
impl Fuzz for exports::test::p3::host::Body {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(
            exports::test::p3::host::Body::new(MockedResource {
                handle: next_mock_label(),
                name: "body".to_string(),
            }),
        )
    }
}
impl<'a> Fuzz for exports::test::p3::host::BodyBorrow<'a> {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        unreachable!()
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
                let at = trace_at();
                let rest = writer.write_all(chunk).await;
                let written = waves.len() - rest.len();
                if written > 0 {
                    let items = format!("[{}]", waves[..written].join(", "));
                    let chunk = Chunk {
                        stream: label.clone(),
                        items,
                    };
                    proxy::recorder::record::record(
                        &host_line_at(at, Event::Write(chunk)),
                    );
                }
                if !rest.is_empty() {
                    proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                    host_dropping(move || drop((rest, chunks.collect::<Vec<_>>())));
                    return;
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
const TRACE_TABLE: Table = Table::Imports;
#[allow(dead_code)]
fn trace_at() -> u32 {
    current_step()
}
#[allow(dead_code)]
async fn trace_reach(at: Option<u32>) {
    if let Some(at) = at {
        reach_step(at).await;
    }
}
#[allow(dead_code)]
fn host_dropping<R>(f: impl FnOnce() -> R) -> R {
    MOVING.with(|moving| moving.set(true));
    let res = f();
    MOVING.with(|moving| moving.set(false));
    res
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
type Job = std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>;
#[derive(Default)]
struct Executor {
    queue: Vec<Job>,
    stopped: bool,
    waker: Option<std::task::Waker>,
}
thread_local! {
    static EXECUTOR : std::cell::RefCell < Executor > =
    std::cell::RefCell::new(Executor::default()); static STEP : std::cell::Cell < u32 > =
    const { std::cell::Cell::new(0) }; static WAKE_AGAIN : std::cell::Cell < bool > =
    const { std::cell::Cell::new(false) }; static HELD : std::cell::Cell < bool > = const
    { std::cell::Cell::new(false) }; static WOKEN_WHILE_HELD : std::cell::Cell < bool > =
    const { std::cell::Cell::new(false) };
}
/// Counts a step, and returns it.
#[allow(dead_code)]
fn next_step() -> u32 {
    STEP.set(STEP.get() + 1);
    STEP.get()
}
/// The step a job runs in.
#[allow(dead_code)]
fn current_step() -> u32 {
    STEP.get()
}
/// Waits, from any task, until the executor has run `runs` more times, and returns the step of
/// the last run.
#[allow(dead_code)]
async fn wait_for_runs(runs: u32) -> u32 {
    let mut left = runs;
    wait_for_run(
            move || {
                if left == 0 {
                    return true;
                }
                left -= 1;
                false
            },
            false,
        )
        .await
}
/// Waits, from any task, until a run of the executor in `step` or later, or returns at once if
/// the count is already there, e.g. replay's for a host line recorded in that step, which checks
/// that it came in that step.
#[allow(dead_code)]
async fn wait_until_step(step: u32) {
    if current_step() >= step {
        return;
    }
    wait_for_run(move || current_step() >= step, false).await;
}
/// Waits, from any task, as `wait_until_step` does, and takes the next step, of a host line
/// `start` makes, e.g. an export call's arrival, which it returns. The executor doesn't run in
/// between, e.g. for a guest's event, so that the step follows the run it waited for, which
/// replay waits for from the step alone.
#[allow(dead_code)]
async fn arrive_after(step: u32) -> u32 {
    if current_step() < step {
        wait_for_run(move || current_step() >= step, true).await;
    }
    HELD.set(false);
    let step = next_step();
    if WOKEN_WHILE_HELD.replace(false) {
        wake_executor();
    }
    step
}
/// Waits, in a job of the executor, until its run in `step`, waking the executor itself until
/// then, e.g. replay's job for a copy recorded in that step.
///
/// Panics if the count passed `step`: replay diverged from the recording.
#[allow(dead_code)]
async fn reach_step(step: u32) {
    std::future::poll_fn(|_| {
            let now = current_step();
            assert!(
                now <= step, "replay reached step {now} before the run in step {step}"
            );
            if now == step {
                std::task::Poll::Ready(())
            } else {
                WAKE_AGAIN.set(true);
                std::task::Poll::Pending
            }
        })
        .await
}
/// Waits until a run of the executor in which `done` returns true, and returns its step. A job
/// asks in each run, waking the executor itself in between, as no host event would. With `hold`,
/// the executor doesn't run again until `arrive_after` releases it.
async fn wait_for_run(mut done: impl FnMut() -> bool + 'static, hold: bool) -> u32 {
    #[derive(Default)]
    struct Done {
        step: Option<u32>,
        waker: Option<std::task::Waker>,
    }
    let shared = std::rc::Rc::new(std::cell::RefCell::new(Done::default()));
    let job_shared = shared.clone();
    spawn(
        std::future::poll_fn(move |_| {
            if done() {
                if hold {
                    HELD.set(true);
                }
                let mut shared = job_shared.borrow_mut();
                shared.step = Some(current_step());
                if let Some(waker) = shared.waker.take() {
                    waker.wake();
                }
                return std::task::Poll::Ready(());
            }
            WAKE_AGAIN.set(true);
            std::task::Poll::Pending
        }),
    );
    std::future::poll_fn(move |cx| {
            let mut shared = shared.borrow_mut();
            match shared.step {
                Some(step) => std::task::Poll::Ready(step),
                None => {
                    shared.waker = Some(cx.waker().clone());
                    std::task::Poll::Pending
                }
            }
        })
        .await
}
/// Queues `job`, from any function, sync or async.
#[allow(dead_code)]
fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
    EXECUTOR
        .with_borrow_mut(|e| {
            e.queue.push(Box::pin(job));
        });
    wake_executor();
}
/// Wakes `run_executor` from another task, with wit-bindgen's `inter-task-wakeup`.
fn wake_executor() {
    let waker = EXECUTOR.with_borrow_mut(|e| e.waker.take());
    if let Some(waker) = waker {
        waker.wake();
    }
}
/// Polls the queued jobs until `stop_executor` is called and they are all done.
#[allow(dead_code)]
async fn run_executor() {
    let mut running: Vec<Job> = Vec::new();
    std::future::poll_fn(|cx| {
            if HELD.get() {
                WOKEN_WHILE_HELD.set(true);
                EXECUTOR.with_borrow_mut(|e| e.waker = Some(cx.waker().clone()));
                return std::task::Poll::Pending;
            }
            next_step();
            WAKE_AGAIN.set(false);
            running.retain_mut(|job| job.as_mut().poll(cx).is_pending());
            loop {
                let mut queued = EXECUTOR
                    .with_borrow_mut(|e| std::mem::take(&mut e.queue));
                if queued.is_empty() {
                    break;
                }
                queued.retain_mut(|job| job.as_mut().poll(cx).is_pending());
                running.append(&mut queued);
            }
            EXECUTOR
                .with_borrow_mut(|e| {
                    if e.stopped && running.is_empty() {
                        std::task::Poll::Ready(())
                    } else {
                        e.waker = Some(cx.waker().clone());
                        if WAKE_AGAIN.get() {
                            cx.waker().wake_by_ref();
                        }
                        std::task::Poll::Pending
                    }
                })
        })
        .await
}
#[allow(dead_code)]
fn stop_executor() {
    EXECUTOR.with_borrow_mut(|e| e.stopped = true);
    wake_executor();
}
