use crate::traits::Trait;
use crate::util::make_path;
use heck::{ToKebabCase, ToSnakeCase};
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct DialogTrait;

impl Trait for DialogTrait {
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
            impl Dialog for #resource_path {
                fn read_value(_dep: u32) -> Self {
                    proxy::conversion::conversion::#call(42)
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Dialog for &'a #resource_path {
                fn read_value(_dep: u32) -> Self {
                    SCOPED_ALLOC.with(|alloc| {
                        let mut alloc = alloc.borrow_mut();
                        alloc.alloc(proxy::conversion::conversion::#call(42))
                    })
                }
            }
            });
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl Dialog for #resource_path {
                fn read_value(_dep: u32) -> Self {
                    Ok(#resource_path::new(MockedResource {
                        handle: 42,
                        name: #wit_name.to_string(),
                    }))
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Dialog for #borrow_path {
                fn read_value(_dep: u32) -> Self {
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
        impl #impl_generics Dialog for #struct_name #ty_generics #where_clause {
            fn read_value(_dep: u32) -> Self {
                Self {
                    #(
                        #field_names: Dialog::read_value(0),
                    )*
                }
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
        impl #impl_generics Dialog for #enum_name #ty_generics #where_clause {
            fn read_value(dep: u32) -> Self {
                todo!()
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
        impl Dialog for #flag_path {
            fn read_value(_dep: u32) -> Self {
                #(
                    #flags |
                )* 0
            }
        }
        });
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let ast: syn::File = parse_quote! {
          trait Dialog {
            fn read_value(dep: u32) -> Self;
          }
          impl Dialog for String {
            fn read_value(dep: u32) -> Self {
                let wave = proxy::util::dialog::read_string(dep);
                let ret: Value = wasm_wave::from_str(&<Self as ValueTyped>::value_type(), &wave).unwrap();
                ret.to_rust()
            }
          }
          impl Dialog for bool {
            fn read_value(dep: u32) -> Self {
                let wave = proxy::util::dialog::read_bool(dep);
                let ret: Value = wasm_wave::from_str(&<Self as ValueTyped>::value_type(), &wave).unwrap();
                ret.to_rust()
            }
          }
        };
        ast.items
    }
}
