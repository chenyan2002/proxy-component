mod bindings;
use bindings::*;
struct Stub;
bindings::export!(Stub with_types_in bindings);
impl exports::test::guest::api::Guest for Stub {
    fn run(
        p: exports::test::guest::api::Point,
        c: &exports::test::guest::api::Counter,
    ) -> Vec<u8> {
        let mut params: Vec<String> = Vec::new();
        let p = p.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&p)).unwrap());
        let c = c.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
        proxy::recorder::record::record_args(
            Some("test:guest/api@0.1.0.run"),
            &params,
            true,
        );
        let res = wrapped_test::guest::api::run(&p, &c);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record_ret(
            Some("test:guest/api@0.1.0.run"),
            Some(&wave_res),
            true,
        );
        res.to_proxy()
    }
    fn classify(
        s: exports::test::guest::api::Shape,
    ) -> ::core::option::Option<exports::test::guest::api::Point> {
        let mut params: Vec<String> = Vec::new();
        let s = s.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&s)).unwrap());
        proxy::recorder::record::record_args(
            Some("test:guest/api@0.1.0.classify"),
            &params,
            true,
        );
        let res = wrapped_test::guest::api::classify(&s);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record_ret(
            Some("test:guest/api@0.1.0.classify"),
            Some(&wave_res),
            true,
        );
        res.to_proxy()
    }
    fn begin(
        c: exports::test::guest::api::Counter,
    ) -> exports::test::guest::api::Counter {
        let mut params: Vec<String> = Vec::new();
        let c = c.to_proxy();
        params.push(wasm_wave::to_string(&ToValue::to_value(&c)).unwrap());
        proxy::recorder::record::record_args(
            Some("test:guest/api@0.1.0.begin"),
            &params,
            true,
        );
        let res = wrapped_test::guest::api::begin(c);
        let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
        proxy::recorder::record::record_ret(
            Some("test:guest/api@0.1.0.begin"),
            Some(&wave_res),
            true,
        );
        res.to_proxy()
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
impl ValueTyped for wrapped_test::host::types::Counter {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl<'a> ValueTyped for &'a wrapped_test::host::types::Counter {
    fn value_type() -> Type {
        Type::handle("counter")
    }
}
impl ToValue for wrapped_test::host::types::Counter {
    fn to_value(&self) -> Value {
        let label = format!("{}-{}", "counter", self.handle());
        Value::make_handle(label.into())
    }
}
impl<'a> ToValue for &'a wrapped_test::host::types::Counter {
    fn to_value(&self) -> Value {
        let label = format!("borrow-{}-{}", "counter", self.handle());
        Value::make_handle(label.into())
    }
}
#[allow(unused_imports)]
use wasm_wave::{
    wasm::WasmValue, value::{Value, Type, convert::{ToRust, ToValue, ValueTyped}},
};
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
    type Output = wrapped_test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        proxy::conversion::conversion::get_wrapped_test_host_types_counter(self)
    }
}
impl<'a> ToProxy for &'a test::host::types::Counter {
    type Output = &'a wrapped_test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        unreachable!()
    }
}
impl ToProxy for wrapped_test::host::types::Counter {
    type Output = test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        proxy::conversion::conversion::get_host_test_host_types_counter(self)
    }
}
impl<'a> ToProxy for &'a wrapped_test::host::types::Counter {
    type Output = &'a test::host::types::Counter;
    fn to_proxy(self) -> Self::Output {
        unreachable!()
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
