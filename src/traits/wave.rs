use crate::codegen::make_path;
use crate::traits::Trait;
use heck::ToKebabCase;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct TypeTrait;

impl Trait for TypeTrait {
    fn resource_trait(&self, module_path: &[String], resource: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = resource.ident.to_string().to_kebab_case();
        res.push(parse_quote! {
            impl ValueTyped for #resource_path {
                fn value_type() -> Type {
                    Type::resource(#wit_name, false)
                }
            }
        });
        res
    }
    fn struct_trait(&self, module_path: &[String], struct_item: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let use_path: syn::Path = syn::parse_str(&module_path.join("::")).unwrap();
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let fields = match &struct_item.fields {
            syn::Fields::Unit => quote! {},
            syn::Fields::Named(fields) => {
                let wit_names = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().to_string().to_kebab_case());
                let field_tys = fields.named.iter().map(|f| &f.ty);
                quote! {
                    #(
                        (#wit_names, <#field_tys as ValueTyped>::value_type())
                    ),*
                }
            }
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        res.push(parse_quote! {
            impl #impl_generics ValueTyped for #struct_name #ty_generics #where_clause {
                #[allow(unused_imports)]
                fn value_type() -> Type {
                    use #use_path::*;
                    let fields = vec![#fields];
                    Type::record(fields).unwrap()
                }
            }
        });
        res
    }
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum) -> Vec<Item> {
        let mut res = Vec::new();
        let use_path: syn::Path = syn::parse_str(&module_path.join("::")).unwrap();
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
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        vec![]
    }
}
