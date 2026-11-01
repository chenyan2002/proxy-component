mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    async fn start() -> () {
        let __executors = (
            wit_bindgen::spawn_local(proxy::executor::executor::run()),
            wit_bindgen::spawn_local(run_executor()),
        );
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
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/api@0.1.0.handle", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::api::handle(url).await;
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                2usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let port: u16 = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&port)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/api@0.1.0.serve", __params
                            .join(", ")
                        ),
                    );
                    Fuzz::consume(test::p3::api::serve(port).await, &mut u);
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
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/api@0.1.0.echo", __params
                            .join(", ")
                        ),
                    );
                    Fuzz::consume(test::p3::api::echo(s), &mut u);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                4usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let url: String = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.fetch", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::fetch(url).await;
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                5usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let ms: u64 = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&ms)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.sleep", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::sleep(ms).await;
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
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.report", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::report(err);
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
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.finish", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::finish(done);
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
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "new test:p3/host/body@0.1.0", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::Body::new(contents);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                9usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let this: test::p3::host::Body = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&this)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:p3/host/body@0.1.0.consume", __params
                            .join(", ")
                        ),
                    );
                    Fuzz::consume(test::p3::host::Body::consume(this), &mut u);
                    for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                        wit_bindgen::yield_async().await;
                    }
                }
                _ => unreachable!(),
            }
            SCOPED_ALLOC
                .with(|alloc| {
                    alloc.borrow_mut().clear();
                });
        }
        proxy::executor::executor::stop();
        stop_executor();
        __executors.0.await;
        __executors.1.await;
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
            handle: 0,
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
impl<T> Fuzz for wit_bindgen::StreamReader<T>
where
    T: Fuzz + ToValue + ValueTyped + wit_stream::StreamPayload,
{
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let (mut writer, reader) = wit_stream::new::<T>();
        let label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        let chunks: Vec<Vec<T>> = Fuzz::fuzz(u)?;
        spawn(async move {
            for chunk in chunks.into_iter().filter(|chunk| !chunk.is_empty()) {
                let items = wasm_wave::to_string(&chunk.to_value()).unwrap();
                if !writer.write_all(chunk).await.is_empty() {
                    proxy::util::debug::print(&format!("{label} dropped by guest"));
                    return;
                }
                proxy::util::debug::print(&format!("{label}.read() = {items}"));
            }
            proxy::util::debug::print(&format!("{label}.read() = dropped"));
        });
        Ok(reader)
    }
    fn consume(mut self, u: &mut Unstructured<'_>) {
        let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
        let reads: Option<u32> = match u.int_in_range(0..=9u8).unwrap_or(0) {
            8 => Some(u.int_in_range(1..=4).unwrap_or(1)),
            9 => Some(0),
            _ => None,
        };
        let capacity = u.int_in_range(1..=4096usize).unwrap_or(4096);
        spawn(async move {
            let mut count = 0;
            while reads != Some(count) {
                let (result, items) = self.read(Vec::with_capacity(capacity)).await;
                if let wit_bindgen::StreamResult::Dropped = result {
                    proxy::util::debug::print(&format!("{label} -> dropped"));
                    return;
                }
                let items = wasm_wave::to_string(&items.to_value()).unwrap();
                proxy::util::debug::print(&format!("{label} -> {items}"));
                count += 1;
            }
            proxy::util::debug::print(&format!("{label} dropped by host"));
        });
    }
}
impl<T> Fuzz for wit_bindgen::FutureReader<T>
where
    T: Fuzz + ToValue + ValueTyped + wit_future::FuturePayload,
{
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let value = T::fuzz(u)?;
        let (writer, reader) = wit_future::new::<
            T,
        >(|| unreachable!("the value is always written"));
        let label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        let wave = wasm_wave::to_string(&value.to_value()).unwrap();
        spawn(async move {
            if writer.write(value).await.is_ok() {
                proxy::util::debug::print(&format!("{label}.read() = {wave}"));
            } else {
                proxy::util::debug::print(&format!("{label} dropped by guest"));
            }
        });
        Ok(reader)
    }
    fn consume(self, u: &mut Unstructured<'_>) {
        let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
        let read = u.int_in_range(0..=9u8).unwrap_or(0) != 9;
        spawn(async move {
            if read {
                let value = wasm_wave::to_string(&self.await.to_value()).unwrap();
                proxy::util::debug::print(&format!("{label} -> {value}"));
            } else {
                drop(self);
                proxy::util::debug::print(&format!("{label} dropped by host"));
            }
        });
    }
}
impl Fuzz for wit_bindgen::rt::async_support::ErrorContext {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self::new(&String::fuzz(u)?))
    }
}
thread_local! {
    #[allow(dead_code)] static PENDING_RET : std::cell::Cell < bool > = const {
    std::cell::Cell::new(false) };
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
    std::cell::RefCell::new(Executor::default());
}
/// Queues `job`, from any function, sync or async.
#[allow(dead_code)]
fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
    EXECUTOR
        .with_borrow_mut(|e| {
            e.queue.push(Box::pin(job));
            if let Some(waker) = e.waker.take() {
                waker.wake();
            }
        });
}
/// Polls the queued jobs until `stop_executor` is called and they are all done.
#[allow(dead_code)]
async fn run_executor() {
    let mut running: Vec<Job> = Vec::new();
    std::future::poll_fn(|cx| {
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
                        std::task::Poll::Pending
                    }
                })
        })
        .await
}
#[allow(dead_code)]
fn stop_executor() {
    EXECUTOR
        .with_borrow_mut(|e| {
            e.stopped = true;
            if let Some(waker) = e.waker.take() {
                waker.wake();
            }
        });
}
