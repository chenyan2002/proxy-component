use crate::codegen::util::{is_wrapped_module, make_path};
use crate::codegen::wit::WitType;
use crate::traits::Trait;
use quote::{format_ident, quote};
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct WaveTrait {
    pub to_value: bool,
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
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let mocked_resource = if self.has_replay_table {
            quote! {
                use std::{alloc::Layout, cell::RefCell};
                // Used to store borrowed resources when calling proxy::conversion during `Replay`
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
        let ast: syn::File = parse_quote! {
          #[allow(unused_imports)]
          use wasm_wave::{wasm::WasmValue, value::{Value, Type, convert::{ToRust, ToValue, ValueTyped}}};
          #mocked_resource
        };
        ast.items
    }
}
