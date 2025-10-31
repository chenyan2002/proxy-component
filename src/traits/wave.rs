use crate::codegen::make_path;
use crate::traits::Trait;
use heck::ToKebabCase;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct WaveTrait {
    pub to_value: bool,
    pub to_rust: bool,
    pub has_replay_table: bool,
}

impl Trait for WaveTrait {
    fn resource_trait(&self, module_path: &[String], resource: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = resource.ident.to_string().to_kebab_case();
        let in_import = module_path[0] != "exports";
        res.push(parse_quote! {
            impl ValueTyped for #resource_path {
                fn value_type() -> Type {
                    Type::resource(#wit_name, false)
                }
            }
        });
        if in_import {
            res.push(parse_quote! {
            impl<'a> ValueTyped for &'a #resource_path {
                fn value_type() -> Type {
                    Type::resource(#wit_name, true)
                }
            }
            });
            if self.to_value {
                res.push(parse_quote! {
                impl ToValue for #resource_path {
                    fn to_value(&self) -> Value {
                        Value::make_resource(&#resource_path::value_type(), self.handle(), false).unwrap()
                    }
                }
                });
                res.push(parse_quote! {
                impl<'a> ToValue for &'a #resource_path {
                    fn to_value(&self) -> Value {
                        Value::make_resource(&#resource_path::value_type(), self.handle(), true).unwrap()
                    }
                }
                });
            }
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl<'a> ValueTyped for #borrow_path {
                fn value_type() -> Type {
                    Type::resource(#wit_name, true)
                }
            }
            });
            if self.to_value && self.has_replay_table {
                res.push(parse_quote! {
                impl ToValue for #resource_path {
                    fn to_value(&self) -> Value {
                        let ptr = self.as_ptr::<Stub>() as u32;
                        let handle = TABLE.with(|map| map.borrow().get(&ptr).unwrap().clone());
                        Value::make_resource(&#resource_path::value_type(), handle, false).unwrap()
                    }
                }});
                res.push(parse_quote! {
                impl<'a> ToValue for #borrow_path {
                    fn to_value(&self) -> Value {
                        let ptr = self.as_ptr::<Stub>() as u32;
                        let handle = TABLE.with(|map| map.borrow().get(&ptr).unwrap().clone());
                        Value::make_resource(&#resource_path::value_type(), handle, true).unwrap()
                    }
                }});
            }
        }
        res
    }
    fn struct_trait(&self, module_path: &[String], struct_item: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let use_path: syn::Path = syn::parse_str(&module_path.join("::")).unwrap();
        let in_import = module_path[0] != "exports";
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let (wit_names, field_names, tys) = match &struct_item.fields {
            syn::Fields::Unit => (Vec::new(), Vec::new(), Vec::new()),
            syn::Fields::Named(fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect();
                let wit_names = field_names
                    .iter()
                    .map(|f| f.to_string().to_kebab_case())
                    .collect();
                let field_tys = fields.named.iter().map(|f| &f.ty).collect();
                (wit_names, field_names, field_tys)
            }
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        res.push(parse_quote! {
            impl #impl_generics ValueTyped for #struct_name #ty_generics #where_clause {
                #[allow(unused_imports)]
                fn value_type() -> Type {
                    use #use_path::*;
                    let fields = vec![
                        #((#wit_names, <#tys as ValueTyped>::value_type())),*
                    ];
                    Type::record(fields).unwrap()
                }
            }
        });
        if self.to_value && in_import {
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
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum) -> Vec<Item> {
        let mut res = Vec::new();
        let use_path: syn::Path = syn::parse_str(&module_path.join("::")).unwrap();
        let in_import = module_path[0] != "exports";
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let cases = enum_item.variants.iter().map(|variant| {
            let tag = &variant.ident;
            let wit_name = tag.to_string().to_kebab_case();
            match &variant.fields {
                syn::Fields::Unit => quote! { (#wit_name, None) },
                syn::Fields::Unnamed(f) => {
                    assert!(f.unnamed.len() == 1);
                    let ty = &f.unnamed.first().unwrap().ty;
                    quote! { (#wit_name, Some(<#ty as ValueTyped>::value_type())) }
                }
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        res.push(parse_quote! {
            impl #impl_generics ValueTyped for #enum_name #ty_generics #where_clause {
                #[allow(unused_imports)]
                fn value_type() -> Type {
                    use #use_path::*;
                    let cases = vec![#(#cases),*];
                    Type::variant(cases).unwrap()
                }
            }
        });
        if self.to_value && in_import {
            let arms = enum_item.variants.iter().map(|variant| {
                let tag = &variant.ident;
                let wit_name = tag.to_string().to_kebab_case();
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
    fn trait_defs(&self) -> Vec<Item> {
        vec![]
    }
}
