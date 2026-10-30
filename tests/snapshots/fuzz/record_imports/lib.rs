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
impl exports::test::host::types::Guest for Stub {
    type Counter = MockedResource;
    fn make_point(x: i32, y: i32) -> exports::test::host::types::Point {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!("import: {}({})", "test:host/types@0.1.0.make-point", __buf),
        );
        __buf += "test:host/types@0.1.0.make-point";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = u.arbitrary().unwrap();
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
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!("import: {}({})", "test:host/types@0.1.0.check", __buf),
        );
        __buf += "test:host/types@0.1.0.check";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = u.arbitrary().unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    fn open(
        name: String,
    ) -> ::core::result::Result<exports::test::host::types::Counter, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!("import: {}({})", "test:host/types@0.1.0.open", __buf),
        );
        __buf += "test:host/types@0.1.0.open";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = u.arbitrary().unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
}
impl exports::test::host::types::GuestCounter for MockedResource {
    fn new(start: u32) -> Self {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "[constructor]test:host/types/counter@0.1.0.new", __buf
            ),
        );
        __buf += "[constructor]test:host/types/counter@0.1.0.new";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = MockedResource {
            name: "counter".to_string(),
            ..u.arbitrary().unwrap()
        };
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
    }
    fn get(&self) -> u32 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!(
                "import: {}({})", "[method]test:host/types/counter@0.1.0.get", __buf
            ),
        );
        __buf += "[method]test:host/types/counter@0.1.0.get";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = u.arbitrary().unwrap();
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
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!("import: {}({})", "test:host/types/counter@0.1.0.merge", __buf),
        );
        __buf += "test:host/types/counter@0.1.0.merge";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = u.arbitrary().unwrap();
        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
        proxy::util::debug::print(&format!("ret: {}", res_str));
        res
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
        let mut __buf = __params.join(",");
        proxy::util::debug::print(
            &format!("import: {}({})", "test:log/sink@0.2.0.write", __buf),
        );
        __buf += "test:log/sink@0.2.0.write";
        let mut u = Unstructured::new(&__buf.as_bytes());
        let res = u.arbitrary().unwrap();
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
impl Arbitrary<'_> for exports::test::host::types::Point {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(exports::test::host::types::Point {
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
impl Arbitrary<'_> for exports::test::host::types::Shape {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(4usize)?;
        match idx {
            0usize => Ok(exports::test::host::types::Shape::Circle(u.arbitrary()?)),
            1usize => Ok(exports::test::host::types::Shape::Rect(u.arbitrary()?)),
            2usize => Ok(exports::test::host::types::Shape::HttpError(u.arbitrary()?)),
            3usize => Ok(exports::test::host::types::Shape::Empty),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let mut res = (1, Some(1));
        let size = <u32 as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::or(res, size);
        let size = <exports::test::host::types::Point as Arbitrary>::size_hint(
            depth + 1,
        );
        res = arbitrary::size_hint::or(res, size);
        let size = <String as Arbitrary>::size_hint(depth + 1);
        res = arbitrary::size_hint::or(res, size);
        res
    }
}
impl Arbitrary<'_> for exports::test::host::types::Color {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let idx = u.choose_index(3usize)?;
        match idx {
            0usize => Ok(exports::test::host::types::Color::Red),
            1usize => Ok(exports::test::host::types::Color::Green),
            2usize => Ok(exports::test::host::types::Color::DnsBlue),
            _ => unreachable!(),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let mut res = (1, Some(1));
        res
    }
}
impl Arbitrary<'_> for exports::test::host::types::Perms {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
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
    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1 + 2usize))
    }
}
impl Arbitrary<'_> for exports::test::host::types::Counter {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(
            exports::test::host::types::Counter::new(MockedResource {
                handle: 42,
                name: "counter".to_string(),
            }),
        )
    }
    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<'a> Arbitrary<'_> for exports::test::host::types::CounterBorrow<'a> {
    fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
        unreachable!()
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
