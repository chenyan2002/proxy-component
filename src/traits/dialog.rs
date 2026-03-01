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
            fn read_value(dep: u32) -> Self {
                Self {
                    #(
                        #field_names: Dialog::read_value(dep + 1),
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
        let tags = enum_item.variants.iter().map(|variant| {
            let tag = &variant.ident;
            quote! { stringify!(#tag) }
        });
        let tags = quote! { [#( #tags.to_string() ),*] };
        let arms = enum_item.variants.iter().enumerate().map(|(idx, variant)| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! {
                    #idx => #enum_name::#tag
                },
                syn::Fields::Unnamed(_) => quote! {
                    #idx => #enum_name::#tag(Dialog::read_value(dep + 1))
                },
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        res.push(parse_quote! {
        impl #impl_generics Dialog for #enum_name #ty_generics #where_clause {
            fn read_value(dep: u32) -> Self {
                let idx = proxy::util::dialog::read_selection(dep, &format!("Select a variant for {}", stringify!(#enum_name)), &#tags) as usize;
                match idx {
                    #(
                        #arms,
                    )*
                    _ => unreachable!(),
                }
            }
        }
        });
        res
    }
    fn flag_trait(&self, module_path: &[String], item: &crate::codegen::ItemFlag) -> Vec<Item> {
        let mut res = Vec::new();
        let flag_path = make_path(module_path, &item.name.to_string());
        let _flag_num = item.flags.len() - 1;
        let flags: Vec<_> = item
            .flags
            .iter()
            .map(|f| {
                quote! { #flag_path::#f }
            })
            .collect();
        let flags_expr = if flags.is_empty() {
            quote! { #flag_path::empty() }
        } else {
            quote! { #( #flags )|* }
        };
        res.push(parse_quote! {
        impl Dialog for #flag_path {
            fn read_value(_dep: u32) -> Self {
                #flags_expr
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
          macro_rules! impl_dialog_primitive {
              ($($ty:ty => $read_fn:ident),* $(,)?) => {
                  $(
                      impl Dialog for $ty {
                          fn read_value(dep: u32) -> Self {
                              let wave = proxy::util::dialog::$read_fn(dep);
                              let ret: Value = wasm_wave::from_str(&<Self as ValueTyped>::value_type(), &wave).unwrap();
                              ret.to_rust()
                          }
                      }
                  )*
              };
          }
          impl_dialog_primitive! {
              bool => read_bool,
              u8 => read_u8,
              u16 => read_u16,
              u32 => read_u32,
              u64 => read_u64,
              i8 => read_s8,
              i16 => read_s16,
              i32 => read_s32,
              i64 => read_s64,
              f32 => read_f32,
              f64 => read_f64,
              char => read_char,
              String => read_string,
          }
        };
        ast.items
    }
}
