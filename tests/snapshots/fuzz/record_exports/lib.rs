mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    fn start() -> () {
        let __buf = proxy::util::debug::get_random();
        let mut u = Unstructured::new(&__buf);
        for _ in 0..10 {
            let idx = u.int_in_range(1..=9usize).unwrap();
            match idx {
                1usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let p: test::guest::api::Point = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
                    let c: test::guest::api::Counter = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:guest/api@0.1.0.run", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::run(&p, &c);
                }
                2usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let s: test::guest::api::Shape = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:guest/api@0.1.0.classify", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::classify(&s);
                }
                3usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let c: test::guest::api::Counter = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:guest/api@0.1.0.begin", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::begin(c);
                }
                4usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let name: String = u.arbitrary().unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})",
                            "[constructor]test:guest/api/session@0.1.0.new", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::Session::new(&name);
                }
                5usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let x: i32 = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
                    let y: i32 = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:host/types@0.1.0.make-point",
                            __params.join(", ")
                        ),
                    );
                    let _ = test::host::types::make_point(x, y);
                }
                6usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let p: test::host::types::Perms = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
                    let c: test::host::types::Color = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:host/types@0.1.0.check", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::host::types::check(p, c);
                }
                7usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let name: String = u.arbitrary().unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:host/types@0.1.0.open", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::host::types::open(&name);
                }
                8usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let start: u32 = u.arbitrary().unwrap();
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})",
                            "[constructor]test:host/types/counter@0.1.0.new", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::host::types::Counter::new(start);
                }
                9usize => {
                    let mut __params: Vec<String> = Vec::new();
                    let a: test::host::types::Counter = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&a)).unwrap());
                    let b: test::host::types::Counter = u.arbitrary().unwrap();
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&b)).unwrap());
                    proxy::util::debug::print(
                        &format!(
                            "export: {}({})", "test:host/types/counter@0.1.0.merge",
                            __params.join(", ")
                        ),
                    );
                    let _ = test::host::types::Counter::merge(&a, b);
                }
                _ => unreachable!(),
            }
            SCOPED_ALLOC
                .with(|alloc| {
                    alloc.borrow_mut().clear();
                });
        }
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
impl Arbitrary<'_> for test::guest::api::Session {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok({
            fn cannot_mock<T>() -> T {
                unimplemented!("cannot mock `session`, a resource of the main component")
            }
            let _ = 42;
            cannot_mock()
        })
    }
    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<'a> Arbitrary<'_> for &'a test::guest::api::Session {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
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
                            let _ = 42;
                            cannot_mock()
                        }),
                )
            })
    }
}
impl Arbitrary<'_> for test::host::types::Point {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(test::host::types::Point {
            x: u.arbitrary()?,
            y: u.arbitrary()?,
            label: u.arbitrary()?,
        })
    }
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let mut res = (0, Some(0));
        let size = <i32 as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::and(res, size);
        let size = <i32 as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::and(res, size);
        let size = <::core::option::Option<String> as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::and(res, size);
        res
    }
}
impl Arbitrary<'_> for test::host::types::Shape {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(4usize)?;
        match idx {
            0usize => Ok(test::host::types::Shape::Circle(u.arbitrary()?)),
            1usize => Ok(test::host::types::Shape::Rect(u.arbitrary()?)),
            2usize => Ok(test::host::types::Shape::HttpError(u.arbitrary()?)),
            3usize => Ok(test::host::types::Shape::Empty),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let mut res = (1, Some(1));
        let size = <u32 as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::or(res, size);
        let size = <test::host::types::Point as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::or(res, size);
        let size = <String as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::or(res, size);
        res
    }
}
impl Arbitrary<'_> for test::host::types::Color {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(3usize)?;
        match idx {
            0usize => Ok(test::host::types::Color::Red),
            1usize => Ok(test::host::types::Color::Green),
            2usize => Ok(test::host::types::Color::DnsBlue),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let mut res = (1, Some(1));
        res
    }
}
impl Arbitrary<'_> for test::host::types::Perms {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
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
    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1 + 2usize))
    }
}
impl Arbitrary<'_> for test::host::types::Counter {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(proxy::conversion::conversion::get_mock_test_host_types_counter(42))
    }
    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<'a> Arbitrary<'_> for &'a test::host::types::Counter {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                Ok(
                    alloc
                        .alloc(
                            proxy::conversion::conversion::get_mock_test_host_types_counter(
                                42,
                            ),
                        ),
                )
            })
    }
}
#[allow(unused_imports)]
use arbitrary::{Arbitrary, Unstructured, Result};
impl Arbitrary<'_> for MockedResource {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self {
            handle: 42,
            name: "mocked-resource".to_string(),
        })
    }
    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
