mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::conversion::conversion::Guest for Stub {
    fn get_mock_test_host_types_counter(
        handle: u32,
    ) -> exports::proxy::conversion::conversion::TestHostTypesCounter {
        let _ = handle;
        exports::proxy::conversion::conversion::TestHostTypesCounter::new(MockedResource {
            handle: 0,
            name: "counter".to_string(),
        })
    }
    fn get_handle_mock_test_host_types_counter(
        x: exports::proxy::conversion::conversion::TestHostTypesCounterBorrow<'_>,
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
impl exports::test::host::types::Guest for Stub {
    type Counter = MockedResource;
    fn make_point(x: i32, y: i32) -> exports::test::host::types::Point {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "test:host/types@0.1.0.make-point", __params.join(",")
            ),
        );
        let __buf = __params.join(",") + "test:host/types@0.1.0.make-point";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    fn check(
        p: exports::test::host::types::Perms,
        c: exports::test::host::types::Color,
    ) -> ::core::result::Result<exports::test::host::types::Shape, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "test:host/types@0.1.0.check", __params.join(",")),
        );
        let __buf = __params.join(",") + "test:host/types@0.1.0.check";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    fn open(
        name: String,
    ) -> ::core::result::Result<exports::test::host::types::Counter, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "test:host/types@0.1.0.open", __params.join(",")),
        );
        let __buf = __params.join(",") + "test:host/types@0.1.0.open";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
}
impl exports::test::host::types::GuestCounter for MockedResource {
    fn new(start: u32) -> Self {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "new test:host/types/counter@0.1.0", __params.join(",")
            ),
        );
        let __buf = __params.join(",") + "new test:host/types/counter@0.1.0";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = MockedResource {
            name: "counter".to_string(),
            ..Fuzz::fuzz(&mut u).unwrap()
        };
        PENDING_RET.with(|pending| pending.set(true));
        res
    }
    fn get(&self) -> u32 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "test:host/types/counter@0.1.0.get", __params.join(",")
            ),
        );
        let __buf = __params.join(",") + "test:host/types/counter@0.1.0.get";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    #[allow(unused_variables)]
    fn add(&self, n: u32) -> () {}
    fn merge(
        a: exports::test::host::types::CounterBorrow<'_>,
        b: exports::test::host::types::Counter,
    ) -> exports::test::host::types::Counter {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&a)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&b)).unwrap());
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "test:host/types/counter@0.1.0.merge", __params
                .join(",")
            ),
        );
        let __buf = __params.join(",") + "test:host/types/counter@0.1.0.merge";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    unsafe fn _resource_new(val: *mut u8) -> u32 {
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "[export]test:host/types@0.1.0")]
        unsafe extern "C" {
            #[link_name = "[resource-new]counter"]
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
impl exports::test::log0_1_0::sink::Guest for Stub {
    #[allow(unused_variables)]
    fn write(msg: String) -> () {}
}
impl exports::test::log0_2_0::sink::Guest for Stub {
    fn write(msg: String) -> bool {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&msg)).unwrap());
        proxy::util::debug::print(
            &format!("import: {}({})", "test:log/sink@0.2.0.write", __params.join(",")),
        );
        let __buf = __params.join(",") + "test:log/sink@0.2.0.write";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = Fuzz::fuzz(&mut u).unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
}
impl ValueTyped for exports::test::host::types::Point {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let fields = vec![
            ("x", < i32 as ValueTyped > ::value_type()), ("y", < i32 as ValueTyped >
            ::value_type()), ("label", < ::core::option::Option < String > as ValueTyped
            > ::value_type())
        ];
        Type::record(fields).unwrap()
    }
}
impl ToValue for exports::test::host::types::Point {
    fn to_value(&self) -> Value {
        let ty = exports::test::host::types::Point::value_type();
        let fields = vec![
            ("x", self.x.to_value()), ("y", self.y.to_value()), ("label", self.label
            .to_value())
        ];
        Value::make_record(&ty, fields).unwrap()
    }
}
impl ValueTyped for exports::test::host::types::Shape {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let cases = vec![
            ("circle", Some(< u32 as ValueTyped > ::value_type())), ("rect", Some(<
            exports::test::host::types::Point as ValueTyped > ::value_type())),
            ("HTTP-error", Some(< String as ValueTyped > ::value_type())), ("empty",
            None)
        ];
        Type::variant(cases).unwrap()
    }
}
impl ToValue for exports::test::host::types::Shape {
    fn to_value(&self) -> Value {
        let ty = exports::test::host::types::Shape::value_type();
        match self {
            Self::Circle(e) => Value::make_variant(&ty, "circle", Some(e.to_value())),
            Self::Rect(e) => Value::make_variant(&ty, "rect", Some(e.to_value())),
            Self::HttpError(e) => {
                Value::make_variant(&ty, "HTTP-error", Some(e.to_value()))
            }
            Self::Empty => Value::make_variant(&ty, "empty", None),
        }
            .unwrap()
    }
}
impl ValueTyped for exports::test::host::types::Color {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let cases = vec![("red", None), ("green", None), ("DNS-blue", None)];
        Type::variant(cases).unwrap()
    }
}
impl ToValue for exports::test::host::types::Color {
    fn to_value(&self) -> Value {
        let ty = exports::test::host::types::Color::value_type();
        match self {
            Self::Red => Value::make_variant(&ty, "red", None),
            Self::Green => Value::make_variant(&ty, "green", None),
            Self::DnsBlue => Value::make_variant(&ty, "DNS-blue", None),
        }
            .unwrap()
    }
}
impl ValueTyped for exports::test::host::types::Perms {
    fn value_type() -> Type {
        Type::flags(vec!["read", "write", "exec"]).unwrap()
    }
}
impl ToValue for exports::test::host::types::Perms {
    fn to_value(&self) -> Value {
        let ty = Self::value_type();
        let mut flags = Vec::new();
        if *self & Self::READ != Self::empty() {
            flags.push("read");
        }
        if *self & Self::WRITE != Self::empty() {
            flags.push("write");
        }
        if *self & Self::EXEC != Self::empty() {
            flags.push("exec");
        }
        Value::make_flags(&ty, flags).unwrap()
    }
}
impl ValueTyped for exports::test::host::types::Counter {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl<'a> ValueTyped for exports::test::host::types::CounterBorrow<'a> {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl ToValue for exports::test::host::types::Counter {
    fn to_value(&self) -> Value {
        let handle = self.get::<MockedResource>().handle;
        let label = format!("{}-{}", "counter", handle);
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for exports::test::host::types::CounterBorrow<'a> {
    fn to_value(&self) -> Value {
        let handle = self.get::<MockedResource>().handle;
        let label = format!("borrow-{}-{}", "counter", handle);
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
impl Fuzz for exports::test::host::types::Point {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(exports::test::host::types::Point {
            x: Fuzz::fuzz(u)?,
            y: Fuzz::fuzz(u)?,
            label: Fuzz::fuzz(u)?,
        })
    }
}
impl Fuzz for exports::test::host::types::Shape {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(4usize)?;
        match idx {
            0usize => Ok(exports::test::host::types::Shape::Circle(Fuzz::fuzz(u)?)),
            1usize => Ok(exports::test::host::types::Shape::Rect(Fuzz::fuzz(u)?)),
            2usize => Ok(exports::test::host::types::Shape::HttpError(Fuzz::fuzz(u)?)),
            3usize => Ok(exports::test::host::types::Shape::Empty),
            _ => unreachable!(),
        }
    }
}
impl Fuzz for exports::test::host::types::Color {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(3usize)?;
        match idx {
            0usize => Ok(exports::test::host::types::Color::Red),
            1usize => Ok(exports::test::host::types::Color::Green),
            2usize => Ok(exports::test::host::types::Color::DnsBlue),
            _ => unreachable!(),
        }
    }
}
impl Fuzz for exports::test::host::types::Perms {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let flag_count = u.int_in_range(0..=2usize)?;
        let mut res = exports::test::host::types::Perms::empty();
        let choices = [
            exports::test::host::types::Perms::READ,
            exports::test::host::types::Perms::WRITE,
            exports::test::host::types::Perms::EXEC,
        ];
        for _ in 0..flag_count {
            let flag = u.choose(&choices)?;
            res |= *flag;
        }
        Ok(res)
    }
}
impl Fuzz for exports::test::host::types::Counter {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(
            exports::test::host::types::Counter::new(MockedResource {
                handle: 0,
                name: "counter".to_string(),
            }),
        )
    }
}
impl<'a> Fuzz for exports::test::host::types::CounterBorrow<'a> {
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
