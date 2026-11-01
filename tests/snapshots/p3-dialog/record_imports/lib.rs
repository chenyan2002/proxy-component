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
        run_executor().await;
        EXECUTOR.with_borrow_mut(|e| e.stopped = false);
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
        proxy::util::dialog::print(
            0,
            &format!("import: {}({})", "test:p3/host@0.1.0.fetch", __params.join(", ")),
        );
        proxy::util::dialog::print(
            0,
            &format!("return type: {}", "result<body, error-code>"),
        );
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
    }
    async fn sleep(ms: u64) -> () {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&ms)).unwrap());
        proxy::util::dialog::print(
            0,
            &format!("import: {}({})", "test:p3/host@0.1.0.sleep", __params.join(", ")),
        );
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
        proxy::util::dialog::print(
            0,
            &format!("import: {}({})", "test:p3/host@0.1.0.finish", __params.join(", ")),
        );
        Dialog::consume(done, 0);
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
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "new test:p3/host/body@0.1.0", __params.join(", ")
            ),
        );
        Dialog::consume(contents, 0);
        proxy::util::dialog::print(0, &format!("return type: {}", "body"));
        let ret = MockedResource {
            name: "body".to_string(),
            ..Dialog::read_value(0)
        };
        PENDING_RET.with(|pending| pending.set(true));
        ret
    }
    async fn size(&self) -> u64 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "test:p3/host/body@0.1.0.size", __params.join(", ")
            ),
        );
        proxy::util::dialog::print(0, &format!("return type: {}", "u64"));
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
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
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "test:p3/host/body@0.1.0.consume", __params.join(", ")
            ),
        );
        proxy::util::dialog::print(
            0,
            &format!(
                "return type: {}",
                "tuple<stream<u8>, future<result<option<body>, error-code>>>"
            ),
        );
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
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
            proxy::util::dialog::print(0, &ret);
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
impl ToRust<exports::test::p3::host::ErrorCode> for Value {
    fn to_rust(&self) -> exports::test::p3::host::ErrorCode {
        match self.unwrap_variant() {
            (ref case, None) if case == "timeout" => {
                exports::test::p3::host::ErrorCode::Timeout
            }
            (ref case, Some(val)) if case == "other" => {
                exports::test::p3::host::ErrorCode::Other(val.to_rust())
            }
            _ => unreachable!(),
        }
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
impl ToRust<exports::test::p3::host::Body> for Value {
    fn to_rust(&self) -> exports::test::p3::host::Body {
        let label = self.unwrap_handle();
        let expect_handle = label
            .strip_prefix(&format!("{}-", "body"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        let handle = exports::test::p3::host::Body::new(MockedResource {
            handle: expect_handle,
            name: "body".to_string(),
        });
        handle
    }
}
impl<'a> ToRust<exports::test::p3::host::BodyBorrow<'a>> for Value {
    fn to_rust(&self) -> exports::test::p3::host::BodyBorrow<'a> {
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
impl Dialog for exports::test::p3::host::ErrorCode {
    fn read_value(dep: u32) -> Self {
        let idx = proxy::util::dialog::read_select(
            dep,
            &format!("Select a variant for {}", "test:p3/host@0.1.0.error-code"),
            &["timeout".to_string(), "other".to_string()],
        ) as usize;
        match idx {
            0usize => exports::test::p3::host::ErrorCode::Timeout,
            1usize => {
                exports::test::p3::host::ErrorCode::Other(Dialog::read_value(dep + 1))
            }
            _ => unreachable!(),
        }
    }
}
impl Dialog for exports::test::p3::host::Body {
    fn read_value(_dep: u32) -> Self {
        exports::test::p3::host::Body::new(MockedResource {
            handle: 0,
            name: "body".to_string(),
        })
    }
}
impl<'a> Dialog for exports::test::p3::host::BodyBorrow<'a> {
    fn read_value(_dep: u32) -> Self {
        unreachable!()
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
        let label = wasm_wave::to_string(&ToValue::to_value(&writer)).unwrap();
        let reader_label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        proxy::util::dialog::print(
            0,
            &format!("new (writer: {label}, reader: {reader_label})"),
        );
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
        let label = wasm_wave::to_string(&ToValue::to_value(&writer)).unwrap();
        let reader_label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
        proxy::util::dialog::print(
            0,
            &format!("new (writer: {label}, reader: {reader_label})"),
        );
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
