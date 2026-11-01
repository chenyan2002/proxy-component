mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    async fn start() -> () {
        let __executor = wit_bindgen::spawn_local(proxy::conversion::executor::run());
        loop {
            let idx = proxy::util::dialog::read_select(
                0,
                "Select an export function to call",
                &[
                    "All done".to_string(),
                    "test:guest/api@0.1.0.run".to_string(),
                    "test:guest/api@0.1.0.classify".to_string(),
                    "test:guest/api@0.1.0.begin".to_string(),
                    "new test:guest/api/session@0.1.0".to_string(),
                    "test:host/types@0.1.0.make-point".to_string(),
                    "test:host/types@0.1.0.check".to_string(),
                    "test:host/types@0.1.0.open".to_string(),
                    "new test:host/types/counter@0.1.0".to_string(),
                    "test:host/types/counter@0.1.0.merge".to_string(),
                ],
            ) as usize;
            match idx {
                0 => break,
                1usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:guest/api@0.1.0.run"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(p), "point"),
                    );
                    let p: test::guest::api::Point = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(c),
                            "borrow<counter>"
                        ),
                    );
                    let c: test::guest::api::Counter = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:guest/api@0.1.0.run", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::run(&p, &c);
                }
                2usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:guest/api@0.1.0.classify"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(s), "shape"),
                    );
                    let s: test::guest::api::Shape = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:guest/api@0.1.0.classify", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::classify(&s);
                }
                3usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:guest/api@0.1.0.begin"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(c), "counter"),
                    );
                    let c: test::guest::api::Counter = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:guest/api@0.1.0.begin", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::guest::api::begin(c);
                }
                4usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "call export func {}", "new test:guest/api/session@0.1.0"
                        ),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(name), "string"
                        ),
                    );
                    let name: String = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "new test:guest/api/session@0.1.0",
                            __params.join(", ")
                        ),
                    );
                    let _ = test::guest::api::Session::new(&name);
                }
                5usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "call export func {}", "test:host/types@0.1.0.make-point"
                        ),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(x), "s32"),
                    );
                    let x: i32 = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(y), "s32"),
                    );
                    let y: i32 = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:host/types@0.1.0.make-point",
                            __params.join(", ")
                        ),
                    );
                    let _ = test::host::types::make_point(x, y);
                }
                6usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:host/types@0.1.0.check"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(p), "perms"),
                    );
                    let p: test::host::types::Perms = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(c), "color"),
                    );
                    let c: test::host::types::Color = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:host/types@0.1.0.check", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::host::types::check(p, c);
                }
                7usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:host/types@0.1.0.open"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(name), "string"
                        ),
                    );
                    let name: String = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:host/types@0.1.0.open", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::host::types::open(&name);
                }
                8usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "call export func {}", "new test:host/types/counter@0.1.0"
                        ),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(start), "u32"),
                    );
                    let start: u32 = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "new test:host/types/counter@0.1.0",
                            __params.join(", ")
                        ),
                    );
                    let _ = test::host::types::Counter::new(start);
                }
                9usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "call export func {}", "test:host/types/counter@0.1.0.merge"
                        ),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(a),
                            "borrow<counter>"
                        ),
                    );
                    let a: test::host::types::Counter = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&a)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(b), "counter"),
                    );
                    let b: test::host::types::Counter = Dialog::read_value(0);
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&b)).unwrap());
                    proxy::util::dialog::print(
                        0,
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
impl Dialog for test::guest::api::Session {
    fn read_value(_dep: u32) -> Self {
        {
            fn cannot_mock<T>() -> T {
                unimplemented!("cannot mock `session`, a resource of the main component")
            }
            let _ = 0;
            cannot_mock()
        }
    }
}
impl<'a> Dialog for &'a test::guest::api::Session {
    fn read_value(_dep: u32) -> Self {
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
                        let _ = 0;
                        cannot_mock()
                    })
            })
    }
}
impl Dialog for test::host::types::Point {
    fn read_value(dep: u32) -> Self {
        proxy::util::dialog::print(
            dep,
            &format!("provide value for struct {}", "test:host/types@0.1.0.point"),
        );
        proxy::util::dialog::print(
            dep + 1,
            &format!("provide value for field {}: {}", "x", "s32"),
        );
        let x = Dialog::read_value(dep + 1);
        proxy::util::dialog::print(
            dep + 1,
            &format!("provide value for field {}: {}", "y", "s32"),
        );
        let y = Dialog::read_value(dep + 1);
        proxy::util::dialog::print(
            dep + 1,
            &format!("provide value for field {}: {}", "label", "option<string>"),
        );
        let label = Dialog::read_value(dep + 1);
        Self { x, y, label }
    }
}
impl Dialog for test::host::types::Shape {
    fn read_value(dep: u32) -> Self {
        let idx = proxy::util::dialog::read_select(
            dep,
            &format!("Select a variant for {}", "test:host/types@0.1.0.shape"),
            &[
                "circle".to_string(),
                "rect".to_string(),
                "HTTP-error".to_string(),
                "empty".to_string(),
            ],
        ) as usize;
        match idx {
            0usize => test::host::types::Shape::Circle(Dialog::read_value(dep + 1)),
            1usize => test::host::types::Shape::Rect(Dialog::read_value(dep + 1)),
            2usize => test::host::types::Shape::HttpError(Dialog::read_value(dep + 1)),
            3usize => test::host::types::Shape::Empty,
            _ => unreachable!(),
        }
    }
}
impl Dialog for test::host::types::Color {
    fn read_value(dep: u32) -> Self {
        let idx = proxy::util::dialog::read_select(
            dep,
            &format!("Select a variant for {}", "test:host/types@0.1.0.color"),
            &["red".to_string(), "green".to_string(), "DNS-blue".to_string()],
        ) as usize;
        match idx {
            0usize => test::host::types::Color::Red,
            1usize => test::host::types::Color::Green,
            2usize => test::host::types::Color::DnsBlue,
            _ => unreachable!(),
        }
    }
}
impl Dialog for test::host::types::Perms {
    fn read_value(dep: u32) -> Self {
        let selections = proxy::util::dialog::read_multi_select(
            dep,
            &format!("Select flags for {}", "test:host/types@0.1.0.perms"),
            &["read".to_string(), "write".to_string(), "exec".to_string()],
        );
        let mut res = test::host::types::Perms::empty();
        for idx in selections {
            match idx as usize {
                0usize => res |= test::host::types::Perms::READ,
                1usize => res |= test::host::types::Perms::WRITE,
                2usize => res |= test::host::types::Perms::EXEC,
                _ => unreachable!(),
            }
        }
        res
    }
}
impl Dialog for test::host::types::Counter {
    fn read_value(_dep: u32) -> Self {
        proxy::conversion::conversion::get_mock_test_host_types_counter(0)
    }
}
impl<'a> Dialog for &'a test::host::types::Counter {
    fn read_value(_dep: u32) -> Self {
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                alloc
                    .alloc(
                        proxy::conversion::conversion::get_mock_test_host_types_counter(
                            0,
                        ),
                    )
            })
    }
}
trait Dialog {
    fn read_value(dep: u32) -> Self;
}
impl Dialog for () {
    fn read_value(_dep: u32) -> Self {
        ()
    }
}
impl<T: Dialog> Dialog for Option<T> {
    fn read_value(dep: u32) -> Self {
        let selection = proxy::util::dialog::read_select(
            dep,
            "Select an option tag",
            &["none".to_string(), "some".to_string()],
        );
        if selection == 0 { None } else { Some(Dialog::read_value(dep + 1)) }
    }
}
impl<T: Dialog + 'static> Dialog for Vec<T> {
    fn read_value(dep: u32) -> Self {
        use std::any::TypeId;
        if TypeId::of::<T>() == TypeId::of::<u8>() {
            let hex = proxy::util::dialog::read_raw_string(
                dep,
                "Enter a string as list<u8>",
            );
            let bytes = hex.into_bytes();
            unsafe { std::mem::transmute::<Vec<u8>, Vec<T>>(bytes) }
        } else {
            let len = proxy::util::dialog::read_num(dep, "Enter the length of the list");
            (0..len).map(|_| Dialog::read_value(dep + 1)).collect()
        }
    }
}
impl<O: Dialog, E: Dialog> Dialog for Result<O, E> {
    fn read_value(dep: u32) -> Self {
        let selection = proxy::util::dialog::read_select(
            dep,
            "Select result",
            &["ok".to_string(), "err".to_string()],
        );
        if selection == 0 {
            Ok(Dialog::read_value(dep + 1))
        } else {
            Err(Dialog::read_value(dep + 1))
        }
    }
}
impl Dialog for MockedResource {
    fn read_value(_dep: u32) -> Self {
        Self {
            handle: 0,
            name: "mocked-resource".to_string(),
        }
    }
}
macro_rules! impl_dialog_primitive {
    ($($ty:ty => $read_fn:ident),* $(,)?) => {
        $(impl Dialog for $ty { fn read_value(dep : u32) -> Self { let wave =
        proxy::util::dialog:: $read_fn (dep); let ret : Value = wasm_wave::from_str(& <
        Self as ValueTyped > ::value_type(), & wave).unwrap(); ret.to_rust() } })*
    };
}
impl_dialog_primitive! {
    bool => read_bool, u8 => read_u8, u16 => read_u16, u32 => read_u32, u64 => read_u64,
    i8 => read_s8, i16 => read_s16, i32 => read_s32, i64 => read_s64, f32 => read_f32,
    f64 => read_f64, char => read_char, String => read_string,
}
macro_rules! impl_dialog_tuple {
    ($($T:ident),+) => {
        impl < $($T : Dialog),+ > Dialog for ($($T,)+) { fn read_value(dep : u32) -> Self
        { ($($T ::read_value(dep + 1),)+) } }
    };
}
impl_dialog_tuple!(T1);
impl_dialog_tuple!(T1, T2);
impl_dialog_tuple!(T1, T2, T3);
impl_dialog_tuple!(T1, T2, T3, T4);
impl_dialog_tuple!(T1, T2, T3, T4, T5);
impl_dialog_tuple!(T1, T2, T3, T4, T5, T6);
impl_dialog_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_dialog_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
thread_local! {
    #[allow(dead_code)] static PENDING_RET : std::cell::Cell < bool > = const {
    std::cell::Cell::new(false) };
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
