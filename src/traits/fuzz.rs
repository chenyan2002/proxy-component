use crate::traits::Trait;
use crate::util::make_path;
use heck::{ToKebabCase, ToSnakeCase};
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct FuzzTrait;

impl Trait for FuzzTrait {
    fn resource_trait(&self, module_path: &[String], resource: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = resource.ident.to_string().to_kebab_case();
        let in_import = module_path[0] != "exports";
        if in_import {
            let call = format!(
                "get_mock_{}_magic42_{}",
                module_path.join("_"),
                resource.ident
            )
            .to_snake_case();
            let call: syn::Ident = syn::parse_str(&call).unwrap();
            res.push(parse_quote! {
            impl Arbitrary<'_> for #resource_path {
                fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
                    Ok(proxy::conversion::conversion::#call(42))
                }
                fn size_hint(_: usize) -> (usize, Option<usize>) {
                    (0, Some(0))
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Arbitrary<'_> for &'a #resource_path {
                fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
                    SCOPED_ALLOC.with(|alloc| {
                        let mut alloc = alloc.borrow_mut();
                        Ok(alloc.alloc(proxy::conversion::conversion::#call(42)))
                    })
                }
            }
            });
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl Arbitrary<'_> for #resource_path {
                fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
                    Ok(#resource_path::new(MockedResource {
                        handle: 42,
                        name: #wit_name.to_string(),
                    }))
                }
                fn size_hint(_: usize) -> (usize, Option<usize>) {
                    (0, Some(0))
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Arbitrary<'_> for #borrow_path {
                fn arbitrary(_u: &mut Unstructured<'_>) -> Result<Self> {
                    unreachable!()
                }
            }
            });
        }
        res
    }
    fn struct_trait(&self, module_path: &[String], struct_item: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
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
        impl #impl_generics Arbitrary<'_> for #struct_name #ty_generics #where_clause {
            fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
                Ok(#struct_name {
                    #(#field_names: u.arbitrary()?),*
                })
            }
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                let mut res = (0, Some(0));
                #(
                    let size = <#tys as Arbitrary>::size_hint(depth + 1);
                    res = arbitrary::size_hint::and(res, size);
                )*
                res
            }
        }
        });
        res
    }
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum) -> Vec<Item> {
        let mut res = Vec::new();
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let arms = enum_item.variants.iter().enumerate().map(|(idx, variant)| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! {
                    #idx => Ok(#enum_name::#tag)
                },
                syn::Fields::Unnamed(_) => quote! {
                    #idx => Ok(#enum_name::#tag(u.arbitrary()?))
                },
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        let size_hint = enum_item
            .variants
            .iter()
            .map(|variant| match &variant.fields {
                syn::Fields::Unit => quote! {},
                syn::Fields::Unnamed(f) => {
                    assert!(f.unnamed.len() == 1);
                    let ty = &f.unnamed.first().unwrap().ty;
                    quote! {
                        let size = <#ty as Arbitrary>::size_hint(depth + 1);
                        res = arbitrary::size_hint::or(res, size);
                    }
                }
                syn::Fields::Named(_) => unreachable!(),
            });
        let variant_len = enum_item.variants.len();
        res.push(parse_quote! {
        impl #impl_generics Arbitrary<'_> for #enum_name #ty_generics #where_clause {
            fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
                let idx = u.choose_index(#variant_len)?;
                match idx {
                    #(#arms),*,
                    _ => unreachable!(),
                }
            }
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                let mut res = (1, Some(1));
                #(
                    #size_hint
                )*
                res
            }
        }
        });
        res
    }
    fn flag_trait(&self, module_path: &[String], item: &crate::codegen::ItemFlag) -> Vec<Item> {
        let mut res = Vec::new();
        let flag_path = make_path(module_path, &item.name.to_string());
        let flag_num = item.flags.len() - 1;
        let flags = item.flags.iter().map(|f| {
            quote! { #flag_path::#f }
        });
        res.push(parse_quote! {
        impl Arbitrary<'_> for #flag_path {
            fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
                let flag_count = u.int_in_range(0..=#flag_num)?;
                let mut res = #flag_path::empty();
                let choices = [#(#flags),*];
                for _ in 0..flag_count {
                    let flag = u.choose(&choices)?;
                    res |= flag;
                }
                Ok(res)
            }
            fn size_hint(_: usize) -> (usize, Option<usize>) {
                (0, Some(#flag_num))
            }
        }
        });
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let mocked_resource = quote! {
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
        };
        let ast: syn::File = parse_quote! {
          #![allow(unused_variables)]
          #[allow(unused_imports)]
          use arbitrary::{Arbitrary, Unstructured, Result};
          use std::io::Write;
          #mocked_resource
        };
        ast.items
    }
}
