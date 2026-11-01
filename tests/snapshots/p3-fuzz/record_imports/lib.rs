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
            handle: 0,
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
        run_executor().await
    }
    fn stop() -> () {
        stop_executor()
    }
}
impl exports::test::p3::host::Guest for Stub {
    type Body = MockedResource;
    async fn fetch(
        url: String,
    ) -> ::core::result::Result<
        exports::test::p3::host::Body,
        exports::test::p3::host::ErrorCode,
    > {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "test:p3/host@0.1.0.fetch", __params.join(",")),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        for _ in 0..__host.int_in_range(0..=3u8).unwrap_or(0) {
            wit_bindgen::yield_async().await;
        }
        let __buf = __params.join(",") + "test:p3/host@0.1.0.fetch";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    async fn sleep(ms: u64) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&ms)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "test:p3/host@0.1.0.sleep", __params.join(",")),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        for _ in 0..__host.int_in_range(0..=3u8).unwrap_or(0) {
            wit_bindgen::yield_async().await;
        }
    }
    #[allow(unused_variables)]
    fn report(err: wit_bindgen::rt::async_support::ErrorContext) -> () {}
    fn finish(
        done: wit_bindgen::rt::async_support::FutureReader<
            ::core::result::Result<(), exports::test::p3::host::ErrorCode>,
        >,
    ) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&done)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "test:p3/host@0.1.0.finish", __params.join(",")),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        Fuzz::consume(done, &mut __host);
    }
}
impl exports::test::p3::host::GuestBody for MockedResource {
    fn new(
        contents: ::core::option::Option<
            wit_bindgen::rt::async_support::StreamReader<u8>,
        >,
    ) -> Self {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&contents)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "new test:p3/host/body@0.1.0", __params.join(",")),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        Fuzz::consume(contents, &mut __host);
        let __buf = __params.join(",") + "new test:p3/host/body@0.1.0";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = MockedResource {
            name: "body".to_string(),
            ..Fuzz::fuzz(&mut u).unwrap()
        };
        PENDING_RET.with(|pending| pending.set(true));
        res
    }
    async fn size(&self) -> u64 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "test:p3/host/body@0.1.0.size", __params.join(",")
            ),
        );
        let __random = proxy::util::debug::get_random();
        let mut __host = Unstructured::new(&__random);
        for _ in 0..__host.int_in_range(0..=3u8).unwrap_or(0) {
            wit_bindgen::yield_async().await;
        }
        let __buf = __params.join(",") + "test:p3/host/body@0.1.0.size";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
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
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "test:p3/host/body@0.1.0.consume", __params.join(",")
            ),
        );
        let __buf = __params.join(",") + "test:p3/host/body@0.1.0.consume";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    unsafe fn _resource_new(val: *mut u8) -> u32 {
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "[export]test:p3/host@0.1.0")]
        unsafe extern "C" {
            #[link_name = "[resource-new]body"]
            fn new(_: *mut u8) -> i32;
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe extern "C" fn new(_: *mut u8) -> i32 {
            unreachable!()
        }
        let handle = unsafe { new(val) } as u32;
        let mock = unsafe { &mut *(val as *mut Option<MockedResource>) }
            .as_mut()
            .unwrap();
        mock.handle = handle;
        if PENDING_RET.with(|pending| pending.take()) {
            let ret = format!(
                "ret: {}", wasm_wave::to_string(& mock.to_value()).unwrap()
            );
            proxy::util::debug::print(&ret);
        }
        handle
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
impl Fuzz for exports::test::p3::host::ErrorCode {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(2usize)?;
        match idx {
            0usize => Ok(exports::test::p3::host::ErrorCode::Timeout),
            1usize => Ok(exports::test::p3::host::ErrorCode::Other(Fuzz::fuzz(u)?)),
            _ => unreachable!(),
        }
    }
}
impl Fuzz for exports::test::p3::host::Body {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(
            exports::test::p3::host::Body::new(MockedResource {
                handle: 0,
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
        let label = wasm_wave::to_string(&ToValue::to_value(&writer)).unwrap();
        let reader_label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        proxy::util::debug::print(
            &format!("new (writer: {label}, reader: {reader_label})"),
        );
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
        let label = wasm_wave::to_string(&ToValue::to_value(&writer)).unwrap();
        let reader_label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        proxy::util::debug::print(
            &format!("new (writer: {label}, reader: {reader_label})"),
        );
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
