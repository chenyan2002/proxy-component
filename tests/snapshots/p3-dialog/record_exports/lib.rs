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
                    "test:p3/api@0.1.0.handle".to_string(),
                    "test:p3/api@0.1.0.serve".to_string(),
                    "test:p3/api@0.1.0.echo".to_string(),
                    "test:p3/host@0.1.0.fetch".to_string(),
                    "test:p3/host@0.1.0.sleep".to_string(),
                    "test:p3/host@0.1.0.report".to_string(),
                    "test:p3/host@0.1.0.finish".to_string(),
                    "new test:p3/host/body@0.1.0".to_string(),
                    "test:p3/host/body@0.1.0.consume".to_string(),
                ],
            ) as usize;
            match idx {
                0 => break,
                1usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/api@0.1.0.handle"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(url), "string"
                        ),
                    );
                    let url: String = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/api@0.1.0.handle", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::api::handle(url).await;
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                2usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/api@0.1.0.serve"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(port), "u16"),
                    );
                    let port: u16 = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&port)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/api@0.1.0.serve", __params
                            .join(", ")
                        ),
                    );
                    Dialog::consume(test::p3::api::serve(port).await, 0);
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                3usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/api@0.1.0.echo"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(s), "stream<u8>"
                        ),
                    );
                    let s: wit_bindgen::rt::async_support::StreamReader<u8> = Dialog::read_value(
                        0,
                    );
                    __params.push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/api@0.1.0.echo", __params
                            .join(", ")
                        ),
                    );
                    Dialog::consume(test::p3::api::echo(s), 0);
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                4usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/host@0.1.0.fetch"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(url), "string"
                        ),
                    );
                    let url: String = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&url)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.fetch", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::fetch(url).await;
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                5usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/host@0.1.0.sleep"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(ms), "u64"),
                    );
                    let ms: u64 = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&ms)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.sleep", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::sleep(ms).await;
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                6usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/host@0.1.0.report"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(err),
                            "error-context"
                        ),
                    );
                    let err: wit_bindgen::rt::async_support::ErrorContext = Dialog::read_value(
                        0,
                    );
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&err)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.report", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::report(err);
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                7usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "test:p3/host@0.1.0.finish"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(done),
                            "future<result<_, error-code>>"
                        ),
                    );
                    let done: wit_bindgen::rt::async_support::FutureReader<
                        ::core::result::Result<(), test::p3::host::ErrorCode>,
                    > = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&done)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/host@0.1.0.finish", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::finish(done);
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                8usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!("call export func {}", "new test:p3/host/body@0.1.0"),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "provide argument for {}: {}", stringify!(contents),
                            "option<stream<u8>>"
                        ),
                    );
                    let contents: ::core::option::Option<
                        wit_bindgen::rt::async_support::StreamReader<u8>,
                    > = Dialog::read_value(0);
                    __params
                        .push(
                            wasm_wave::to_string(&ToValue::to_value(&contents)).unwrap(),
                        );
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "new test:p3/host/body@0.1.0", __params
                            .join(", ")
                        ),
                    );
                    let _ = test::p3::host::Body::new(contents);
                    for _ in 0..8 {
                        wit_bindgen::yield_async().await;
                    }
                }
                9usize => {
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "call export func {}", "test:p3/host/body@0.1.0.consume"
                        ),
                    );
                    let mut __params: Vec<String> = Vec::new();
                    proxy::util::dialog::print(
                        0,
                        &format!("provide argument for {}: {}", stringify!(this), "body"),
                    );
                    let this: test::p3::host::Body = Dialog::read_value(0);
                    __params
                        .push(wasm_wave::to_string(&ToValue::to_value(&this)).unwrap());
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "export: {}({})", "test:p3/host/body@0.1.0.consume", __params
                            .join(", ")
                        ),
                    );
                    Dialog::consume(test::p3::host::Body::consume(this), 0);
                    for _ in 0..8 {
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
        join_spawned().await;
        proxy::conversion::executor::stop();
        __executor.await;
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
impl ToRust<test::p3::host::ErrorCode> for Value {
    fn to_rust(&self) -> test::p3::host::ErrorCode {
        match self.unwrap_variant() {
            (ref case, None) if case == "timeout" => test::p3::host::ErrorCode::Timeout,
            (ref case, Some(val)) if case == "other" => {
                test::p3::host::ErrorCode::Other(val.to_rust())
            }
            _ => unreachable!(),
        }
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
impl ToRust<test::p3::host::Body> for Value {
    fn to_rust(&self) -> test::p3::host::Body {
        let label = self.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("{}-", "body"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        proxy::conversion::conversion::get_mock_test_p3_host_body(handle)
    }
}
impl<'a> ToRust<&'a test::p3::host::Body> for Value {
    fn to_rust(&self) -> &'a test::p3::host::Body {
        let label = self.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("borrow-{}-", "body"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                alloc
                    .alloc(
                        proxy::conversion::conversion::get_mock_test_p3_host_body(handle),
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
impl Dialog for test::p3::host::ErrorCode {
    fn read_value(dep: u32) -> Self {
        let idx = proxy::util::dialog::read_select(
            dep,
            &format!("Select a variant for {}", "test:p3/host@0.1.0.error-code"),
            &["timeout".to_string(), "other".to_string()],
        ) as usize;
        match idx {
            0usize => test::p3::host::ErrorCode::Timeout,
            1usize => test::p3::host::ErrorCode::Other(Dialog::read_value(dep + 1)),
            _ => unreachable!(),
        }
    }
}
impl Dialog for test::p3::host::Body {
    fn read_value(_dep: u32) -> Self {
        proxy::conversion::conversion::get_mock_test_p3_host_body(0)
    }
}
impl<'a> Dialog for &'a test::p3::host::Body {
    fn read_value(_dep: u32) -> Self {
        SCOPED_ALLOC
            .with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                alloc.alloc(proxy::conversion::conversion::get_mock_test_p3_host_body(0))
            })
    }
}
trait Dialog: Sized {
    fn read_value(dep: u32) -> Self;
    #[allow(dead_code)]
    fn consume(self, _dep: u32) {}
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
    fn consume(self, dep: u32) {
        if let Some(x) = self {
            x.consume(dep);
        }
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
    fn consume(self, dep: u32) {
        for x in self {
            x.consume(dep);
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
    fn consume(self, dep: u32) {
        match self {
            Ok(x) => x.consume(dep),
            Err(x) => x.consume(dep),
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
        { ($($T ::read_value(dep + 1),)+) } #[allow(non_snake_case)] fn consume(self, dep
        : u32) { let ($($T,)+) = self; $($T .consume(dep);)+ } }
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
impl<T> Dialog for wit_bindgen::StreamReader<T>
where
    T: Dialog + ToValue + ValueTyped + wit_stream::StreamPayload + 'static,
{
    fn read_value(_dep: u32) -> Self {
        let (mut writer, reader) = wit_stream::new::<T>();
        let label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        spawn(async move {
            let choices = ["write a chunk".to_string(), "end the stream".to_string()];
            loop {
                let prompt = format!("The host writes {label}");
                if proxy::util::dialog::read_select(0, &prompt, &choices) == 1 {
                    proxy::util::dialog::print(0, &format!("{label}.read() = dropped"));
                    return;
                }
                let chunk: Vec<T> = Dialog::read_value(1);
                let items = wasm_wave::to_string(&chunk.to_value()).unwrap();
                if !writer.write_all(chunk).await.is_empty() {
                    proxy::util::dialog::print(0, &format!("{label} dropped by guest"));
                    return;
                }
                proxy::util::dialog::print(0, &format!("{label}.read() = {items}"));
            }
        });
        reader
    }
    fn consume(mut self, _dep: u32) {
        let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
        spawn(async move {
            let choices = ["read a chunk".to_string(), "drop the stream".to_string()];
            loop {
                let prompt = format!("The host reads {label}");
                if proxy::util::dialog::read_select(0, &prompt, &choices) == 1 {
                    proxy::util::dialog::print(0, &format!("{label} dropped by host"));
                    return;
                }
                let (result, items) = self.read(Vec::with_capacity(4096)).await;
                if let wit_bindgen::StreamResult::Dropped = result {
                    proxy::util::dialog::print(0, &format!("{label} -> dropped"));
                    return;
                }
                let items = wasm_wave::to_string(&items.to_value()).unwrap();
                proxy::util::dialog::print(0, &format!("{label} -> {items}"));
            }
        });
    }
}
impl<T> Dialog for wit_bindgen::FutureReader<T>
where
    T: Dialog + ToValue + ValueTyped + wit_future::FuturePayload,
{
    fn read_value(_dep: u32) -> Self {
        let (writer, reader) = wit_future::new::<
            T,
        >(|| unreachable!("the value is always written"));
        let label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        spawn(async move {
            proxy::util::dialog::print(0, &format!("The host writes {label}"));
            let value: T = Dialog::read_value(1);
            let wave = wasm_wave::to_string(&value.to_value()).unwrap();
            if writer.write(value).await.is_ok() {
                proxy::util::dialog::print(0, &format!("{label}.read() = {wave}"));
            } else {
                proxy::util::dialog::print(0, &format!("{label} dropped by guest"));
            }
        });
        reader
    }
    fn consume(self, _dep: u32) {
        let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
        spawn(async move {
            let choices = ["read the value".to_string(), "drop the future".to_string()];
            let prompt = format!("The host reads {label}");
            if proxy::util::dialog::read_select(0, &prompt, &choices) == 1 {
                drop(self);
                proxy::util::dialog::print(0, &format!("{label} dropped by host"));
                return;
            }
            let value = wasm_wave::to_string(&self.await.to_value()).unwrap();
            proxy::util::dialog::print(0, &format!("{label} -> {value}"));
        });
    }
}
impl Dialog for wit_bindgen::rt::async_support::ErrorContext {
    fn read_value(dep: u32) -> Self {
        Self::new(
            &proxy::util::dialog::read_raw_string(
                dep,
                "Enter the message of an error-context",
            ),
        )
    }
}
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
