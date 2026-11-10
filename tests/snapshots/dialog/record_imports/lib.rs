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
            handle: next_mock_label(),
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
        run_executor().await;
        EXECUTOR.with_borrow_mut(|e| e.stopped = false);
    }
    fn stop() -> () {
        stop_executor()
    }
    fn step() -> u32 {
        next_step()
    }
    fn current_step() -> u32 {
        current_step()
    }
    async fn wait_until_step(step: u32) -> () {
        wait_until_step(step).await
    }
    async fn step_after(step: u32) -> u32 {
        step_after(step).await
    }
    fn host_drops(dropping: bool) -> () {
        let _ = dropping;
    }
}
impl exports::test::host::types::Guest for Stub {
    type Counter = MockedResource;
    fn make_point(x: i32, y: i32) -> exports::test::host::types::Point {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "test:host/types@0.1.0.make-point", __params.join(", ")
            ),
        );
        proxy::util::dialog::print(0, &format!("return type: {}", "point"));
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
    }
    fn check(
        p: exports::test::host::types::Perms,
        c: exports::test::host::types::Color,
    ) -> ::core::result::Result<exports::test::host::types::Shape, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
        __params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "test:host/types@0.1.0.check", __params.join(", ")
            ),
        );
        proxy::util::dialog::print(
            0,
            &format!("return type: {}", "result<shape, string>"),
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
    fn open(
        name: String,
    ) -> ::core::result::Result<exports::test::host::types::Counter, String> {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
        proxy::util::dialog::print(
            0,
            &format!("import: {}({})", "test:host/types@0.1.0.open", __params.join(", ")),
        );
        proxy::util::dialog::print(
            0,
            &format!("return type: {}", "result<counter, string>"),
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
}
impl exports::test::host::types::GuestCounter for MockedResource {
    fn new(start: u32) -> Self {
        let mut __params: Vec<String> = Vec::new();
        __params.push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "new test:host/types/counter@0.1.0", __params
                .join(", ")
            ),
        );
        proxy::util::dialog::print(0, &format!("return type: {}", "counter"));
        let ret = MockedResource {
            name: "counter".to_string(),
            ..Dialog::read_value(0)
        };
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
    }
    fn get(&self) -> u32 {
        let mut __params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "test:host/types/counter@0.1.0.get", __params
                .join(", ")
            ),
        );
        proxy::util::dialog::print(0, &format!("return type: {}", "u32"));
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
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
        proxy::util::dialog::print(
            0,
            &format!(
                "import: {}({})", "test:host/types/counter@0.1.0.merge", __params
                .join(", ")
            ),
        );
        proxy::util::dialog::print(0, &format!("return type: {}", "counter"));
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
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
        proxy::util::dialog::print(
            0,
            &format!("import: {}({})", "test:log/sink@0.2.0.write", __params.join(", ")),
        );
        proxy::util::dialog::print(0, &format!("return type: {}", "bool"));
        let ret = Dialog::read_value(0);
        proxy::util::dialog::print(
            0,
            &format!(
                "ret: {}", wasm_wave::to_string(& ToValue::to_value(& ret)).unwrap()
            ),
        );
        ret
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
impl Dialog for exports::test::host::types::Point {
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
impl Dialog for exports::test::host::types::Shape {
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
            0usize => {
                exports::test::host::types::Shape::Circle(Dialog::read_value(dep + 1))
            }
            1usize => {
                exports::test::host::types::Shape::Rect(Dialog::read_value(dep + 1))
            }
            2usize => {
                exports::test::host::types::Shape::HttpError(Dialog::read_value(dep + 1))
            }
            3usize => exports::test::host::types::Shape::Empty,
            _ => unreachable!(),
        }
    }
}
impl Dialog for exports::test::host::types::Color {
    fn read_value(dep: u32) -> Self {
        let idx = proxy::util::dialog::read_select(
            dep,
            &format!("Select a variant for {}", "test:host/types@0.1.0.color"),
            &["red".to_string(), "green".to_string(), "DNS-blue".to_string()],
        ) as usize;
        match idx {
            0usize => exports::test::host::types::Color::Red,
            1usize => exports::test::host::types::Color::Green,
            2usize => exports::test::host::types::Color::DnsBlue,
            _ => unreachable!(),
        }
    }
}
impl Dialog for exports::test::host::types::Perms {
    fn read_value(dep: u32) -> Self {
        let selections = proxy::util::dialog::read_multi_select(
            dep,
            &format!("Select flags for {}", "test:host/types@0.1.0.perms"),
            &["read".to_string(), "write".to_string(), "exec".to_string()],
        );
        let mut res = exports::test::host::types::Perms::empty();
        for idx in selections {
            match idx as usize {
                0usize => res |= exports::test::host::types::Perms::READ,
                1usize => res |= exports::test::host::types::Perms::WRITE,
                2usize => res |= exports::test::host::types::Perms::EXEC,
                _ => unreachable!(),
            }
        }
        res
    }
}
impl Dialog for exports::test::host::types::Counter {
    fn read_value(_dep: u32) -> Self {
        exports::test::host::types::Counter::new(MockedResource {
            handle: next_mock_label(),
            name: "counter".to_string(),
        })
    }
}
impl<'a> Dialog for exports::test::host::types::CounterBorrow<'a> {
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
type Job = std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>;
#[derive(Default)]
struct Executor {
    queue: Vec<Job>,
    stopped: bool,
    waker: Option<std::task::Waker>,
}
thread_local! {
    static EXECUTOR : std::cell::RefCell < Executor > =
    std::cell::RefCell::new(Executor::default()); static STEP : std::cell::Cell < u32 > =
    const { std::cell::Cell::new(0) }; static WAKE_AGAIN : std::cell::Cell < bool > =
    const { std::cell::Cell::new(false) }; static HELD : std::cell::Cell < u32 > = const
    { std::cell::Cell::new(0) }; static WOKEN_WHILE_HELD : std::cell::Cell < bool > =
    const { std::cell::Cell::new(false) }; static PROBE : std::cell::Cell < bool > =
    const { std::cell::Cell::new(false) }; static PROBED : std::cell::Cell < bool > =
    const { std::cell::Cell::new(false) };
}
/// Counts a step, and returns it.
#[allow(dead_code)]
fn next_step() -> u32 {
    STEP.set(STEP.get() + 1);
    STEP.get()
}
/// The step a job runs in.
#[allow(dead_code)]
fn current_step() -> u32 {
    STEP.get()
}
/// Waits, from any task, until the executor has run `runs` more times, and returns the step of
/// the last run.
#[allow(dead_code)]
async fn wait_for_runs(runs: u32) -> u32 {
    let mut left = runs;
    wait_for_run(move || {
            if left == 0 {
                return true;
            }
            left -= 1;
            false
        })
        .await
}
/// Waits, from any task, until a run of the executor in `step` or later, or returns at once if
/// the count is already there, e.g. replay's for a host line recorded in that step, which checks
/// that it came in that step.
#[allow(dead_code)]
async fn wait_until_step(step: u32) {
    if current_step() >= step {
        return;
    }
    wait_for_run(move || current_step() >= step).await;
}
/// Waits, from any task, as `wait_until_step` does, and takes the next step, of a host line
/// `start` makes, e.g. an export call's arrival, which it returns: the one after the run it
/// waited for, as the executor doesn't run in between (see `wait_for_run`).
#[allow(dead_code)]
async fn step_after(step: u32) -> u32 {
    wait_until_step(step).await;
    next_step()
}
/// Waits, in a job of the executor, until its run in `step`, waking the executor itself until
/// then, e.g. replay's job for a copy recorded in that step.
///
/// Panics if the count passed `step`: replay diverged from the recording.
#[allow(dead_code)]
async fn reach_step(step: u32) {
    std::future::poll_fn(|_| {
            let now = current_step();
            assert!(
                now <= step, "replay reached step {now} before the run in step {step}"
            );
            if now == step {
                std::task::Poll::Ready(())
            } else {
                WAKE_AGAIN.set(true);
                std::task::Poll::Pending
            }
        })
        .await
}
/// Waits until a run of the executor in which `done` returns true, and returns its step. A job
/// asks in each run, waking the executor itself in between, as no host event would.
async fn wait_for_run(mut done: impl FnMut() -> bool + 'static) -> u32 {
    let job = std::future::poll_fn(move |_| {
        if done() {
            return std::task::Poll::Ready(());
        }
        WAKE_AGAIN.set(true);
        std::task::Poll::Pending
    });
    wait_for_job(job).await.1
}
/// Runs `job` in the executor, from any task, and returns its value, and the step of the run it
/// completed in, e.g. record mode's host call that blocked, whose completion wakes the executor.
///
/// The executor doesn't run again until the waiting task resumes, e.g. for a guest's event, so
/// that what the task does next, e.g. return a STARTED call, or take an export call's step,
/// follows the run it waited for, in record and replay alike: replay knows only the step.
///
/// `job` may borrow from the waiting task, e.g. a host call on the resource the guest passed: it
/// is only polled while the waiting task waits, and dropped if the task is dropped first, e.g.
/// the guest cancelled its call, which cancels the host's call too.
#[allow(dead_code)]
async fn wait_for_job<'a, T: 'static>(
    job: impl std::future::Future<Output = T> + 'a,
) -> (T, u32) {
    type Job<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'a>>;
    struct Done<T> {
        job: Option<Job<'static, T>>,
        value: Option<(T, u32)>,
        waker: Option<std::task::Waker>,
        held: bool,
    }
    /// The waiting task's side, which drops the job if it is still pending, and releases the
    /// executor once the task resumes, or is dropped first.
    struct Waiter<T>(std::rc::Rc<std::cell::RefCell<Done<T>>>);
    impl<T> Drop for Waiter<T> {
        fn drop(&mut self) {
            let mut done = self.0.borrow_mut();
            drop(done.job.take());
            if std::mem::take(&mut done.held) {
                HELD.set(HELD.get() - 1);
                if HELD.get() == 0 && WOKEN_WHILE_HELD.replace(false) {
                    wake_executor();
                }
            }
        }
    }
    let job: Job<'a, T> = Box::pin(job);
    let job: Job<'static, T> = unsafe { std::mem::transmute(job) };
    let done = Done {
        job: Some(job),
        value: None,
        waker: None,
        held: false,
    };
    let waiter = Waiter(std::rc::Rc::new(std::cell::RefCell::new(done)));
    let job_shared = waiter.0.clone();
    spawn(
        std::future::poll_fn(move |cx| {
            let mut shared = job_shared.borrow_mut();
            let Some(job) = shared.job.as_mut() else {
                return std::task::Poll::Ready(());
            };
            let std::task::Poll::Ready(value) = job.as_mut().poll(cx) else {
                return std::task::Poll::Pending;
            };
            shared.job = None;
            HELD.set(HELD.get() + 1);
            shared.value = Some((value, current_step()));
            shared.held = true;
            if let Some(waker) = shared.waker.take() {
                waker.wake();
            }
            std::task::Poll::Ready(())
        }),
    );
    let value = std::future::poll_fn(|cx| {
            let mut shared = waiter.0.borrow_mut();
            match shared.value.take() {
                Some(value) => std::task::Poll::Ready(value),
                None => {
                    shared.waker = Some(cx.waker().clone());
                    std::task::Poll::Pending
                }
            }
        })
        .await;
    drop(waiter);
    value
}
/// Queues `job`, from any function, sync or async.
#[allow(dead_code)]
fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
    EXECUTOR
        .with_borrow_mut(|e| {
            e.queue.push(Box::pin(job));
        });
    wake_executor();
}
/// Wakes `run_executor` from another task, with wit-bindgen's `inter-task-wakeup`.
fn wake_executor() {
    let waker = EXECUTOR.with_borrow_mut(|e| e.waker.take());
    if let Some(waker) = waker {
        waker.wake();
    }
}
/// Polls the queued jobs until `stop_executor` is called and they are all done.
#[allow(dead_code)]
async fn run_executor() {
    let mut running: Vec<Job> = Vec::new();
    std::future::poll_fn(|cx| {
            if PROBE.replace(false) {
                PROBED.set(true);
                cx.waker().wake_by_ref();
                return std::task::Poll::Pending;
            }
            PROBED.set(false);
            if HELD.get() > 0 {
                WOKEN_WHILE_HELD.set(true);
                EXECUTOR.with_borrow_mut(|e| e.waker = Some(cx.waker().clone()));
                return std::task::Poll::Pending;
            }
            next_step();
            WAKE_AGAIN.set(false);
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
                        if WAKE_AGAIN.get() {
                            cx.waker().wake_by_ref();
                        }
                        std::task::Poll::Pending
                    }
                })
        })
        .await
}
/// The callback of the executor's task, which `run` exports, in place of wit-bindgen's (see
/// `patch_bindings` in the generator). Resumed after a yield, e.g. by a job waiting for a later
/// step, the executor first takes an event that came meanwhile, as a waiting executor woken by
/// that event does, e.g. in record mode: wit-bindgen would only take it after a poll, in the
/// next run. So its first poll is a probe, which only wakes it: wit-bindgen then takes a
/// pending event, if any, and polls it again. If there was none, wit-bindgen yields, and the
/// callback runs it at once instead.
#[allow(dead_code)]
unsafe fn executor_callback(event0: u32, event1: u32, event2: u32) -> u32 {
    const EVENT_NONE: u32 = 0;
    if event0 == EVENT_NONE {
        PROBE.set(true);
    }
    let code = unsafe {
        wit_bindgen::rt::async_support::callback(event0, event1, event2)
    };
    if PROBED.replace(false) {
        return unsafe { wit_bindgen::rt::async_support::callback(EVENT_NONE, 0, 0) };
    }
    code
}
#[allow(dead_code)]
fn stop_executor() {
    EXECUTOR.with_borrow_mut(|e| e.stopped = true);
    wake_executor();
}
