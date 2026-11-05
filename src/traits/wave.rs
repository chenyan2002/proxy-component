use crate::codegen::util::{is_wrapped_module, make_path};
use crate::codegen::wit::WitType;
use crate::traits::{Trait, new_mock};
use quote::{format_ident, quote};
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct WaveTrait {
    pub to_value: bool,
    pub to_rust: bool,
    pub has_replay_table: bool,
    /// Label wrapped host resources by their host handles, see `proxy:conversion`
    pub wrapped_labels: bool,
    /// Label mocked host resources by the ids the imports component gives them, see
    /// `proxy:conversion`
    pub mock_labels: bool,
}

impl Trait for WaveTrait {
    fn resource_trait(
        &self,
        module_path: &[String],
        resource: &ItemStruct,
        wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = &wit.name;
        let in_import = module_path[0] != "exports";
        res.push(parse_quote! {
            impl ValueTyped for #resource_path {
                fn value_type() -> Type {
                    Type::handle(#wit_name)
                }
            }
        });
        if in_import {
            res.push(parse_quote! {
            impl<'a> ValueTyped for &'a #resource_path {
                fn value_type() -> Type {
                    Type::handle(#wit_name)
                }
            }
            });
            if self.to_value {
                // A wrapped host resource is labeled by its host handle, which the component
                // wrapping it knows, and a mock by its id, which the component mocking it knows,
                // so that both generated components label it alike.
                let handle = match &wit.conversion {
                    Some(conversion)
                        if self.mock_labels
                            || (self.wrapped_labels && is_wrapped_module(&module_path[0])) =>
                    {
                        let conversion = conversion.to_string();
                        let get_handle = format_ident!(
                            "get_handle_{}",
                            conversion.strip_prefix("get_").unwrap()
                        );
                        quote! { proxy::conversion::conversion::#get_handle(self) }
                    }
                    _ => quote! { self.handle() },
                };
                res.push(parse_quote! {
                impl ToValue for #resource_path {
                    fn to_value(&self) -> Value {
                        let label = format!("{}-{}", #wit_name, #handle);
                        Value::make_handle(label.into())
                    }
                }
                });
                res.push(parse_quote! {
                impl<'a> ToValue for &'a #resource_path {
                    fn to_value(&self) -> Value {
                        let label = format!("borrow-{}-{}", #wit_name, #handle);
                        Value::make_handle(label.into())
                    }
                }
                });
            }
            if self.to_rust {
                let mock = new_mock(wit, quote! { handle });
                res.push(parse_quote! {
                impl FromWave for #resource_path {
                    fn from_wave(value: &Value) -> Self {
                        let label = value.unwrap_handle();
                        let handle = label
                            .strip_prefix(&format!("{}-", #wit_name))
                            .and_then(|s| s.parse::<u32>().ok())
                            .expect("invalid handle label");
                        #mock
                    }
                }
                });
                res.push(parse_quote! {
                impl<'a> FromWave for &'a #resource_path {
                    fn from_wave(value: &Value) -> Self {
                        let label = value.unwrap_handle();
                        let handle = label
                            .strip_prefix(&format!("borrow-{}-", #wit_name))
                            .and_then(|s| s.parse::<u32>().ok())
                            .expect("invalid handle label");
                        SCOPED_ALLOC.with(|alloc| {
                            let mut alloc = alloc.borrow_mut();
                            alloc.alloc(#mock)
                        })
                    }
                }
                });
            }
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl<'a> ValueTyped for #borrow_path {
                fn value_type() -> Type {
                    Type::handle(#wit_name)
                }
            }
            });
            if self.to_value {
                if self.has_replay_table {
                    res.push(parse_quote! {
                    impl ToValue for #resource_path {
                        fn to_value(&self) -> Value {
                            let handle = self.get::<MockedResource>().handle;
                            let label = format!("{}-{}", #wit_name, handle);
                            Value::make_handle(label.into())
                        }
                    }});
                    res.push(parse_quote! {
                    impl<'a> ToValue for #borrow_path {
                        fn to_value(&self) -> Value {
                            let handle = self.get::<MockedResource>().handle;
                            let label = format!("borrow-{}-{}", #wit_name, handle);
                            Value::make_handle(label.into())
                        }
                    }});
                } else {
                    res.push(parse_quote! {
                    impl ToValue for #resource_path {
                        fn to_value(&self) -> Value {
                            let label = format!("{}-{}", #wit_name, self.handle());
                            Value::make_handle(label.into())
                        }
                    }
                    });
                    let proxy_path = crate::codegen::get_proxy_path(module_path);
                    let proxy_path = make_path(&proxy_path, &resource.ident.to_string());
                    res.push(parse_quote! {
                    impl<'a> ToValue for #borrow_path {
                        fn to_value(&self) -> Value {
                            type T = #proxy_path;
                            let label = format!("borrow-{}-{}", #wit_name, self.get::<T>().handle());
                            Value::make_handle(label.into())
                        }
                    }
                    });
                }
            }
            if self.to_rust && self.has_replay_table {
                res.push(parse_quote! {
                impl FromWave for #resource_path {
                    fn from_wave(value: &Value) -> Self {
                        let label = value.unwrap_handle();
                        let expect_handle = label
                            .strip_prefix(&format!("{}-", #wit_name))
                            .and_then(|s| s.parse::<u32>().ok())
                            .expect("invalid handle label");
                        let handle = #resource_path::new(MockedResource {
                            handle: expect_handle,
                            name: #wit_name.to_string(),
                        });
                        // handle.handle() is not expected to match expect_handle, because each component
                        // has its own resource table, and the order of resource creation can be different.
                        // However, for the user component, the handle id should match, but we don't have a
                        // way to check that at the moment.
                        handle
                    }
                }
                });
                res.push(parse_quote! {
                impl<'a> FromWave for #borrow_path {
                    fn from_wave(_value: &Value) -> Self {
                        unreachable!()
                    }
                }
                });
            }
        }
        res
    }
    fn struct_trait(
        &self,
        module_path: &[String],
        struct_item: &ItemStruct,
        wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let wit_names = &wit.members;
        let (field_names, tys) = match &struct_item.fields {
            syn::Fields::Unit => (Vec::new(), Vec::new()),
            syn::Fields::Named(fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect();
                let field_tys = fields.named.iter().map(|f| &f.ty).collect();
                (field_names, field_tys)
            }
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        res.push(parse_quote! {
            impl #impl_generics ValueTyped for #struct_name #ty_generics #where_clause {
                #[allow(unused_imports)]
                fn value_type() -> Type {
                    let fields = vec![
                        #((#wit_names, <#tys as ValueTyped>::value_type())),*
                    ];
                    Type::record(fields).unwrap()
                }
            }
        });
        if self.to_value {
            res.push(parse_quote! {
            impl #impl_generics ToValue for #struct_name #ty_generics #where_clause {
                fn to_value(&self) -> Value {
                    let ty = #struct_name::value_type();
                    let fields = vec![
                        #((#wit_names, self.#field_names.to_value())),*
                    ];
                    Value::make_record(&ty, fields).unwrap()
                }
            }
            });
        }
        if self.to_rust {
            res.push(parse_quote! {
            impl #impl_generics FromWave for #struct_name #ty_generics #where_clause {
                #[allow(unused_variables)]
                fn from_wave(value: &Value) -> Self {
                    let fields: std::collections::BTreeMap<_, _> = value.unwrap_record().collect();
                    #struct_name {
                        #(#field_names: FromWave::from_wave(&fields[#wit_names])),*
                    }
                }
            }
            });
        }
        res
    }
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum, wit: &WitType) -> Vec<Item> {
        let mut res = Vec::new();
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let cases = enum_item
            .variants
            .iter()
            .zip(&wit.members)
            .map(|(variant, wit_name)| match &variant.fields {
                syn::Fields::Unit => quote! { (#wit_name, None) },
                syn::Fields::Unnamed(f) => {
                    assert!(f.unnamed.len() == 1);
                    let ty = &f.unnamed.first().unwrap().ty;
                    quote! { (#wit_name, Some(<#ty as ValueTyped>::value_type())) }
                }
                syn::Fields::Named(_) => unreachable!(),
            });
        res.push(parse_quote! {
            impl #impl_generics ValueTyped for #enum_name #ty_generics #where_clause {
                #[allow(unused_imports)]
                fn value_type() -> Type {
                    let cases = vec![#(#cases),*];
                    Type::variant(cases).unwrap()
                }
            }
        });
        if self.to_value {
            let arms = enum_item.variants.iter().zip(&wit.members).map(|(variant, wit_name)| {
                let tag = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unit => quote! { Self::#tag => Value::make_variant(&ty, #wit_name, None) },
                    syn::Fields::Unnamed(_) => {
                        quote! { Self::#tag(e) => Value::make_variant(&ty, #wit_name, Some(e.to_value())) }
                    }
                    syn::Fields::Named(_) => unreachable!(),
                }
            });
            res.push(parse_quote! {
            impl #impl_generics ToValue for #enum_name #ty_generics #where_clause {
                fn to_value(&self) -> Value {
                    let ty = #enum_name::value_type();
                    match self {
                        #(#arms),*
                    }.unwrap()
                }
            }
            });
        }
        if self.to_rust {
            let arms = enum_item.variants.iter().zip(&wit.members).map(|(variant, wit_name)| {
                let tag = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unit => quote! {
                        (ref case, None) if case == #wit_name => #enum_name::#tag
                    },
                    syn::Fields::Unnamed(_) => quote! {
                        (ref case, Some(val)) if case == #wit_name => #enum_name::#tag(FromWave::from_wave(&val))
                    },
                    syn::Fields::Named(_) => unreachable!(),
                }
            });
            res.push(parse_quote! {
            impl #impl_generics FromWave for #enum_name #ty_generics #where_clause {
                fn from_wave(value: &Value) -> Self {
                    match value.unwrap_variant() {
                        #(#arms),*,
                        _ => unreachable!(),
                    }
                }
            }
            });
        }
        res
    }
    fn flag_trait(
        &self,
        module_path: &[String],
        item: &crate::codegen::ItemFlag,
        wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let flag_path = make_path(module_path, &item.name.to_string());
        let wit_flags = &wit.members;
        res.push(parse_quote! {
            impl ValueTyped for #flag_path {
                fn value_type() -> Type {
                    Type::flags(vec![#(#wit_flags),*]).unwrap()
                }
            }
        });
        if self.to_value {
            let check_flags = item.flags.iter().zip(&wit.members).map(|(f, wit_name)| {
                quote! {
                    if *self & Self::#f != Self::empty() {
                        flags.push(#wit_name);
                    }
                }
            });
            res.push(parse_quote! {
            impl ToValue for #flag_path {
                fn to_value(&self) -> Value {
                    let ty = Self::value_type();
                    let mut flags = Vec::new();
                    #(#check_flags)*
                    Value::make_flags(&ty, flags).unwrap()
                }
            }
            });
        }
        if self.to_rust {
            let set_flags = item.flags.iter().zip(&wit.members).map(|(f, wit_name)| {
                quote! {
                    if flags.iter().any(|flag| flag == #wit_name) {
                        res |= #flag_path::#f;
                    }
                }
            });
            res.push(parse_quote! {
            impl FromWave for #flag_path {
                fn from_wave(value: &Value) -> Self {
                    let flags: Vec<_> = value.unwrap_flags().map(|f| f.into_owned()).collect();
                    let mut res = #flag_path::empty();
                    #(#set_flags)*
                    res
                }
            }
            });
        }
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let mocked_resource = if self.has_replay_table {
            quote! {
                use std::{alloc::Layout, cell::RefCell};
                // Used to store borrowed resources when calling proxy::conversion during ToRust trait
                #[allow(dead_code)]
                struct ScopedAlloc {
                    ptrs: Vec<(*mut u8, Layout, fn(*mut u8))>,
                }
                thread_local! {
                    static SCOPED_ALLOC: RefCell<ScopedAlloc> = RefCell::new(ScopedAlloc::new());
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
                        self.ptrs.push((
                            ptr as *mut u8,
                            Layout::new::<T>(),
                            drop_ptr::<T>,
                        ));
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
            }
        } else {
            quote! {}
        };
        let mocked_from_wave = if self.has_replay_table {
            quote! {
            // Only called by constructor
            impl FromWave for MockedResource {
                fn from_wave(value: &Value) -> Self {
                    let label = value.unwrap_handle();
                    let (name, handle) = label
                        .rsplit_once('-')
                        .and_then(|(name, handle_str)| {
                            handle_str.parse::<u32>().ok().map(|h| (name.to_string(), h))
                        })
                        .expect("invalid handle label");
                    MockedResource {
                        handle,
                        name,
                    }
                }
            }
            }
        } else {
            quote! {}
        };
        let from_wave = if self.to_rust {
            quote! {
                /// A value of the trace, as the Rust type of the bindings: wasm-wave's `ToRust`, as a
                /// local trait, so that it can be implemented for wit-bindgen's streams, futures and
                /// error-contexts, as `Fuzz` is for `Arbitrary`.
                trait FromWave: Sized {
                    fn from_wave(value: &Value) -> Self;
                    /// From the payload of a result's or a variant's case, which `()` has none of.
                    fn from_payload(value: Option<&Value>) -> Self {
                        Self::from_wave(value.expect("the case has a payload"))
                    }
                }
                macro_rules! impl_from_wave_primitive {
                    ($($t:ty),*) => {
                        $(
                            impl FromWave for $t {
                                fn from_wave(value: &Value) -> Self {
                                    value.to_rust()
                                }
                            }
                        )*
                    };
                }
                impl_from_wave_primitive!(bool, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, char, String);
                impl FromWave for () {
                    fn from_wave(_value: &Value) -> Self {}
                    fn from_payload(_value: Option<&Value>) -> Self {}
                }
                impl<T: FromWave> FromWave for Vec<T> {
                    fn from_wave(value: &Value) -> Self {
                        value.unwrap_list().map(|x| T::from_wave(&x)).collect()
                    }
                }
                impl<T: FromWave> FromWave for Option<T> {
                    fn from_wave(value: &Value) -> Self {
                        value.unwrap_option().map(|x| T::from_wave(&x))
                    }
                }
                impl<T: FromWave, E: FromWave> FromWave for core::result::Result<T, E> {
                    fn from_wave(value: &Value) -> Self {
                        match value.unwrap_result() {
                            Ok(x) => Ok(T::from_payload(x.as_deref())),
                            Err(x) => Err(E::from_payload(x.as_deref())),
                        }
                    }
                }
                macro_rules! impl_from_wave_tuple {
                    ($($T:ident),+) => {
                        impl<$($T: FromWave),+> FromWave for ($($T,)+) {
                            fn from_wave(value: &Value) -> Self {
                                let mut items = value.unwrap_tuple();
                                ($($T::from_wave(&items.next().unwrap()),)+)
                            }
                        }
                    };
                }
                impl_from_wave_tuple!(T1);
                impl_from_wave_tuple!(T1, T2);
                impl_from_wave_tuple!(T1, T2, T3);
                impl_from_wave_tuple!(T1, T2, T3, T4);
                impl_from_wave_tuple!(T1, T2, T3, T4, T5);
                impl_from_wave_tuple!(T1, T2, T3, T4, T5, T6);
                impl_from_wave_tuple!(T1, T2, T3, T4, T5, T6, T7);
                impl_from_wave_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
                impl_from_wave_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
                // The host's ends of streams and futures, which replay makes from the trace (phase
                // 3b-2 of `docs/p3-record-replay.md`).
                impl<T: 'static> FromWave for wit_bindgen::StreamReader<T> {
                    fn from_wave(_value: &Value) -> Self {
                        unimplemented!("replaying a stream")
                    }
                }
                impl<T: 'static> FromWave for wit_bindgen::FutureReader<T> {
                    fn from_wave(_value: &Value) -> Self {
                        unimplemented!("replaying a future")
                    }
                }
                // The trace doesn't have an error-context's message, only its label.
                impl FromWave for wit_bindgen::rt::async_support::ErrorContext {
                    fn from_wave(_value: &Value) -> Self {
                        Self::new("")
                    }
                }
                #mocked_from_wave
            }
        } else {
            quote! {}
        };
        let ast: syn::File = parse_quote! {
          #[allow(unused_imports)]
          use wasm_wave::{wasm::WasmValue, value::{Value, Type, convert::{ToRust, ToValue, ValueTyped}}};
          #mocked_resource
          #from_wave
        };
        ast.items
    }
}
