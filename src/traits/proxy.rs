use crate::codegen::State;
use crate::codegen::util::make_path;
use crate::codegen::wit::WitType;
use crate::traits::Trait;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{File, Item, ItemEnum, ItemStruct, parse_quote};

pub struct ProxyTrait<'a> {
    state: &'a State,
}
impl<'a> ProxyTrait<'a> {
    pub fn new(state: &'a State) -> Self {
        ProxyTrait { state }
    }
}

impl Trait for ProxyTrait<'_> {
    fn resource_trait(
        &self,
        module_path: &[String],
        resource: &ItemStruct,
        _wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let output_path = self.get_proxy_path(module_path);
        let output_owned = make_path(&output_path, &resource.ident.to_string());
        let in_import = module_path[0] != "exports";
        if in_import {
            let is_import_only = output_path[0] != "exports";
            if is_import_only {
                let call = self
                    .state
                    .conversion_func(&output_path, &resource.ident.to_string());
                let output_path = make_path(&output_path, &resource.ident.to_string());
                res.push(parse_quote! {
                    impl ToProxy for #resource_path {
                        type Output = #output_path;
                        fn to_proxy(self) -> Self::Output {
                            proxy::conversion::conversion::#call(self)
                        }
                    }
                });
                res.push(parse_quote! {
                    impl<'a> ToProxy for &'a #resource_path {
                        type Output = &'a #output_path;
                        fn to_proxy(self) -> Self::Output {
                            unreachable!()
                        }
                    }
                });
            } else {
                let export_borrow =
                    make_path(&output_path, &format!("{}Borrow<'a>", &resource.ident));
                res.push(parse_quote! {
                impl ToProxy for #resource_path {
                  type Output = #output_owned;
                  fn to_proxy(self) -> Self::Output {
                    Self::Output::new(self)
                  }
                }});
                res.push(parse_quote! {
                impl<'a> ToProxy for &'a #resource_path {
                  type Output = #export_borrow;
                  fn to_proxy(self) -> Self::Output {
                    unsafe { Self::Output::lift(self as *const _ as *const u8) }
                  }
                }});
            }
        } else {
            let export_borrow = make_path(module_path, &format!("{}Borrow<'a>", &resource.ident));
            res.push(parse_quote! {
                impl ToProxy for #resource_path {
                    type Output = #output_owned;
                    fn to_proxy(self) -> Self::Output {
                        self.into_inner()
                    }
                }
            });
            res.push(parse_quote! {
                impl<'a> ToProxy for #export_borrow {
                    type Output = &'a #output_owned;
                    fn to_proxy(self) -> Self::Output {
                        type T = #output_owned;
                        // only possible after wit-bindgen 0.48
                        self.get::<T>()
                    }
                }
            });
        }
        res
    }
    fn struct_trait(
        &self,
        module_path: &[String],
        struct_item: &ItemStruct,
        _wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let name = struct_item.ident.to_string();
        let struct_name = make_path(module_path, &name);
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let output_path = self.get_proxy_path(module_path);
        if !self.state.has_type_def(&output_path, &name) {
            let identity_cast = identity_cast();
            return vec![parse_quote! {
                impl #impl_generics ToProxy for #struct_name #ty_generics #where_clause {
                    type Output = #struct_name #ty_generics;
                    fn to_proxy(self) -> Self::Output {
                        self
                    }
                    #identity_cast
                }
            }];
        }
        let output_path = make_path(&output_path, &name);
        let fields = match &struct_item.fields {
            syn::Fields::Unit => quote! { Self::Output },
            syn::Fields::Named(fields) => {
                let field_names = fields.named.iter().map(|f| &f.ident);
                quote! { Self::Output { #(#field_names: self.#field_names.to_proxy()),* } }
            }
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        res.push(parse_quote! {
            impl #impl_generics ToProxy for #struct_name #ty_generics #where_clause {
                type Output = #output_path #ty_generics;
                fn to_proxy(self) -> Self::Output {
                    #fields
                }
            }
        });
        res
    }
    fn enum_trait(
        &self,
        module_path: &[String],
        enum_item: &ItemEnum,
        _wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let name = enum_item.ident.to_string();
        let enum_name = make_path(module_path, &name);
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let output_path = self.get_proxy_path(module_path);
        if !self.state.has_type_def(&output_path, &name) {
            let identity_cast = identity_cast();
            return vec![parse_quote! {
                impl #impl_generics ToProxy for #enum_name #ty_generics #where_clause {
                    type Output = #enum_name #ty_generics;
                    fn to_proxy(self) -> Self::Output {
                        self
                    }
                    #identity_cast
                }
            }];
        }
        let output_path = make_path(&output_path, &name);
        let match_arms = enum_item.variants.iter().map(|variant| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! { Self::#tag => Self::Output::#tag },
                syn::Fields::Unnamed(_) => {
                    quote! { Self::#tag(e) => Self::Output::#tag(e.to_proxy()) }
                }
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        res.push(parse_quote! {
            impl #impl_generics ToProxy for #enum_name #ty_generics #where_clause {
                type Output = #output_path #ty_generics;
                fn to_proxy(self) -> Self::Output {
                    match self {
                        #(#match_arms),*
                    }
                }
            }
        });
        res
    }
    fn flag_trait(
        &self,
        module_path: &[String],
        item: &crate::codegen::ItemFlag,
        _wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let name = item.name.to_string();
        let flag_name = make_path(module_path, &name);
        let output_path = self.get_proxy_path(module_path);
        if !self.state.has_type_def(&output_path, &name) {
            let identity_cast = identity_cast();
            return vec![parse_quote! {
                impl ToProxy for #flag_name {
                    type Output = #flag_name;
                    fn to_proxy(self) -> Self::Output {
                        self
                    }
                    #identity_cast
                }
            }];
        }
        let output_path = make_path(&output_path, &name);
        res.push(parse_quote! {
            impl ToProxy for #flag_name {
                type Output = #output_path;
                fn to_proxy(self) -> Self::Output {
                    Self::Output::from_bits_retain(self.bits())
                }
            }
        });
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let identity_cast = identity_cast();
        let defs: File = parse_quote! {
        trait ToProxy: Sized {
          type Output;
          fn to_proxy(self) -> Self::Output;
          // True iff `Output == Self`, for this type and all types nested in it.
          const IS_ID: bool = false;
          // Converts `Self` to `Output` inside any type constructor `F` (e.g. `Vec<Option<_>>`)
          // without touching the value. Returns `Ok` iff `IS_ID`; identity impls return `Ok(x)`, which
          // type-checks only because `Output == Self` there.
          fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
              Err(x)
          }
        }
        // A type constructor, i.e. a function from types to types. Rust can only express this as
        // a trait with a generic associated type: `F::Apply<X>` is `F` applied to `X`.
        //
        // How to read the impls below: each struct is one constructor, and its `Apply` says what
        // it produces. `InVec<F>` means "a `Vec` around `X`, then `F` around that", so
        // constructors nest inside-out: the outermost struct is the innermost wrapper.
        //   Id::Apply<X>                  = X
        //   InVec<Id>::Apply<X>           = Vec<X>
        //   InOption<InVec<Id>>::Apply<X> = Vec<Option<X>>
        //   InResultErr<Id, u8>::Apply<X> = Result<u8, X>
        // Types with several slots (`Result`, tuples) need one constructor per slot, since each
        // slot is a different function of `X`.
        //
        // The same impls read two other ways:
        // - Fill: `F` is a type with one hole, and `Apply<X>` fills the hole with `X`.
        //   `InVec<Id>` is `Vec<_>`, `InOption<InVec<Id>>` is `Vec<Option<_>>`, and
        //   `InResultErr<Id, u8>` is `Result<u8, _>`.
        // - Family pattern: `F` names a family of types indexed by `X`, and `Apply<X>` is the
        //   member at index `X`. `InVec<Id>` is the family {`Vec<u8>`, `Vec<String>`, ...}, and
        //   `InVec<Id>::Apply<u8>` picks out `Vec<u8>`. See
        //   https://smallcultfollowing.com/babysteps/blog/2016/11/03/associated-type-constructors-part-2-family-traits/
        //
        // `ToProxy::cast` uses this as Leibniz equality: if `A == B`, then `F::Apply<A>` and
        // `F::Apply<B>` are the same type for every `F`. An identity impl (`Output == Self`) can
        // therefore return its `F::Apply<Self>` argument as `F::Apply<Self::Output>` unchanged, no
        // matter how deeply `Self` is nested in containers. `cast` is the Rust analog of Haskell's
        // `subst :: forall c. c a -> c b` in
        // https://hackage.haskell.org/package/eq/docs/Data-Eq-Type.html
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
            #identity_cast
        }
        impl<T: crate::ToProxy> crate::ToProxy for Vec::<T> {
            type Output = Vec::<T::Output>;
            fn to_proxy(self) -> Self::Output {
                match T::cast::<InVec<Id>>(self) {
                    Ok(v) => v,
                    Err(v) => v.into_iter().map(|x| x.to_proxy()).collect(),
                }
            }
            const IS_ID: bool = T::IS_ID;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                T::cast::<InVec<F>>(x)
            }
        }
        impl<T, E> ToProxy for Result<T, E>
        where T: ToProxy, E: ToProxy {
            type Output = Result<T::Output, E::Output>;
            fn to_proxy(self) -> Self::Output {
                match self {
                    Ok(ok) => Ok(ok.to_proxy()),
                    Err(err) => Err(err.to_proxy()),
                }
            }
            const IS_ID: bool = T::IS_ID && E::IS_ID;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                // Check first, so that we never fail after casting only one side.
                if !Self::IS_ID {
                    return Err(x);
                }
                let Ok(x) = T::cast::<InResultOk<F, E>>(x) else { unreachable!() };
                let Ok(x) = E::cast::<InResultErr<F, T::Output>>(x) else { unreachable!() };
                Ok(x)
            }
        }
        impl<Inner> ToProxy for Option<Inner>
        where Inner: ToProxy {
            type Output = Option<Inner::Output>;
            fn to_proxy(self) -> Self::Output {
                self.map(|x| x.to_proxy())
            }
            const IS_ID: bool = Inner::IS_ID;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                Inner::cast::<InOption<F>>(x)
            }
        }
        macro_rules! impl_to_import_export_for_primitive {
            ($($t:ty),*) => {
                $(
                    impl ToProxy for $t {
                        type Output = $t;
                        fn to_proxy(self) -> Self::Output {
                            self
                        }
                        #identity_cast
                    }
                )*
            };
        }
        impl_to_import_export_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64, (), bool, char);
        };
        let mut items = defs.items;
        for n in 1..=9 {
            items.extend(tuple_trait(n));
        }
        items.extend(pump_defs(self.state.has_streams, self.state.has_futures));
        items
    }
}
/// Record mode's pumps: a stream or future the proxy receives, e.g. the host's in an import's
/// result, isn't passed on, as the copies would then go between the host and the guest without
/// it. Its conversion makes a pair of the other side's type, hands on its reader, and spawns a
/// job, the pump, which copies from the end it received to the pair's writer, and records each
/// copy, and each side's drop. The stream is labeled by the end the proxy received, which the pump
/// keeps for its lifetime. `SENDER` says who sent it.
///
/// A call's arguments are printed converted, and its result as received, so the reader handed
/// on in the arguments is printed by the received end's label too, which `label_by_writer`
/// registers, and so are the items of a stream or future the caller sends.
fn pump_defs(has_streams: bool, has_futures: bool) -> Vec<Item> {
    let mut items = crate::traits::host_end_defs(has_streams, has_futures);
    if !has_streams && !has_futures {
        return items;
    }
    let file: File = parse_quote! {
        /// The other party.
        fn trace_receiver(sender: Party) -> Party {
            match sender {
                Party::Host => Party::Guest,
                Party::Guest => Party::Host,
            }
        }
        /// A line of `by`, which has a step if it is the host's.
        fn pump_line(by: Party, at: u32, event: Event) -> Line {
            let at = matches!(by, Party::Host).then_some(at);
            Line { by, at, event }
        }
        /// Whether the caller of the call sends the value, rather than the callee: the guest's
        /// imports component, and the host's exports component, which call.
        fn sent_by_caller(sender: Party) -> bool {
            matches!(TRACE_TABLE, Table::Imports) == matches!(sender, Party::Guest)
        }
    };
    items.extend(file.items);
    if has_streams {
        let file: File = parse_quote! {
            impl<T> ToProxy for wit_bindgen::StreamReader<T>
            where
                T: ToProxy + ToValue + wit_stream::StreamPayload + 'static,
                T::Output: ToValue + wit_stream::StreamPayload + 'static,
            {
                type Output = wit_bindgen::StreamReader<T::Output>;
                fn to_proxy(self) -> Self::Output {
                    let (writer, reader) = wit_stream::new::<T::Output>();
                    let sender = SENDER.get();
                    if sent_by_caller(sender) {
                        label_by_writer(reader.handle(), self.handle());
                    }
                    let label = Label { table: TRACE_TABLE, name: handle_label(&self) };
                    spawn(pump_stream(self, writer, label, sender));
                    reader
                }
            }
            /// Copies the items of a stream from `from`, which `sender` writes, to `to`, which
            /// the other party reads, once both are ready, so that each copy completes in the
            /// run it's issued in, and its line is recorded before either party goes on, e.g.
            /// the guest traps:
            ///
            /// - A zero-length write waits until the reader reads, so that the writer is read
            ///   only then, as without the proxy. A host's end may answer it at once, without
            ///   being ready, e.g. wasmtime's, which then takes a chunk of the guest's early.
            /// - A zero-length read waits until the writer writes, leaving its write pending.
            async fn pump_stream<T, U>(
                mut from: wit_bindgen::StreamReader<T>,
                mut to: wit_bindgen::StreamWriter<U>,
                label: Label,
                sender: Party,
            ) where
                T: ToProxy<Output = U> + ToValue + wit_stream::StreamPayload,
                U: ToValue + wit_stream::StreamPayload,
            {
                let receiver = trace_receiver(sender);
                let wave = |item: &dyn ToValue| wasm_wave::to_string(&item.to_value()).unwrap();
                let dropped = |by: Party| {
                    proxy::recorder::record::record(&pump_line(by, trace_at(), Event::Drop(label.clone())));
                };
                loop {
                    let (ready, _) = to.write(Vec::new()).await;
                    if let wit_bindgen::StreamResult::Dropped = ready {
                        return dropped(receiver);
                    }
                    let (ready, _) = from.read(Vec::new()).await;
                    if let wit_bindgen::StreamResult::Dropped = ready {
                        return dropped(sender);
                    }
                    let read_at = trace_at();
                    let (result, items) = from.read(Vec::with_capacity(4096)).await;
                    let received: Vec<String> = items.iter().map(|item| wave(item)).collect();
                    let mut items: Vec<U> = items.into_iter().map(ToProxy::to_proxy).collect();
                    let waves = if sent_by_caller(sender) {
                        items.iter().map(|item| wave(item)).collect()
                    } else {
                        received
                    };
                    // The host's read of the guest's items, in its step.
                    if matches!(sender, Party::Guest) && !items.is_empty() {
                        let chunk = Chunk { stream: label.clone(), items: format!("[{}]", waves.join(", ")) };
                        proxy::recorder::record::record(&pump_line(Party::Host, read_at, Event::Read(chunk)));
                    }
                    // The host's write of its items, a line per part the guest takes.
                    let mut offset = 0;
                    while !items.is_empty() {
                        let (write_at, count) = (trace_at(), items.len());
                        let (written_result, rest) = to.write(items).await;
                        let rest = rest.into_vec();
                        let written = count - rest.len();
                        if matches!(sender, Party::Host) && written > 0 {
                            let items = format!("[{}]", waves[offset..offset + written].join(", "));
                            let chunk = Chunk { stream: label.clone(), items };
                            proxy::recorder::record::record(&pump_line(Party::Host, write_at, Event::Write(chunk)));
                        }
                        offset += written;
                        if let wit_bindgen::StreamResult::Dropped = written_result {
                            return dropped(receiver);
                        }
                        items = rest;
                        if !items.is_empty() {
                            let (ready, _) = to.write(Vec::new()).await;
                            if let wit_bindgen::StreamResult::Dropped = ready {
                                return dropped(receiver);
                            }
                        }
                    }
                    if let wit_bindgen::StreamResult::Dropped = result {
                        return dropped(sender);
                    }
                }
            }
        };
        items.extend(file.items);
    }
    if has_futures {
        let file: File = parse_quote! {
            impl<T> ToProxy for wit_bindgen::FutureReader<T>
            where
                T: ToProxy + ToValue + wit_future::FuturePayload + 'static,
                T::Output: ToValue + wit_future::FuturePayload + 'static,
            {
                type Output = wit_bindgen::FutureReader<T::Output>;
                fn to_proxy(self) -> Self::Output {
                    // The pump always writes the value, unless the reader is dropped first.
                    let (writer, reader) =
                        wit_future::new::<T::Output>(|| unreachable!("the value is always written"));
                    let sender = SENDER.get();
                    if sent_by_caller(sender) {
                        label_by_writer(reader.handle(), self.handle());
                    }
                    let label = Label { table: TRACE_TABLE, name: handle_label(&self) };
                    spawn(pump_future(self, writer, label, sender));
                    reader
                }
            }
            /// Passes the value of a future from `from`, which `sender` writes, to `to`, which
            /// the other party reads, unless it drops it first. The value's line is recorded as
            /// the pump writes it, before the reader can take it and go on, e.g. trap; a drop
            /// follows if the reader drops its end instead. A future can't wait until the other
            /// side reads: the host's read of the guest's value is recorded once it completes.
            async fn pump_future<T, U>(
                from: wit_bindgen::FutureReader<T>,
                to: wit_bindgen::FutureWriter<U>,
                label: Label,
                sender: Party,
            ) where
                T: ToProxy<Output = U> + ToValue + wit_future::FuturePayload,
                U: ToValue + wit_future::FuturePayload,
            {
                let value = from.await;
                let received = wasm_wave::to_string(&value.to_value()).unwrap();
                let value = value.to_proxy();
                let wave = if sent_by_caller(sender) {
                    wasm_wave::to_string(&value.to_value()).unwrap()
                } else {
                    received
                };
                let value_line = Event::Value(FutureValue { future: label.clone(), value: wave });
                proxy::recorder::record::record(&pump_line(sender, trace_at(), value_line));
                if to.write(value).await.is_err() {
                    let line = pump_line(trace_receiver(sender), trace_at(), Event::Drop(label));
                    proxy::recorder::record::record(&line);
                }
            }
        };
        items.extend(file.items);
    }
    items
}
// Overrides `ToProxy::cast` for impls where `Output == Self`.
fn identity_cast() -> TokenStream {
    quote! {
        const IS_ID: bool = true;
        fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
            Ok(x)
        }
    }
}
// `ToProxy` for an n-tuple, plus one `TypeCtor` per position for `cast`.
fn tuple_trait(n: usize) -> Vec<Item> {
    let mut res = Vec::new();
    let tys: Vec<_> = (0..n).map(|i| format_ident!("T{i}")).collect();
    let idx = (0..n).map(syn::Index::from);
    let mut casts = Vec::new();
    for i in 0..n {
        let ctx = format_ident!("InTuple{n}_{i}");
        let params: Vec<_> = (0..n)
            .filter(|j| *j != i)
            .map(|j| format_ident!("A{j}"))
            .collect();
        let slots = (0..n).map(|j| {
            if j == i {
                quote! { X }
            } else {
                let a = format_ident!("A{j}");
                quote! { #a }
            }
        });
        res.push(parse_quote! {
            #[allow(dead_code, non_camel_case_types)]
            struct #ctx<F, #(#params),*>(core::marker::PhantomData<(F, #(#params),*)>);
        });
        res.push(parse_quote! {
            impl<F: TypeCtor, #(#params),*> TypeCtor for #ctx<F, #(#params),*> {
                type Apply<X> = F::Apply<(#(#slots,)*)>;
            }
        });
        // Positions before `i` have already been cast to their `Output`.
        let args = (0..n).filter(|j| *j != i).map(|j| {
            let t = &tys[j];
            if j < i {
                quote! { #t::Output }
            } else {
                quote! { #t }
            }
        });
        let ti = &tys[i];
        casts.push(quote! {
            let Ok(x) = #ti::cast::<#ctx<F, #(#args),*>>(x) else { unreachable!() };
        });
    }
    res.push(parse_quote! {
        impl<#(#tys: ToProxy),*> ToProxy for (#(#tys,)*) {
            type Output = (#(#tys::Output,)*);
            fn to_proxy(self) -> Self::Output {
                (#(self.#idx.to_proxy(),)*)
            }
            const IS_ID: bool = #(#tys::IS_ID)&&*;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                // Check first, so that we never fail after casting only some positions.
                if !Self::IS_ID {
                    return Err(x);
                }
                #(#casts)*
                Ok(x)
            }
        }
    });
    res
}
impl ProxyTrait<'_> {
    fn get_proxy_path(&self, src_path: &[String]) -> Vec<String> {
        let from_export = src_path[0] == "exports";
        let mut res = crate::codegen::get_proxy_path(src_path);
        if !from_export && !self.state.module_paths.contains(&res) {
            res.remove(0);
        }
        assert!(
            self.state.module_paths.contains(&res),
            "no module {res:?} for the types of {src_path:?}"
        );
        res
    }
}
