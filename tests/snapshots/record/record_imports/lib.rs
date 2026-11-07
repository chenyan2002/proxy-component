mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::proxy::conversion::conversion::Guest for Stub {
    fn get_wrapped_test_host_types_counter(
        x: exports::proxy::conversion::conversion::HostTestHostTypesCounter,
    ) -> exports::proxy::conversion::conversion::WrappedTestHostTypesCounter {
        x.to_proxy()
    }
    fn get_host_test_host_types_counter(
        x: exports::proxy::conversion::conversion::WrappedTestHostTypesCounter,
    ) -> exports::proxy::conversion::conversion::HostTestHostTypesCounter {
        x.to_proxy()
    }
    fn get_handle_wrapped_test_host_types_counter(
        x: exports::proxy::conversion::conversion::WrappedTestHostTypesCounterBorrow<'_>,
    ) -> u32 {
        x.to_proxy().handle()
    }
}
impl exports::proxy::conversion::executor::Guest for Stub {
    async fn run() -> () {
        thread_local! {
            static RUNNING : std::cell::Cell < bool > = const {
            std::cell::Cell::new(false) };
        }
        if RUNNING.replace(true) {
            EXECUTOR.with_borrow_mut(|e| e.stopped = false);
            return;
        }
        run_executor().await;
        RUNNING.set(false);
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
impl exports::wrapped_test::host::types::Guest for Stub {
    type Counter = test::host::types::Counter;
    fn make_point(x: i32, y: i32) -> exports::wrapped_test::host::types::Point {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let x = x.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&x)).unwrap());
        let y = y.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&y)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:host/types@0.1.0.make-point",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (test::host::types::make_point(x, y), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res.to_proxy()
    }
    fn check(
        p: exports::wrapped_test::host::types::Perms,
        c: exports::wrapped_test::host::types::Color,
    ) -> ::core::result::Result<exports::wrapped_test::host::types::Shape, String> {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let p = p.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
        let c = c.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:host/types@0.1.0.check",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (test::host::types::check(p, c), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res.to_proxy()
    }
    fn open(
        name: String,
    ) -> ::core::result::Result<exports::wrapped_test::host::types::Counter, String> {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let name = name.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&name)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:host/types@0.1.0.open",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (test::host::types::open(&name), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res.to_proxy()
    }
}
impl exports::wrapped_test::host::types::GuestCounter for test::host::types::Counter {
    fn new(start: u32) -> Self {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let start = start.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&start)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "new test:host/types/counter@0.1.0",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (Self::new(start), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res
    }
    fn get(&self) -> u32 {
        let __id = trace_task_id(Table::Imports);
        let params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        SENDER.set(Party::Guest);
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:host/types/counter@0.1.0.get",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (self.get(), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res.to_proxy()
    }
    fn add(&self, n: u32) -> () {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = vec![
            wasm_wave::to_string(& ToValue::to_value(& self)).unwrap()
        ];
        SENDER.set(Party::Guest);
        let n = n.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&n)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:host/types/counter@0.1.0.add",
                    false,
                    params,
                )
            },
        );
        let ((), __at) = (self.add(n), None);
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, None)
            },
        );
    }
    fn merge(
        a: exports::wrapped_test::host::types::CounterBorrow<'_>,
        b: exports::wrapped_test::host::types::Counter,
    ) -> exports::wrapped_test::host::types::Counter {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let a = a.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&a)).unwrap());
        let b = b.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&b)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:host/types/counter@0.1.0.merge",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (Self::merge(&a, b), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res.to_proxy()
    }
    unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
        if let Some(res) = *unsafe { Box::from_raw(handle) } {
            let name = wasm_wave::to_string(&res.to_value()).unwrap();
            proxy::recorder::record::record(
                &trace_drop(Party::Guest, Table::Imports, name),
            );
            drop(res);
        }
        None
    }
}
impl exports::wrapped_test::log0_1_0::sink::Guest for Stub {
    fn write(msg: String) -> () {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let msg = msg.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&msg)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:log/sink@0.1.0.write",
                    false,
                    params,
                )
            },
        );
        let ((), __at) = (test::log0_1_0::sink::write(&msg), None);
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, None)
            },
        );
    }
}
impl exports::wrapped_test::log0_2_0::sink::Guest for Stub {
    fn write(msg: String) -> bool {
        let __id = trace_task_id(Table::Imports);
        let mut params: Vec<String> = Vec::new();
        SENDER.set(Party::Guest);
        let msg = msg.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&msg)).unwrap());
        let __at: Option<u32> = None;
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_call(
                    Party::Guest,
                    __id,
                    "test:log/sink@0.2.0.write",
                    false,
                    params,
                )
            },
        );
        let (res, __at) = (test::log0_2_0::sink::write(&msg), None);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record(
            &Line {
                at: __at,
                ..trace_return(Party::Host, __id, Some(wave_res))
            },
        );
        SENDER.set(Party::Host);
        res.to_proxy()
    }
}
impl ValueTyped for exports::wrapped_test::host::types::Counter {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl<'a> ValueTyped for exports::wrapped_test::host::types::CounterBorrow<'a> {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl ToValue for exports::wrapped_test::host::types::Counter {
    fn to_value(&self) -> Value {
        let label = format!("{}-{}", "counter", self.handle());
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for exports::wrapped_test::host::types::CounterBorrow<'a> {
    fn to_value(&self) -> Value {
        type T = test::host::types::Counter;
        let label = format!("borrow-{}-{}", "counter", self.get:: < T > ().handle());
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
impl ToProxy for exports::wrapped_test::host::types::Counter {
    type Output = test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        self.into_inner()
    }
}
impl<'a> ToProxy for exports::wrapped_test::host::types::CounterBorrow<'a> {
    type Output = &'a test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        type T = test::host::types::Counter;
        self.get::<T>()
    }
}
impl ToProxy for test::host::types::Point {
    type Output = test::host::types::Point;
    fn to_proxy(self) -> Self::Output {
        self
    }
    const IS_ID: bool = true;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Ok(x)
    }
}
impl ToProxy for test::host::types::Shape {
    type Output = test::host::types::Shape;
    fn to_proxy(self) -> Self::Output {
        self
    }
    const IS_ID: bool = true;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Ok(x)
    }
}
impl ToProxy for test::host::types::Color {
    type Output = test::host::types::Color;
    fn to_proxy(self) -> Self::Output {
        self
    }
    const IS_ID: bool = true;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Ok(x)
    }
}
impl ToProxy for test::host::types::Perms {
    type Output = test::host::types::Perms;
    fn to_proxy(self) -> Self::Output {
        self
    }
    const IS_ID: bool = true;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Ok(x)
    }
}
impl ToProxy for test::host::types::Counter {
    type Output = exports::wrapped_test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        Self::Output::new(self)
    }
}
impl<'a> ToProxy for &'a test::host::types::Counter {
    type Output = exports::wrapped_test::host::types::CounterBorrow<'a>;
    fn to_proxy(self) -> Self::Output {
        unsafe { Self::Output::lift(self as *const _ as *const u8) }
    }
}
trait ToProxy: Sized {
    type Output;
    fn to_proxy(self) -> Self::Output;
    const IS_ID: bool = false;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Err(x)
    }
}
trait TypeCtor {
    type Apply<X>;
}
#[allow(dead_code)]
struct Id;
impl TypeCtor for Id {
    type Apply<X> = X;
}
#[allow(dead_code)]
struct InVec<F>(core::marker::PhantomData<F>);
impl<F: TypeCtor> TypeCtor for InVec<F> {
    type Apply<X> = F::Apply<Vec<X>>;
}
#[allow(dead_code)]
struct InOption<F>(core::marker::PhantomData<F>);
impl<F: TypeCtor> TypeCtor for InOption<F> {
    type Apply<X> = F::Apply<Option<X>>;
}
#[allow(dead_code)]
struct InResultOk<F, E>(core::marker::PhantomData<(F, E)>);
impl<F: TypeCtor, E> TypeCtor for InResultOk<F, E> {
    type Apply<X> = F::Apply<Result<X, E>>;
}
#[allow(dead_code)]
struct InResultErr<F, T>(core::marker::PhantomData<(F, T)>);
impl<F: TypeCtor, T> TypeCtor for InResultErr<F, T> {
    type Apply<X> = F::Apply<Result<T, X>>;
}
impl crate::ToProxy for String {
    type Output = String;
    fn to_proxy(self) -> Self::Output {
        self
    }
    const IS_ID: bool = true;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Ok(x)
    }
}
impl<T: crate::ToProxy> crate::ToProxy for Vec<T> {
    type Output = Vec<T::Output>;
    fn to_proxy(self) -> Self::Output {
        match T::cast::<InVec<Id>>(self) {
            Ok(v) => v,
            Err(v) => v.into_iter().map(|x| x.to_proxy()).collect(),
        }
    }
    const IS_ID: bool = T::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        T::cast::<InVec<F>>(x)
    }
}
impl<T, E> ToProxy for Result<T, E>
where
    T: ToProxy,
    E: ToProxy,
{
    type Output = Result<T::Output, E::Output>;
    fn to_proxy(self) -> Self::Output {
        match self {
            Ok(ok) => Ok(ok.to_proxy()),
            Err(err) => Err(err.to_proxy()),
        }
    }
    const IS_ID: bool = T::IS_ID && E::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T::cast::<InResultOk<F, E>>(x) else { unreachable!() };
        let Ok(x) = E::cast::<InResultErr<F, T::Output>>(x) else { unreachable!() };
        Ok(x)
    }
}
impl<Inner> ToProxy for Option<Inner>
where
    Inner: ToProxy,
{
    type Output = Option<Inner::Output>;
    fn to_proxy(self) -> Self::Output {
        self.map(|x| x.to_proxy())
    }
    const IS_ID: bool = Inner::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        Inner::cast::<InOption<F>>(x)
    }
}
macro_rules! impl_to_import_export_for_primitive {
    ($($t:ty),*) => {
        $(impl ToProxy for $t { type Output = $t; fn to_proxy(self) -> Self::Output {
        self } const IS_ID : bool = true; fn cast < F : TypeCtor > (x : F::Apply < Self
        >) -> Result < F::Apply < Self::Output >, F::Apply < Self >> { Ok(x) } })*
    };
}
impl_to_import_export_for_primitive!(
    u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64, (), bool, char
);
#[allow(dead_code, non_camel_case_types)]
struct InTuple1_0<F>(core::marker::PhantomData<(F,)>);
impl<F: TypeCtor> TypeCtor for InTuple1_0<F> {
    type Apply<X> = F::Apply<(X,)>;
}
impl<T0: ToProxy> ToProxy for (T0,) {
    type Output = (T0::Output,);
    fn to_proxy(self) -> Self::Output {
        (self.0.to_proxy(),)
    }
    const IS_ID: bool = T0::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple1_0<F>>(x) else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple2_0<F, A1>(core::marker::PhantomData<(F, A1)>);
impl<F: TypeCtor, A1> TypeCtor for InTuple2_0<F, A1> {
    type Apply<X> = F::Apply<(X, A1)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple2_1<F, A0>(core::marker::PhantomData<(F, A0)>);
impl<F: TypeCtor, A0> TypeCtor for InTuple2_1<F, A0> {
    type Apply<X> = F::Apply<(A0, X)>;
}
impl<T0: ToProxy, T1: ToProxy> ToProxy for (T0, T1) {
    type Output = (T0::Output, T1::Output);
    fn to_proxy(self) -> Self::Output {
        (self.0.to_proxy(), self.1.to_proxy())
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple2_0<F, T1>>(x) else { unreachable!() };
        let Ok(x) = T1::cast::<InTuple2_1<F, T0::Output>>(x) else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple3_0<F, A1, A2>(core::marker::PhantomData<(F, A1, A2)>);
impl<F: TypeCtor, A1, A2> TypeCtor for InTuple3_0<F, A1, A2> {
    type Apply<X> = F::Apply<(X, A1, A2)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple3_1<F, A0, A2>(core::marker::PhantomData<(F, A0, A2)>);
impl<F: TypeCtor, A0, A2> TypeCtor for InTuple3_1<F, A0, A2> {
    type Apply<X> = F::Apply<(A0, X, A2)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple3_2<F, A0, A1>(core::marker::PhantomData<(F, A0, A1)>);
impl<F: TypeCtor, A0, A1> TypeCtor for InTuple3_2<F, A0, A1> {
    type Apply<X> = F::Apply<(A0, A1, X)>;
}
impl<T0: ToProxy, T1: ToProxy, T2: ToProxy> ToProxy for (T0, T1, T2) {
    type Output = (T0::Output, T1::Output, T2::Output);
    fn to_proxy(self) -> Self::Output {
        (self.0.to_proxy(), self.1.to_proxy(), self.2.to_proxy())
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple3_0<F, T1, T2>>(x) else { unreachable!() };
        let Ok(x) = T1::cast::<InTuple3_1<F, T0::Output, T2>>(x) else { unreachable!() };
        let Ok(x) = T2::cast::<InTuple3_2<F, T0::Output, T1::Output>>(x) else {
            unreachable!()
        };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple4_0<F, A1, A2, A3>(core::marker::PhantomData<(F, A1, A2, A3)>);
impl<F: TypeCtor, A1, A2, A3> TypeCtor for InTuple4_0<F, A1, A2, A3> {
    type Apply<X> = F::Apply<(X, A1, A2, A3)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple4_1<F, A0, A2, A3>(core::marker::PhantomData<(F, A0, A2, A3)>);
impl<F: TypeCtor, A0, A2, A3> TypeCtor for InTuple4_1<F, A0, A2, A3> {
    type Apply<X> = F::Apply<(A0, X, A2, A3)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple4_2<F, A0, A1, A3>(core::marker::PhantomData<(F, A0, A1, A3)>);
impl<F: TypeCtor, A0, A1, A3> TypeCtor for InTuple4_2<F, A0, A1, A3> {
    type Apply<X> = F::Apply<(A0, A1, X, A3)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple4_3<F, A0, A1, A2>(core::marker::PhantomData<(F, A0, A1, A2)>);
impl<F: TypeCtor, A0, A1, A2> TypeCtor for InTuple4_3<F, A0, A1, A2> {
    type Apply<X> = F::Apply<(A0, A1, A2, X)>;
}
impl<T0: ToProxy, T1: ToProxy, T2: ToProxy, T3: ToProxy> ToProxy for (T0, T1, T2, T3) {
    type Output = (T0::Output, T1::Output, T2::Output, T3::Output);
    fn to_proxy(self) -> Self::Output {
        (self.0.to_proxy(), self.1.to_proxy(), self.2.to_proxy(), self.3.to_proxy())
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID && T3::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple4_0<F, T1, T2, T3>>(x) else { unreachable!() };
        let Ok(x) = T1::cast::<InTuple4_1<F, T0::Output, T2, T3>>(x) else {
            unreachable!()
        };
        let Ok(x) = T2::cast::<InTuple4_2<F, T0::Output, T1::Output, T3>>(x) else {
            unreachable!()
        };
        let Ok(x) = T3::cast::<InTuple4_3<F, T0::Output, T1::Output, T2::Output>>(x)
        else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple5_0<F, A1, A2, A3, A4>(core::marker::PhantomData<(F, A1, A2, A3, A4)>);
impl<F: TypeCtor, A1, A2, A3, A4> TypeCtor for InTuple5_0<F, A1, A2, A3, A4> {
    type Apply<X> = F::Apply<(X, A1, A2, A3, A4)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple5_1<F, A0, A2, A3, A4>(core::marker::PhantomData<(F, A0, A2, A3, A4)>);
impl<F: TypeCtor, A0, A2, A3, A4> TypeCtor for InTuple5_1<F, A0, A2, A3, A4> {
    type Apply<X> = F::Apply<(A0, X, A2, A3, A4)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple5_2<F, A0, A1, A3, A4>(core::marker::PhantomData<(F, A0, A1, A3, A4)>);
impl<F: TypeCtor, A0, A1, A3, A4> TypeCtor for InTuple5_2<F, A0, A1, A3, A4> {
    type Apply<X> = F::Apply<(A0, A1, X, A3, A4)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple5_3<F, A0, A1, A2, A4>(core::marker::PhantomData<(F, A0, A1, A2, A4)>);
impl<F: TypeCtor, A0, A1, A2, A4> TypeCtor for InTuple5_3<F, A0, A1, A2, A4> {
    type Apply<X> = F::Apply<(A0, A1, A2, X, A4)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple5_4<F, A0, A1, A2, A3>(core::marker::PhantomData<(F, A0, A1, A2, A3)>);
impl<F: TypeCtor, A0, A1, A2, A3> TypeCtor for InTuple5_4<F, A0, A1, A2, A3> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, X)>;
}
impl<T0: ToProxy, T1: ToProxy, T2: ToProxy, T3: ToProxy, T4: ToProxy> ToProxy
for (T0, T1, T2, T3, T4) {
    type Output = (T0::Output, T1::Output, T2::Output, T3::Output, T4::Output);
    fn to_proxy(self) -> Self::Output {
        (
            self.0.to_proxy(),
            self.1.to_proxy(),
            self.2.to_proxy(),
            self.3.to_proxy(),
            self.4.to_proxy(),
        )
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID && T3::IS_ID && T4::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple5_0<F, T1, T2, T3, T4>>(x) else { unreachable!() };
        let Ok(x) = T1::cast::<InTuple5_1<F, T0::Output, T2, T3, T4>>(x) else {
            unreachable!()
        };
        let Ok(x) = T2::cast::<InTuple5_2<F, T0::Output, T1::Output, T3, T4>>(x) else {
            unreachable!()
        };
        let Ok(x) = T3::cast::<InTuple5_3<F, T0::Output, T1::Output, T2::Output, T4>>(x)
        else { unreachable!() };
        let Ok(x) = T4::cast::<
            InTuple5_4<F, T0::Output, T1::Output, T2::Output, T3::Output>,
        >(x) else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple6_0<F, A1, A2, A3, A4, A5>(
    core::marker::PhantomData<(F, A1, A2, A3, A4, A5)>,
);
impl<F: TypeCtor, A1, A2, A3, A4, A5> TypeCtor for InTuple6_0<F, A1, A2, A3, A4, A5> {
    type Apply<X> = F::Apply<(X, A1, A2, A3, A4, A5)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple6_1<F, A0, A2, A3, A4, A5>(
    core::marker::PhantomData<(F, A0, A2, A3, A4, A5)>,
);
impl<F: TypeCtor, A0, A2, A3, A4, A5> TypeCtor for InTuple6_1<F, A0, A2, A3, A4, A5> {
    type Apply<X> = F::Apply<(A0, X, A2, A3, A4, A5)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple6_2<F, A0, A1, A3, A4, A5>(
    core::marker::PhantomData<(F, A0, A1, A3, A4, A5)>,
);
impl<F: TypeCtor, A0, A1, A3, A4, A5> TypeCtor for InTuple6_2<F, A0, A1, A3, A4, A5> {
    type Apply<X> = F::Apply<(A0, A1, X, A3, A4, A5)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple6_3<F, A0, A1, A2, A4, A5>(
    core::marker::PhantomData<(F, A0, A1, A2, A4, A5)>,
);
impl<F: TypeCtor, A0, A1, A2, A4, A5> TypeCtor for InTuple6_3<F, A0, A1, A2, A4, A5> {
    type Apply<X> = F::Apply<(A0, A1, A2, X, A4, A5)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple6_4<F, A0, A1, A2, A3, A5>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A5)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A5> TypeCtor for InTuple6_4<F, A0, A1, A2, A3, A5> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, X, A5)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple6_5<F, A0, A1, A2, A3, A4>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4> TypeCtor for InTuple6_5<F, A0, A1, A2, A3, A4> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, X)>;
}
impl<
    T0: ToProxy,
    T1: ToProxy,
    T2: ToProxy,
    T3: ToProxy,
    T4: ToProxy,
    T5: ToProxy,
> ToProxy for (T0, T1, T2, T3, T4, T5) {
    type Output = (
        T0::Output,
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
    );
    fn to_proxy(self) -> Self::Output {
        (
            self.0.to_proxy(),
            self.1.to_proxy(),
            self.2.to_proxy(),
            self.3.to_proxy(),
            self.4.to_proxy(),
            self.5.to_proxy(),
        )
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID && T3::IS_ID && T4::IS_ID
        && T5::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple6_0<F, T1, T2, T3, T4, T5>>(x) else {
            unreachable!()
        };
        let Ok(x) = T1::cast::<InTuple6_1<F, T0::Output, T2, T3, T4, T5>>(x) else {
            unreachable!()
        };
        let Ok(x) = T2::cast::<InTuple6_2<F, T0::Output, T1::Output, T3, T4, T5>>(x)
        else { unreachable!() };
        let Ok(x) = T3::cast::<
            InTuple6_3<F, T0::Output, T1::Output, T2::Output, T4, T5>,
        >(x) else { unreachable!() };
        let Ok(x) = T4::cast::<
            InTuple6_4<F, T0::Output, T1::Output, T2::Output, T3::Output, T5>,
        >(x) else { unreachable!() };
        let Ok(x) = T5::cast::<
            InTuple6_5<F, T0::Output, T1::Output, T2::Output, T3::Output, T4::Output>,
        >(x) else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_0<F, A1, A2, A3, A4, A5, A6>(
    core::marker::PhantomData<(F, A1, A2, A3, A4, A5, A6)>,
);
impl<F: TypeCtor, A1, A2, A3, A4, A5, A6> TypeCtor
for InTuple7_0<F, A1, A2, A3, A4, A5, A6> {
    type Apply<X> = F::Apply<(X, A1, A2, A3, A4, A5, A6)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_1<F, A0, A2, A3, A4, A5, A6>(
    core::marker::PhantomData<(F, A0, A2, A3, A4, A5, A6)>,
);
impl<F: TypeCtor, A0, A2, A3, A4, A5, A6> TypeCtor
for InTuple7_1<F, A0, A2, A3, A4, A5, A6> {
    type Apply<X> = F::Apply<(A0, X, A2, A3, A4, A5, A6)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_2<F, A0, A1, A3, A4, A5, A6>(
    core::marker::PhantomData<(F, A0, A1, A3, A4, A5, A6)>,
);
impl<F: TypeCtor, A0, A1, A3, A4, A5, A6> TypeCtor
for InTuple7_2<F, A0, A1, A3, A4, A5, A6> {
    type Apply<X> = F::Apply<(A0, A1, X, A3, A4, A5, A6)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_3<F, A0, A1, A2, A4, A5, A6>(
    core::marker::PhantomData<(F, A0, A1, A2, A4, A5, A6)>,
);
impl<F: TypeCtor, A0, A1, A2, A4, A5, A6> TypeCtor
for InTuple7_3<F, A0, A1, A2, A4, A5, A6> {
    type Apply<X> = F::Apply<(A0, A1, A2, X, A4, A5, A6)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_4<F, A0, A1, A2, A3, A5, A6>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A5, A6)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A5, A6> TypeCtor
for InTuple7_4<F, A0, A1, A2, A3, A5, A6> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, X, A5, A6)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_5<F, A0, A1, A2, A3, A4, A6>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A6)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A6> TypeCtor
for InTuple7_5<F, A0, A1, A2, A3, A4, A6> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, X, A6)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple7_6<F, A0, A1, A2, A3, A4, A5>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A5)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A5> TypeCtor
for InTuple7_6<F, A0, A1, A2, A3, A4, A5> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, A5, X)>;
}
impl<
    T0: ToProxy,
    T1: ToProxy,
    T2: ToProxy,
    T3: ToProxy,
    T4: ToProxy,
    T5: ToProxy,
    T6: ToProxy,
> ToProxy for (T0, T1, T2, T3, T4, T5, T6) {
    type Output = (
        T0::Output,
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
    );
    fn to_proxy(self) -> Self::Output {
        (
            self.0.to_proxy(),
            self.1.to_proxy(),
            self.2.to_proxy(),
            self.3.to_proxy(),
            self.4.to_proxy(),
            self.5.to_proxy(),
            self.6.to_proxy(),
        )
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID && T3::IS_ID && T4::IS_ID
        && T5::IS_ID && T6::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple7_0<F, T1, T2, T3, T4, T5, T6>>(x) else {
            unreachable!()
        };
        let Ok(x) = T1::cast::<InTuple7_1<F, T0::Output, T2, T3, T4, T5, T6>>(x) else {
            unreachable!()
        };
        let Ok(x) = T2::cast::<InTuple7_2<F, T0::Output, T1::Output, T3, T4, T5, T6>>(x)
        else { unreachable!() };
        let Ok(x) = T3::cast::<
            InTuple7_3<F, T0::Output, T1::Output, T2::Output, T4, T5, T6>,
        >(x) else { unreachable!() };
        let Ok(x) = T4::cast::<
            InTuple7_4<F, T0::Output, T1::Output, T2::Output, T3::Output, T5, T6>,
        >(x) else { unreachable!() };
        let Ok(x) = T5::cast::<
            InTuple7_5<F, T0::Output, T1::Output, T2::Output, T3::Output, T4::Output, T6>,
        >(x) else { unreachable!() };
        let Ok(x) = T6::cast::<
            InTuple7_6<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T5::Output,
            >,
        >(x) else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_0<F, A1, A2, A3, A4, A5, A6, A7>(
    core::marker::PhantomData<(F, A1, A2, A3, A4, A5, A6, A7)>,
);
impl<F: TypeCtor, A1, A2, A3, A4, A5, A6, A7> TypeCtor
for InTuple8_0<F, A1, A2, A3, A4, A5, A6, A7> {
    type Apply<X> = F::Apply<(X, A1, A2, A3, A4, A5, A6, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_1<F, A0, A2, A3, A4, A5, A6, A7>(
    core::marker::PhantomData<(F, A0, A2, A3, A4, A5, A6, A7)>,
);
impl<F: TypeCtor, A0, A2, A3, A4, A5, A6, A7> TypeCtor
for InTuple8_1<F, A0, A2, A3, A4, A5, A6, A7> {
    type Apply<X> = F::Apply<(A0, X, A2, A3, A4, A5, A6, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_2<F, A0, A1, A3, A4, A5, A6, A7>(
    core::marker::PhantomData<(F, A0, A1, A3, A4, A5, A6, A7)>,
);
impl<F: TypeCtor, A0, A1, A3, A4, A5, A6, A7> TypeCtor
for InTuple8_2<F, A0, A1, A3, A4, A5, A6, A7> {
    type Apply<X> = F::Apply<(A0, A1, X, A3, A4, A5, A6, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_3<F, A0, A1, A2, A4, A5, A6, A7>(
    core::marker::PhantomData<(F, A0, A1, A2, A4, A5, A6, A7)>,
);
impl<F: TypeCtor, A0, A1, A2, A4, A5, A6, A7> TypeCtor
for InTuple8_3<F, A0, A1, A2, A4, A5, A6, A7> {
    type Apply<X> = F::Apply<(A0, A1, A2, X, A4, A5, A6, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_4<F, A0, A1, A2, A3, A5, A6, A7>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A5, A6, A7)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A5, A6, A7> TypeCtor
for InTuple8_4<F, A0, A1, A2, A3, A5, A6, A7> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, X, A5, A6, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_5<F, A0, A1, A2, A3, A4, A6, A7>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A6, A7)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A6, A7> TypeCtor
for InTuple8_5<F, A0, A1, A2, A3, A4, A6, A7> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, X, A6, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_6<F, A0, A1, A2, A3, A4, A5, A7>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A5, A7)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A5, A7> TypeCtor
for InTuple8_6<F, A0, A1, A2, A3, A4, A5, A7> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, A5, X, A7)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple8_7<F, A0, A1, A2, A3, A4, A5, A6>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A5, A6)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A5, A6> TypeCtor
for InTuple8_7<F, A0, A1, A2, A3, A4, A5, A6> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, A5, A6, X)>;
}
impl<
    T0: ToProxy,
    T1: ToProxy,
    T2: ToProxy,
    T3: ToProxy,
    T4: ToProxy,
    T5: ToProxy,
    T6: ToProxy,
    T7: ToProxy,
> ToProxy for (T0, T1, T2, T3, T4, T5, T6, T7) {
    type Output = (
        T0::Output,
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
    );
    fn to_proxy(self) -> Self::Output {
        (
            self.0.to_proxy(),
            self.1.to_proxy(),
            self.2.to_proxy(),
            self.3.to_proxy(),
            self.4.to_proxy(),
            self.5.to_proxy(),
            self.6.to_proxy(),
            self.7.to_proxy(),
        )
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID && T3::IS_ID && T4::IS_ID
        && T5::IS_ID && T6::IS_ID && T7::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple8_0<F, T1, T2, T3, T4, T5, T6, T7>>(x) else {
            unreachable!()
        };
        let Ok(x) = T1::cast::<InTuple8_1<F, T0::Output, T2, T3, T4, T5, T6, T7>>(x)
        else { unreachable!() };
        let Ok(x) = T2::cast::<
            InTuple8_2<F, T0::Output, T1::Output, T3, T4, T5, T6, T7>,
        >(x) else { unreachable!() };
        let Ok(x) = T3::cast::<
            InTuple8_3<F, T0::Output, T1::Output, T2::Output, T4, T5, T6, T7>,
        >(x) else { unreachable!() };
        let Ok(x) = T4::cast::<
            InTuple8_4<F, T0::Output, T1::Output, T2::Output, T3::Output, T5, T6, T7>,
        >(x) else { unreachable!() };
        let Ok(x) = T5::cast::<
            InTuple8_5<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T6,
                T7,
            >,
        >(x) else { unreachable!() };
        let Ok(x) = T6::cast::<
            InTuple8_6<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T5::Output,
                T7,
            >,
        >(x) else { unreachable!() };
        let Ok(x) = T7::cast::<
            InTuple8_7<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T5::Output,
                T6::Output,
            >,
        >(x) else { unreachable!() };
        Ok(x)
    }
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_0<F, A1, A2, A3, A4, A5, A6, A7, A8>(
    core::marker::PhantomData<(F, A1, A2, A3, A4, A5, A6, A7, A8)>,
);
impl<F: TypeCtor, A1, A2, A3, A4, A5, A6, A7, A8> TypeCtor
for InTuple9_0<F, A1, A2, A3, A4, A5, A6, A7, A8> {
    type Apply<X> = F::Apply<(X, A1, A2, A3, A4, A5, A6, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_1<F, A0, A2, A3, A4, A5, A6, A7, A8>(
    core::marker::PhantomData<(F, A0, A2, A3, A4, A5, A6, A7, A8)>,
);
impl<F: TypeCtor, A0, A2, A3, A4, A5, A6, A7, A8> TypeCtor
for InTuple9_1<F, A0, A2, A3, A4, A5, A6, A7, A8> {
    type Apply<X> = F::Apply<(A0, X, A2, A3, A4, A5, A6, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_2<F, A0, A1, A3, A4, A5, A6, A7, A8>(
    core::marker::PhantomData<(F, A0, A1, A3, A4, A5, A6, A7, A8)>,
);
impl<F: TypeCtor, A0, A1, A3, A4, A5, A6, A7, A8> TypeCtor
for InTuple9_2<F, A0, A1, A3, A4, A5, A6, A7, A8> {
    type Apply<X> = F::Apply<(A0, A1, X, A3, A4, A5, A6, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_3<F, A0, A1, A2, A4, A5, A6, A7, A8>(
    core::marker::PhantomData<(F, A0, A1, A2, A4, A5, A6, A7, A8)>,
);
impl<F: TypeCtor, A0, A1, A2, A4, A5, A6, A7, A8> TypeCtor
for InTuple9_3<F, A0, A1, A2, A4, A5, A6, A7, A8> {
    type Apply<X> = F::Apply<(A0, A1, A2, X, A4, A5, A6, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_4<F, A0, A1, A2, A3, A5, A6, A7, A8>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A5, A6, A7, A8)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A5, A6, A7, A8> TypeCtor
for InTuple9_4<F, A0, A1, A2, A3, A5, A6, A7, A8> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, X, A5, A6, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_5<F, A0, A1, A2, A3, A4, A6, A7, A8>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A6, A7, A8)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A6, A7, A8> TypeCtor
for InTuple9_5<F, A0, A1, A2, A3, A4, A6, A7, A8> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, X, A6, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_6<F, A0, A1, A2, A3, A4, A5, A7, A8>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A5, A7, A8)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A5, A7, A8> TypeCtor
for InTuple9_6<F, A0, A1, A2, A3, A4, A5, A7, A8> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, A5, X, A7, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_7<F, A0, A1, A2, A3, A4, A5, A6, A8>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A5, A6, A8)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A5, A6, A8> TypeCtor
for InTuple9_7<F, A0, A1, A2, A3, A4, A5, A6, A8> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, A5, A6, X, A8)>;
}
#[allow(dead_code, non_camel_case_types)]
struct InTuple9_8<F, A0, A1, A2, A3, A4, A5, A6, A7>(
    core::marker::PhantomData<(F, A0, A1, A2, A3, A4, A5, A6, A7)>,
);
impl<F: TypeCtor, A0, A1, A2, A3, A4, A5, A6, A7> TypeCtor
for InTuple9_8<F, A0, A1, A2, A3, A4, A5, A6, A7> {
    type Apply<X> = F::Apply<(A0, A1, A2, A3, A4, A5, A6, A7, X)>;
}
impl<
    T0: ToProxy,
    T1: ToProxy,
    T2: ToProxy,
    T3: ToProxy,
    T4: ToProxy,
    T5: ToProxy,
    T6: ToProxy,
    T7: ToProxy,
    T8: ToProxy,
> ToProxy for (T0, T1, T2, T3, T4, T5, T6, T7, T8) {
    type Output = (
        T0::Output,
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
    );
    fn to_proxy(self) -> Self::Output {
        (
            self.0.to_proxy(),
            self.1.to_proxy(),
            self.2.to_proxy(),
            self.3.to_proxy(),
            self.4.to_proxy(),
            self.5.to_proxy(),
            self.6.to_proxy(),
            self.7.to_proxy(),
            self.8.to_proxy(),
        )
    }
    const IS_ID: bool = T0::IS_ID && T1::IS_ID && T2::IS_ID && T3::IS_ID && T4::IS_ID
        && T5::IS_ID && T6::IS_ID && T7::IS_ID && T8::IS_ID;
    fn cast<F: TypeCtor>(
        x: F::Apply<Self>,
    ) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
        if !Self::IS_ID {
            return Err(x);
        }
        let Ok(x) = T0::cast::<InTuple9_0<F, T1, T2, T3, T4, T5, T6, T7, T8>>(x) else {
            unreachable!()
        };
        let Ok(x) = T1::cast::<InTuple9_1<F, T0::Output, T2, T3, T4, T5, T6, T7, T8>>(x)
        else { unreachable!() };
        let Ok(x) = T2::cast::<
            InTuple9_2<F, T0::Output, T1::Output, T3, T4, T5, T6, T7, T8>,
        >(x) else { unreachable!() };
        let Ok(x) = T3::cast::<
            InTuple9_3<F, T0::Output, T1::Output, T2::Output, T4, T5, T6, T7, T8>,
        >(x) else { unreachable!() };
        let Ok(x) = T4::cast::<
            InTuple9_4<F, T0::Output, T1::Output, T2::Output, T3::Output, T5, T6, T7, T8>,
        >(x) else { unreachable!() };
        let Ok(x) = T5::cast::<
            InTuple9_5<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T6,
                T7,
                T8,
            >,
        >(x) else { unreachable!() };
        let Ok(x) = T6::cast::<
            InTuple9_6<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T5::Output,
                T7,
                T8,
            >,
        >(x) else { unreachable!() };
        let Ok(x) = T7::cast::<
            InTuple9_7<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T5::Output,
                T6::Output,
                T8,
            >,
        >(x) else { unreachable!() };
        let Ok(x) = T8::cast::<
            InTuple9_8<
                F,
                T0::Output,
                T1::Output,
                T2::Output,
                T3::Output,
                T4::Output,
                T5::Output,
                T6::Output,
                T7::Output,
            >,
        >(x) else { unreachable!() };
        Ok(x)
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
/// The guard over an async import call while its wrapper waits (see `cancel_defs` in the
/// generator); `done` once it returns.
#[allow(dead_code)]
struct TraceCancel(Option<TaskId>);
impl TraceCancel {
    #[allow(dead_code)]
    fn done(mut self) {
        self.0 = None;
    }
}
impl Drop for TraceCancel {
    fn drop(&mut self) {
        if let Some(id) = self.0 {
            proxy::recorder::record::record(
                &Line {
                    by: Party::Guest,
                    at: None,
                    event: Event::Cancel(id),
                },
            );
            proxy::recorder::record::record(
                &Line {
                    by: Party::Host,
                    at: None,
                    event: Event::Cancelled(id),
                },
            );
        }
    }
}
thread_local! {
    static SENDER : std::cell::Cell < Party > = const { std::cell::Cell::new(Party::Host)
    };
}
#[allow(dead_code)]
const TRACE_TABLE: Table = Table::Imports;
#[allow(dead_code)]
fn trace_at() -> u32 {
    current_step()
}
/// The host's result of an async call: at once, if it is ready on the first poll, so
/// that the guest sees RETURNED, and otherwise from an executor job, which awaits the
/// host's subtask, with the step of the run it completed in, as replay returns it in
/// that step. The call may borrow the guest's arguments, e.g. a method's `self`.
#[allow(dead_code)]
async fn host_result<T: 'static>(
    id: TaskId,
    call: impl std::future::Future<Output = T>,
) -> (T, Option<u32>) {
    let mut call = Box::pin(call);
    let first = std::future::poll_fn(|cx| std::task::Poll::Ready(call.as_mut().poll(cx)))
        .await;
    if let std::task::Poll::Ready(value) = first {
        return (value, None);
    }
    proxy::recorder::record::record(
        &Line {
            by: Party::Host,
            at: None,
            event: Event::Started(id),
        },
    );
    let (value, at) = wait_for_job(call).await;
    (value, Some(at))
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
