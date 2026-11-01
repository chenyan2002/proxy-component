mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::conversion::conversion::Guest for Stub {
    fn get_mock_test_host_types_counter(
        handle: u32,
    ) -> exports::proxy::conversion::conversion::TestHostTypesCounter {
        exports::proxy::conversion::conversion::TestHostTypesCounter::new(MockedResource {
            handle,
            name: "counter".to_string(),
        })
    }
}
impl exports::proxy::executor::executor::Guest for Stub {
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
        let wave = proxy::recorder::replay::replay_import(
                Some("test:host/types@0.1.0.make-point"),
                Some(&__params),
            )
            .unwrap();
        let ret: Value = wasm_wave::from_str(
                &<exports::test::host::types::Point as ValueTyped>::value_type(),
                &wave,
            )
            .unwrap();
        ret.to_rust()
    }
    fn check(
        p: exports::test::host::types::Perms,
        c: exports::test::host::types::Color,
    ) -> ::core::result::Result<exports::test::host::types::Shape, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
        let wave = proxy::recorder::replay::replay_import(
                Some("test:host/types@0.1.0.check"),
                Some(&__params),
            )
            .unwrap();
        let ret: Value = wasm_wave::from_str(
                &<::core::result::Result<
                    exports::test::host::types::Shape,
                    String,
                > as ValueTyped>::value_type(),
                &wave,
            )
            .unwrap();
        ret.to_rust()
    }
    fn open(
        name: String,
    ) -> ::core::result::Result<exports::test::host::types::Counter, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
        let wave = proxy::recorder::replay::replay_import(
                Some("test:host/types@0.1.0.open"),
                Some(&__params),
            )
            .unwrap();
        let ret: Value = wasm_wave::from_str(
                &<::core::result::Result<
                    exports::test::host::types::Counter,
                    String,
                > as ValueTyped>::value_type(),
                &wave,
            )
            .unwrap();
        ret.to_rust()
    }
}
impl exports::test::host::types::GuestCounter for MockedResource {
    fn new(start: u32) -> Self {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
        let wave = proxy::recorder::replay::replay_import(
                Some("new test:host/types/counter@0.1.0"),
                Some(&__params),
            )
            .unwrap();
        let ret: Value = wasm_wave::from_str(&<Self as ValueTyped>::value_type(), &wave)
            .unwrap();
        MockedResource {
            name: "counter".to_string(),
            ..ret.to_rust()
        }
    }
    fn get(&self) -> u32 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        let wave = proxy::recorder::replay::replay_import(
                Some("test:host/types/counter@0.1.0.get"),
                Some(&__params),
            )
            .unwrap();
        let ret: Value = wasm_wave::from_str(&<u32 as ValueTyped>::value_type(), &wave)
            .unwrap();
        ret.to_rust()
    }
    fn add(&self, n: u32) -> () {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        __params.push(wasm_wave::to_string(&ToValue::to_value(&n)).unwrap());
        let wave = proxy::recorder::replay::replay_import(
            Some("test:host/types/counter@0.1.0.add"),
            Some(&__params),
        );
        assert!(wave.is_none());
    }
    fn merge(
        a: exports::test::host::types::CounterBorrow<'_>,
        b: exports::test::host::types::Counter,
    ) -> exports::test::host::types::Counter {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&a)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&b)).unwrap());
        let __ret = {
            let wave = proxy::recorder::replay::replay_import(
                    Some("test:host/types/counter@0.1.0.merge"),
                    Some(&__params),
                )
                .unwrap();
            let ret: Value = wasm_wave::from_str(
                    &<exports::test::host::types::Counter as ValueTyped>::value_type(),
                    &wave,
                )
                .unwrap();
            ret.to_rust()
        };
        MOVING.with(|moving| moving.set(true));
        drop((b,));
        MOVING.with(|moving| moving.set(false));
        __ret
    }
    unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
        if let Some(res) = *unsafe { Box::from_raw(handle) } {
            if !MOVING.with(|moving| moving.get()) {
                let params = vec![wasm_wave::to_string(& res.to_value()).unwrap()];
                let ret = proxy::recorder::replay::replay_import(
                    Some("drop"),
                    Some(&params),
                );
                assert!(ret.is_none());
            }
        }
        None
    }
}
impl exports::test::log0_1_0::sink::Guest for Stub {
    fn write(msg: String) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&msg)).unwrap());
        let wave = proxy::recorder::replay::replay_import(
            Some("test:log/sink@0.1.0.write"),
            Some(&__params),
        );
        assert!(wave.is_none());
    }
}
impl exports::test::log0_2_0::sink::Guest for Stub {
    fn write(msg: String) -> bool {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&msg)).unwrap());
        let wave = proxy::recorder::replay::replay_import(
                Some("test:log/sink@0.2.0.write"),
                Some(&__params),
            )
            .unwrap();
        let ret: Value = wasm_wave::from_str(&<bool as ValueTyped>::value_type(), &wave)
            .unwrap();
        ret.to_rust()
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
impl ToRust<exports::test::host::types::Point> for Value {
    fn to_rust(&self) -> exports::test::host::types::Point {
        let fields: std::collections::BTreeMap<_, _> = self.unwrap_record().collect();
        exports::test::host::types::Point {
            x: fields["x"].to_rust(),
            y: fields["y"].to_rust(),
            label: fields["label"].to_rust(),
        }
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
impl ToRust<exports::test::host::types::Shape> for Value {
    fn to_rust(&self) -> exports::test::host::types::Shape {
        match self.unwrap_variant() {
            (ref case, Some(val)) if case == "circle" => {
                exports::test::host::types::Shape::Circle(val.to_rust())
            }
            (ref case, Some(val)) if case == "rect" => {
                exports::test::host::types::Shape::Rect(val.to_rust())
            }
            (ref case, Some(val)) if case == "HTTP-error" => {
                exports::test::host::types::Shape::HttpError(val.to_rust())
            }
            (ref case, None) if case == "empty" => {
                exports::test::host::types::Shape::Empty
            }
            _ => unreachable!(),
        }
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
impl ToRust<exports::test::host::types::Color> for Value {
    fn to_rust(&self) -> exports::test::host::types::Color {
        match self.unwrap_variant() {
            (ref case, None) if case == "red" => exports::test::host::types::Color::Red,
            (ref case, None) if case == "green" => {
                exports::test::host::types::Color::Green
            }
            (ref case, None) if case == "DNS-blue" => {
                exports::test::host::types::Color::DnsBlue
            }
            _ => unreachable!(),
        }
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
impl ToRust<exports::test::host::types::Perms> for Value {
    fn to_rust(&self) -> exports::test::host::types::Perms {
        let flags: Vec<_> = self.unwrap_flags().map(|f| f.into_owned()).collect();
        let mut res = exports::test::host::types::Perms::empty();
        if flags.iter().any(|flag| flag == "read") {
            res |= exports::test::host::types::Perms::READ;
        }
        if flags.iter().any(|flag| flag == "write") {
            res |= exports::test::host::types::Perms::WRITE;
        }
        if flags.iter().any(|flag| flag == "exec") {
            res |= exports::test::host::types::Perms::EXEC;
        }
        res
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
impl ToRust<exports::test::host::types::Counter> for Value {
    fn to_rust(&self) -> exports::test::host::types::Counter {
        let label = self.unwrap_handle();
        let expect_handle = label
            .strip_prefix(&format!("{}-", "counter"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        let handle = exports::test::host::types::Counter::new(MockedResource {
            handle: expect_handle,
            name: "counter".to_string(),
        });
        handle
    }
}
impl<'a> ToRust<exports::test::host::types::CounterBorrow<'a>> for Value {
    fn to_rust(&self) -> exports::test::host::types::CounterBorrow<'a> {
        unreachable!()
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
thread_local! {
    #[allow(dead_code)] static MOVING : std::cell::Cell < bool > = const {
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
