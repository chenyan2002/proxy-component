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
        while let Some((method, args)) = proxy::recorder::replay::replay_export() {
            match method.as_str() {
                "test:guest/api@0.1.0.run" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::guest::api::Point as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let p: test::guest::api::Point = arg_value.to_rust();
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::guest::api::Counter as ValueTyped>::value_type(),
                            &args[1usize],
                        )
                        .unwrap();
                    let c: test::guest::api::Counter = arg_value.to_rust();
                    let res = test::guest::api::run(&p, &c);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:guest/api@0.1.0.run"),
                        Some(&wave_res),
                    );
                }
                "test:guest/api@0.1.0.classify" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::guest::api::Shape as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let s: test::guest::api::Shape = arg_value.to_rust();
                    let res = test::guest::api::classify(&s);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:guest/api@0.1.0.classify"),
                        Some(&wave_res),
                    );
                }
                "test:guest/api@0.1.0.begin" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::guest::api::Counter as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let c: test::guest::api::Counter = arg_value.to_rust();
                    let res = test::guest::api::begin(c);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:guest/api@0.1.0.begin"),
                        Some(&wave_res),
                    );
                    std::mem::forget(res);
                }
                "new test:guest/api/session@0.1.0" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<String as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let name: String = arg_value.to_rust();
                    let res = test::guest::api::Session::new(&name);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("new test:guest/api/session@0.1.0"),
                        Some(&wave_res),
                    );
                    std::mem::forget(res);
                }
                "test:host/types@0.1.0.make-point" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<i32 as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let x: i32 = arg_value.to_rust();
                    let arg_value: Value = wasm_wave::from_str(
                            &<i32 as ValueTyped>::value_type(),
                            &args[1usize],
                        )
                        .unwrap();
                    let y: i32 = arg_value.to_rust();
                    let res = test::host::types::make_point(x, y);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:host/types@0.1.0.make-point"),
                        Some(&wave_res),
                    );
                }
                "test:host/types@0.1.0.check" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::host::types::Perms as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let p: test::host::types::Perms = arg_value.to_rust();
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::host::types::Color as ValueTyped>::value_type(),
                            &args[1usize],
                        )
                        .unwrap();
                    let c: test::host::types::Color = arg_value.to_rust();
                    let res = test::host::types::check(p, c);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:host/types@0.1.0.check"),
                        Some(&wave_res),
                    );
                }
                "test:host/types@0.1.0.open" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<String as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let name: String = arg_value.to_rust();
                    let res = test::host::types::open(&name);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:host/types@0.1.0.open"),
                        Some(&wave_res),
                    );
                    std::mem::forget(res);
                }
                "new test:host/types/counter@0.1.0" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<u32 as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let start: u32 = arg_value.to_rust();
                    let res = test::host::types::Counter::new(start);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("new test:host/types/counter@0.1.0"),
                        Some(&wave_res),
                    );
                    std::mem::forget(res);
                }
                "test:host/types/counter@0.1.0.merge" => {
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::host::types::Counter as ValueTyped>::value_type(),
                            &args[0usize],
                        )
                        .unwrap();
                    let a: test::host::types::Counter = arg_value.to_rust();
                    let arg_value: Value = wasm_wave::from_str(
                            &<test::host::types::Counter as ValueTyped>::value_type(),
                            &args[1usize],
                        )
                        .unwrap();
                    let b: test::host::types::Counter = arg_value.to_rust();
                    let res = test::host::types::Counter::merge(&a, b);
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(
                        Some("test:host/types/counter@0.1.0.merge"),
                        Some(&wave_res),
                    );
                    std::mem::forget(res);
                }
                "drop" => {
                    let (name, handle) = args[0]
                        .strip_prefix('#')
                        .and_then(|label| label.rsplit_once('-'))
                        .and_then(|(name, handle)| Some((
                            name,
                            handle.parse::<u32>().ok()?,
                        )))
                        .expect("invalid handle label");
                    match name {
                        "session" => {
                            drop(unsafe {
                                test::guest::api::Session::from_handle(handle)
                            })
                        }
                        _ => unreachable!(),
                    }
                    proxy::recorder::replay::assert_export_ret(Some("drop"), None);
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
impl ValueTyped for test::guest::api::Session {
    fn value_type() -> Type {
        Type::handle("session")
    }
}
impl<'a> ValueTyped for &'a test::guest::api::Session {
    fn value_type() -> Type {
        Type::handle("session")
    }
}
impl ToValue for test::guest::api::Session {
    fn to_value(&self) -> Value {
        let label = format!("{}-{}", "session", self.handle());
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for &'a test::guest::api::Session {
    fn to_value(&self) -> Value {
        let label = format!("borrow-{}-{}", "session", self.handle());
        Value::make_handle(label.into())
    }
}
impl ToRust<test::guest::api::Session> for Value {
    fn to_rust(&self) -> test::guest::api::Session {
        let label = self.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("{}-", "session"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        {
            fn cannot_mock<T>() -> T {
                unimplemented!("cannot mock `session`, a resource of the main component")
            }
            let _ = handle;
            cannot_mock()
        }
    }
}
impl<'a> ToRust<&'a test::guest::api::Session> for Value {
    fn to_rust(&self) -> &'a test::guest::api::Session {
        let label = self.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("borrow-{}-", "session"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                alloc
                    .alloc({
                        fn cannot_mock<T>() -> T {
                            unimplemented!(
                                "cannot mock `session`, a resource of the main component"
                            )
                        }
                        let _ = handle;
                        cannot_mock()
                    })
            })
    }
}
impl ValueTyped for test::host::types::Point {
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
impl ToValue for test::host::types::Point {
    fn to_value(&self) -> Value {
        let ty = test::host::types::Point::value_type();
        let fields = vec![
            ("x", self.x.to_value()), ("y", self.y.to_value()), ("label", self.label
            .to_value())
        ];
        Value::make_record(&ty, fields).unwrap()
    }
}
impl ToRust<test::host::types::Point> for Value {
    fn to_rust(&self) -> test::host::types::Point {
        let fields: std::collections::BTreeMap<_, _> = self.unwrap_record().collect();
        test::host::types::Point {
            x: fields["x"].to_rust(),
            y: fields["y"].to_rust(),
            label: fields["label"].to_rust(),
        }
    }
}
impl ValueTyped for test::host::types::Shape {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let cases = vec![
            ("circle", Some(< u32 as ValueTyped > ::value_type())), ("rect", Some(<
            test::host::types::Point as ValueTyped > ::value_type())), ("HTTP-error",
            Some(< String as ValueTyped > ::value_type())), ("empty", None)
        ];
        Type::variant(cases).unwrap()
    }
}
impl ToValue for test::host::types::Shape {
    fn to_value(&self) -> Value {
        let ty = test::host::types::Shape::value_type();
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
impl ToRust<test::host::types::Shape> for Value {
    fn to_rust(&self) -> test::host::types::Shape {
        match self.unwrap_variant() {
            (ref case, Some(val)) if case == "circle" => {
                test::host::types::Shape::Circle(val.to_rust())
            }
            (ref case, Some(val)) if case == "rect" => {
                test::host::types::Shape::Rect(val.to_rust())
            }
            (ref case, Some(val)) if case == "HTTP-error" => {
                test::host::types::Shape::HttpError(val.to_rust())
            }
            (ref case, None) if case == "empty" => test::host::types::Shape::Empty,
            _ => unreachable!(),
        }
    }
}
impl ValueTyped for test::host::types::Color {
    #[allow(unused_imports)]
    fn value_type() -> Type {
        let cases = vec![("red", None), ("green", None), ("DNS-blue", None)];
        Type::variant(cases).unwrap()
    }
}
impl ToValue for test::host::types::Color {
    fn to_value(&self) -> Value {
        let ty = test::host::types::Color::value_type();
        match self {
            Self::Red => Value::make_variant(&ty, "red", None),
            Self::Green => Value::make_variant(&ty, "green", None),
            Self::DnsBlue => Value::make_variant(&ty, "DNS-blue", None),
        }
            .unwrap()
    }
}
impl ToRust<test::host::types::Color> for Value {
    fn to_rust(&self) -> test::host::types::Color {
        match self.unwrap_variant() {
            (ref case, None) if case == "red" => test::host::types::Color::Red,
            (ref case, None) if case == "green" => test::host::types::Color::Green,
            (ref case, None) if case == "DNS-blue" => test::host::types::Color::DnsBlue,
            _ => unreachable!(),
        }
    }
}
impl ValueTyped for test::host::types::Perms {
    fn value_type() -> Type {
        Type::flags(vec!["read", "write", "exec"]).unwrap()
    }
}
impl ToValue for test::host::types::Perms {
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
impl ToRust<test::host::types::Perms> for Value {
    fn to_rust(&self) -> test::host::types::Perms {
        let flags: Vec<_> = self.unwrap_flags().map(|f| f.into_owned()).collect();
        let mut res = test::host::types::Perms::empty();
        if flags.iter().any(|flag| flag == "read") {
            res |= test::host::types::Perms::READ;
        }
        if flags.iter().any(|flag| flag == "write") {
            res |= test::host::types::Perms::WRITE;
        }
        if flags.iter().any(|flag| flag == "exec") {
            res |= test::host::types::Perms::EXEC;
        }
        res
    }
}
impl ValueTyped for test::host::types::Counter {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl<'a> ValueTyped for &'a test::host::types::Counter {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl ToValue for test::host::types::Counter {
    fn to_value(&self) -> Value {
        let label = format!("{}-{}", "counter", self.handle());
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for &'a test::host::types::Counter {
    fn to_value(&self) -> Value {
        let label = format!("borrow-{}-{}", "counter", self.handle());
        Value::make_handle(label.into())
    }
}
impl ToRust<test::host::types::Counter> for Value {
    fn to_rust(&self) -> test::host::types::Counter {
        let label = self.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("{}-", "counter"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        proxy::conversion::conversion::get_mock_test_host_types_counter(handle)
    }
}
impl<'a> ToRust<&'a test::host::types::Counter> for Value {
    fn to_rust(&self) -> &'a test::host::types::Counter {
        let label = self.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("borrow-{}-", "counter"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                alloc
                    .alloc(
                        proxy::conversion::conversion::get_mock_test_host_types_counter(
                            handle,
                        ),
                    )
            })
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
