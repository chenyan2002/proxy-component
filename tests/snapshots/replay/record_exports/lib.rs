mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::recorder::start_replay::Guest for Stub {
    async fn start() -> () {
        static PANIC_HOOK: std::sync::Once = std::sync::Once::new();
        PANIC_HOOK
            .call_once(|| std::panic::set_hook(
                Box::new(|info| proxy::recorder::replay::fail(&info.to_string())),
            ));
        /// A line of the guest, which fuzz mode records, and replay checks.
        #[allow(dead_code)]
        fn trace_guest(line: &Line) {
            proxy::recorder::replay::check(line);
        }
        /// Calls an async export, and reports that the guest started it, if it blocks.
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
                        trace_guest(
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
        let __executor = wit_bindgen::spawn_local(proxy::conversion::executor::run());
        while let Some(line) = proxy::recorder::replay::next_host_line() {
            if let Some(at) = line.at {
                let step = proxy::conversion::executor::arrive_after(at - 1).await;
                assert_eq!(step, at, "replay came to {:?} in step {step}", line.event);
            }
            match line.event {
                Event::Call(Call { id: __id, func, args, .. }) => {
                    match func.as_str() {
                        "test:guest/api@0.1.0.run" => {
                            let res = {
                                let arg_value: Value = wasm_wave::from_str(
                                        &<test::guest::api::Point as ValueTyped>::value_type(),
                                        &args[0usize],
                                    )
                                    .unwrap();
                                let p: test::guest::api::Point = Replay::replay(&arg_value);
                                let arg_value: Value = wasm_wave::from_str(
                                        &<test::guest::api::Counter as ValueTyped>::value_type(),
                                        &args[1usize],
                                    )
                                    .unwrap();
                                let c: test::guest::api::Counter = Replay::replay(
                                    &arg_value,
                                );
                                let res = test::guest::api::run(&p, &c);
                                trace_guest(
                                    &trace_return(
                                        Party::Guest,
                                        __id,
                                        Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                                    ),
                                );
                                proxy::conversion::executor::host_drops(true);
                                res
                            };
                            Replay::consume(res);
                            SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                            proxy::conversion::executor::host_drops(false);
                        }
                        "test:guest/api@0.1.0.classify" => {
                            let res = {
                                let arg_value: Value = wasm_wave::from_str(
                                        &<test::guest::api::Shape as ValueTyped>::value_type(),
                                        &args[0usize],
                                    )
                                    .unwrap();
                                let s: test::guest::api::Shape = Replay::replay(&arg_value);
                                let res = test::guest::api::classify(&s);
                                trace_guest(
                                    &trace_return(
                                        Party::Guest,
                                        __id,
                                        Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                                    ),
                                );
                                proxy::conversion::executor::host_drops(true);
                                res
                            };
                            Replay::consume(res);
                            SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                            proxy::conversion::executor::host_drops(false);
                        }
                        "test:guest/api@0.1.0.begin" => {
                            let res = {
                                let arg_value: Value = wasm_wave::from_str(
                                        &<test::guest::api::Counter as ValueTyped>::value_type(),
                                        &args[0usize],
                                    )
                                    .unwrap();
                                let c: test::guest::api::Counter = Replay::replay(
                                    &arg_value,
                                );
                                let res = test::guest::api::begin(c);
                                trace_guest(
                                    &trace_return(
                                        Party::Guest,
                                        __id,
                                        Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                                    ),
                                );
                                proxy::conversion::executor::host_drops(true);
                                res
                            };
                            Replay::consume(res);
                            SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                            proxy::conversion::executor::host_drops(false);
                        }
                        "new test:guest/api/session@0.1.0" => {
                            let res = {
                                let arg_value: Value = wasm_wave::from_str(
                                        &<String as ValueTyped>::value_type(),
                                        &args[0usize],
                                    )
                                    .unwrap();
                                let name: String = Replay::replay(&arg_value);
                                let res = test::guest::api::Session::new(&name);
                                trace_guest(
                                    &trace_return(
                                        Party::Guest,
                                        __id,
                                        Some(wasm_wave::to_string(&res.to_value()).unwrap()),
                                    ),
                                );
                                proxy::conversion::executor::host_drops(true);
                                res
                            };
                            Replay::consume(res);
                            SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
                            proxy::conversion::executor::host_drops(false);
                        }
                        _ => unreachable!("the trace calls an unknown export {func}"),
                    }
                }
                Event::Drop(label) => {
                    let (name, handle) = label
                        .name
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
                }
                event => unreachable!("`start` doesn't perform {event:?}"),
            }
        }
        join_spawned().await;
        proxy::conversion::executor::stop();
        __executor.await;
        proxy::recorder::replay::finish();
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
impl Replay for test::guest::api::Session {
    fn replay(value: &Value) -> Self {
        let label = value.unwrap_handle();
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
    fn consume(self) {
        std::mem::forget(self);
    }
}
impl<'a> Replay for &'a test::guest::api::Session {
    fn replay(value: &Value) -> Self {
        let label = value.unwrap_handle();
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
impl Replay for test::host::types::Point {
    #[allow(unused_variables)]
    fn replay(value: &Value) -> Self {
        let fields: std::collections::BTreeMap<_, _> = value.unwrap_record().collect();
        test::host::types::Point {
            x: Replay::replay(&fields["x"]),
            y: Replay::replay(&fields["y"]),
            label: Replay::replay(&fields["label"]),
        }
    }
    #[allow(unused_variables)]
    fn consume(self) {
        Replay::consume(self.x);
        Replay::consume(self.y);
        Replay::consume(self.label);
    }
}
impl Replay for test::host::types::Shape {
    fn replay(value: &Value) -> Self {
        match value.unwrap_variant() {
            (ref case, Some(val)) if case == "circle" => {
                test::host::types::Shape::Circle(Replay::replay(&val))
            }
            (ref case, Some(val)) if case == "rect" => {
                test::host::types::Shape::Rect(Replay::replay(&val))
            }
            (ref case, Some(val)) if case == "HTTP-error" => {
                test::host::types::Shape::HttpError(Replay::replay(&val))
            }
            (ref case, None) if case == "empty" => test::host::types::Shape::Empty,
            _ => unreachable!(),
        }
    }
    fn consume(self) {
        match self {
            test::host::types::Shape::Circle(x) => Replay::consume(x),
            test::host::types::Shape::Rect(x) => Replay::consume(x),
            test::host::types::Shape::HttpError(x) => Replay::consume(x),
            test::host::types::Shape::Empty => {}
        }
    }
}
impl Replay for test::host::types::Color {
    fn replay(value: &Value) -> Self {
        match value.unwrap_variant() {
            (ref case, None) if case == "red" => test::host::types::Color::Red,
            (ref case, None) if case == "green" => test::host::types::Color::Green,
            (ref case, None) if case == "DNS-blue" => test::host::types::Color::DnsBlue,
            _ => unreachable!(),
        }
    }
    fn consume(self) {
        match self {
            test::host::types::Color::Red => {}
            test::host::types::Color::Green => {}
            test::host::types::Color::DnsBlue => {}
        }
    }
}
impl Replay for test::host::types::Perms {
    fn replay(value: &Value) -> Self {
        let flags: Vec<_> = value.unwrap_flags().map(|f| f.into_owned()).collect();
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
impl Replay for test::host::types::Counter {
    fn replay(value: &Value) -> Self {
        let label = value.unwrap_handle();
        let handle = label
            .strip_prefix(&format!("{}-", "counter"))
            .and_then(|s| s.parse::<u32>().ok())
            .expect("invalid handle label");
        proxy::conversion::conversion::get_mock_test_host_types_counter(handle)
    }
}
impl<'a> Replay for &'a test::host::types::Counter {
    fn replay(value: &Value) -> Self {
        let label = value.unwrap_handle();
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
trait Replay: Sized {
    fn replay(value: &Value) -> Self;
    fn replay_payload(value: Option<&Value>) -> Self {
        Self::replay(value.expect("the case has a payload"))
    }
    #[allow(dead_code)]
    fn consume(self) {}
}
/// The WAVE value of `text`, of type `T`.
#[allow(dead_code)]
fn wave_value<T: ValueTyped>(text: &str) -> Value {
    wasm_wave::from_str(&T::value_type(), text).unwrap()
}
macro_rules! impl_replay_primitive {
    ($($t:ty),*) => {
        $(impl Replay for $t { fn replay(value : & Value) -> Self { value.to_rust() } })*
    };
}
impl_replay_primitive!(
    bool, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, char, String
);
impl Replay for () {
    fn replay(_value: &Value) -> Self {}
    fn replay_payload(_value: Option<&Value>) -> Self {}
}
impl<T: Replay> Replay for Vec<T> {
    fn replay(value: &Value) -> Self {
        value.unwrap_list().map(|x| T::replay(&x)).collect()
    }
    fn consume(self) {
        self.into_iter().for_each(Replay::consume);
    }
}
impl<T: Replay> Replay for Option<T> {
    fn replay(value: &Value) -> Self {
        value.unwrap_option().map(|x| T::replay(&x))
    }
    fn consume(self) {
        self.into_iter().for_each(Replay::consume);
    }
}
impl<T: Replay, E: Replay> Replay for core::result::Result<T, E> {
    fn replay(value: &Value) -> Self {
        match value.unwrap_result() {
            Ok(x) => Ok(T::replay_payload(x.as_deref())),
            Err(x) => Err(E::replay_payload(x.as_deref())),
        }
    }
    fn consume(self) {
        match self {
            Ok(x) => x.consume(),
            Err(x) => x.consume(),
        }
    }
}
macro_rules! impl_replay_tuple {
    ($($T:ident),+) => {
        impl < $($T : Replay),+ > Replay for ($($T,)+) { fn replay(value : & Value) ->
        Self { let mut items = value.unwrap_tuple(); ($($T ::replay(& items.next()
        .unwrap()),)+) } #[allow(non_snake_case)] fn consume(self) { let ($($T,)+) =
        self; $($T .consume();)+ } }
    };
}
impl_replay_tuple!(T1);
impl_replay_tuple!(T1, T2);
impl_replay_tuple!(T1, T2, T3);
impl_replay_tuple!(T1, T2, T3, T4);
impl_replay_tuple!(T1, T2, T3, T4, T5);
impl_replay_tuple!(T1, T2, T3, T4, T5, T6);
impl_replay_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_replay_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_replay_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl Replay for MockedResource {
    fn replay(value: &Value) -> Self {
        let label = value.unwrap_handle();
        let (name, handle) = label
            .rsplit_once('-')
            .and_then(|(name, handle_str)| {
                handle_str.parse::<u32>().ok().map(|h| (name.to_string(), h))
            })
            .expect("invalid handle label");
        MockedResource { handle, name }
    }
}
impl Replay for wit_bindgen::rt::async_support::ErrorContext {
    fn replay(_value: &Value) -> Self {
        Self::new("")
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
    proxy::conversion::executor::current_step()
}
#[allow(dead_code)]
async fn trace_reach(at: Option<u32>) {
    if let Some(at) = at {
        proxy::conversion::executor::wait_until_step(at).await;
    }
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
