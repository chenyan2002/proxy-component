mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    async fn start() -> () {
        let mut __executor = wit_bindgen::spawn_local(
            proxy::conversion::executor::run(),
        );
        loop {
            let idx = proxy::util::dialog::read_select(
                0,
                "Select an export function to call",
                &[
                    "All done".to_string(),
                    "test:p3/api@0.1.0.handle".to_string(),
                    "test:p3/api@0.1.0.serve".to_string(),
                    "test:p3/api@0.1.0.echo".to_string(),
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
                    let res = test::p3::api::handle(url).await;
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "ret: {}", wasm_wave::to_string(& ToValue::to_value(& res))
                            .unwrap()
                        ),
                    );
                    let _ = res;
                    join_spawned().await;
                    proxy::conversion::executor::stop();
                    __executor.await;
                    __executor = wit_bindgen::spawn_local(
                        proxy::conversion::executor::run(),
                    );
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
                    let res = test::p3::api::serve(port).await;
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "ret: {}", wasm_wave::to_string(& ToValue::to_value(& res))
                            .unwrap()
                        ),
                    );
                    Dialog::consume(res, 0);
                    join_spawned().await;
                    proxy::conversion::executor::stop();
                    __executor.await;
                    __executor = wit_bindgen::spawn_local(
                        proxy::conversion::executor::run(),
                    );
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
                    let res = test::p3::api::echo(s);
                    proxy::util::dialog::print(
                        0,
                        &format!(
                            "ret: {}", wasm_wave::to_string(& ToValue::to_value(& res))
                            .unwrap()
                        ),
                    );
                    Dialog::consume(res, 0);
                    join_spawned().await;
                    proxy::conversion::executor::stop();
                    __executor.await;
                    __executor = wit_bindgen::spawn_local(
                        proxy::conversion::executor::run(),
                    );
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
            handle: next_mock_label(),
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
thread_local! {
    static READER_LABELS : std::cell::RefCell < std::collections::HashMap < u32, u32 >> =
    std::cell::RefCell::new(std::collections::HashMap::new());
}
/// Labels a stream or future this component made by the handle of its `writer`, which it
/// keeps for the stream's lifetime, also where its `reader` is printed: in the value that
/// hands the reader out, after which the reader moves on, and its handle can be reused.
/// So the entry is gone once the reader is printed.
fn label_by_writer(reader: u32, writer: u32) {
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        wasm_wave::value::convert::set_handle_label(|_kind, handle| {
            READER_LABELS.with_borrow_mut(|labels| labels.remove(&handle))
        })
    });
    READER_LABELS
        .with_borrow_mut(|labels| {
            labels.remove(&writer);
            labels.insert(reader, writer);
        });
}
/// The label of a stream or future, e.g. `#stream-7`, by the end this component keeps.
fn handle_label(end: &impl ToValue) -> String {
    wasm_wave::to_string(&end.to_value()).unwrap()
}
/// A stream the host writes: its writer, its reader, to hand out, and its label.
fn host_stream<T: wit_stream::StreamPayload + 'static>(
    label_reader: bool,
) -> (wit_bindgen::StreamWriter<T>, wit_bindgen::StreamReader<T>, String) {
    let (writer, reader) = wit_stream::new::<T>();
    if label_reader {
        label_by_writer(reader.handle(), writer.handle());
    }
    let label = handle_label(&writer);
    (writer, reader, label)
}
/// A future the host writes, which it always does: its writer, its reader, to hand
/// out, and its label.
fn host_future<T: wit_future::FuturePayload + 'static>(
    label_reader: bool,
) -> (wit_bindgen::FutureWriter<T>, wit_bindgen::FutureReader<T>, String) {
    let (writer, reader) = wit_future::new::<
        T,
    >(|| unreachable!("the value is always written"));
    if label_reader {
        label_by_writer(reader.handle(), writer.handle());
    }
    let label = handle_label(&writer);
    (writer, reader, label)
}
impl<T> Dialog for wit_bindgen::StreamReader<T>
where
    T: Dialog + ToValue + ValueTyped + wit_stream::StreamPayload + 'static,
{
    fn read_value(_dep: u32) -> Self {
        let (mut writer, reader, label) = host_stream::<T>(true);
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
        let label = handle_label(&self);
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
        let (writer, reader, label) = host_future::<T>(true);
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
        let label = handle_label(&self);
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
    static MOCK_LABEL : std::cell::Cell < u32 > = const { std::cell::Cell::new(0) };
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
