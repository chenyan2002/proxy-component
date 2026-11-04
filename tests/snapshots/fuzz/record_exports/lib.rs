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
                    let p: test::guest::api::Point = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
                    let c: test::guest::api::Counter = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:guest/api@0.1.0.run",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::guest::api::run(&p, &c);
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
                    let s: test::guest::api::Shape = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:guest/api@0.1.0.classify",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::guest::api::classify(&s);
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
                    let c: test::guest::api::Counter = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:guest/api@0.1.0.begin",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::guest::api::begin(c);
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
                    let name: String = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "new test:guest/api/session@0.1.0",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::guest::api::Session::new(&name);
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
                    let x: i32 = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
                    let y: i32 = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:host/types@0.1.0.make-point",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::host::types::make_point(x, y);
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
                6usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let p: test::host::types::Perms = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
                    let c: test::host::types::Color = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:host/types@0.1.0.check",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::host::types::check(p, c);
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
                7usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let name: String = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:host/types@0.1.0.open",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::host::types::open(&name);
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
                8usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let start: u32 = Fuzz::fuzz(&mut u).unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "new test:host/types/counter@0.1.0",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::host::types::Counter::new(start);
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
                    let a: test::host::types::Counter = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&a)).unwrap());
                    let b: test::host::types::Counter = Fuzz::fuzz(&mut u).unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&b)).unwrap());
                    let __id = trace_task_id(Table::Exports);
                    let __at = proxy::conversion::executor::step();
                    proxy::recorder::record::record(
                        &Line {
                            at: Some(__at),
                            ..trace_call(
                                Party::Host,
                                __id,
                                "test:host/types/counter@0.1.0.merge",
                                false,
                                __params,
                            )
                        },
                    );
                    let res = test::host::types::Counter::merge(&a, b);
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
        let label = format!(
            "{}-{}", "counter",
            proxy::conversion::conversion::get_handle_mock_test_host_types_counter(self)
        );
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for &'a test::host::types::Counter {
    fn to_value(&self) -> Value {
        let label = format!(
            "borrow-{}-{}", "counter",
            proxy::conversion::conversion::get_handle_mock_test_host_types_counter(self)
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
impl Fuzz for test::guest::api::Session {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok({
            fn cannot_mock<T>() -> T {
                unimplemented!("cannot mock `session`, a resource of the main component")
            }
            let _ = 0;
            cannot_mock()
        })
    }
    fn consume(self, _u: &mut Unstructured<'_>) {
        let name = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
        let line = trace_drop(Party::Host, TRACE_TABLE, name);
        proxy::recorder::record::record(
            &Line {
                at: Some(trace_at()),
                ..line
            },
        );
    }
}
impl<'a> Fuzz for &'a test::guest::api::Session {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                Ok(
                    alloc
                        .alloc({
                            fn cannot_mock<T>() -> T {
                                unimplemented!(
                                    "cannot mock `session`, a resource of the main component"
                                )
                            }
                            let _ = 0;
                            cannot_mock()
                        }),
                )
            })
    }
}
impl Fuzz for test::host::types::Point {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(test::host::types::Point {
            x: Fuzz::fuzz(u)?,
            y: Fuzz::fuzz(u)?,
            label: Fuzz::fuzz(u)?,
        })
    }
    #[allow(unused_variables)]
    fn consume(self, u: &mut Unstructured<'_>) {
        Fuzz::consume(self.x, u);
        Fuzz::consume(self.y, u);
        Fuzz::consume(self.label, u);
    }
}
impl Fuzz for test::host::types::Shape {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(4usize)?;
        match idx {
            0usize => Ok(test::host::types::Shape::Circle(Fuzz::fuzz(u)?)),
            1usize => Ok(test::host::types::Shape::Rect(Fuzz::fuzz(u)?)),
            2usize => Ok(test::host::types::Shape::HttpError(Fuzz::fuzz(u)?)),
            3usize => Ok(test::host::types::Shape::Empty),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables)]
    fn consume(self, u: &mut Unstructured<'_>) {
        match self {
            test::host::types::Shape::Circle(x) => Fuzz::consume(x, u),
            test::host::types::Shape::Rect(x) => Fuzz::consume(x, u),
            test::host::types::Shape::HttpError(x) => Fuzz::consume(x, u),
            test::host::types::Shape::Empty => {}
        }
    }
}
impl Fuzz for test::host::types::Color {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(3usize)?;
        match idx {
            0usize => Ok(test::host::types::Color::Red),
            1usize => Ok(test::host::types::Color::Green),
            2usize => Ok(test::host::types::Color::DnsBlue),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables)]
    fn consume(self, u: &mut Unstructured<'_>) {
        match self {
            test::host::types::Color::Red => {}
            test::host::types::Color::Green => {}
            test::host::types::Color::DnsBlue => {}
        }
    }
}
impl Fuzz for test::host::types::Perms {
    fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
        let flag_count = u.int_in_range(0..=2usize)?;
        let mut res = test::host::types::Perms::empty();
        let choices = [
            test::host::types::Perms::READ,
            test::host::types::Perms::WRITE,
            test::host::types::Perms::EXEC,
        ];
        for _ in 0..flag_count {
            let flag = u.choose(&choices)?;
            res |= *flag;
        }
        Ok(res)
    }
}
impl Fuzz for test::host::types::Counter {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(proxy::conversion::conversion::get_mock_test_host_types_counter(0))
    }
}
impl<'a> Fuzz for &'a test::host::types::Counter {
    fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                Ok(
                    alloc
                        .alloc(
                            proxy::conversion::conversion::get_mock_test_host_types_counter(
                                0,
                            ),
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
