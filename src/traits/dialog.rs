use crate::traits::Trait;
use crate::util::{make_path, wit_func_name};
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
                    let handle = HANDLE_ID.with(|id| {
                        let mut id = id.borrow_mut();
                        let current_id = *id;
                        *id += 1;
                        current_id
                    });
                    proxy::conversion::conversion::#call(handle)
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Dialog for &'a #resource_path {
                fn read_value(_dep: u32) -> Self {
                    let handle = HANDLE_ID.with(|id| {
                        let mut id = id.borrow_mut();
                        let current_id = *id;
                        *id += 1;
                        current_id
                    });
                    SCOPED_ALLOC.with(|alloc| {
                        let mut alloc = alloc.borrow_mut();
                        alloc.alloc(proxy::conversion::conversion::#call(handle))
                    })
                }
            }
            });
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl Dialog for #resource_path {
                fn read_value(_dep: u32) -> Self {
                    let handle = HANDLE_ID.with(|id| {
                        let mut id = id.borrow_mut();
                        let current_id = *id;
                        *id += 1;
                        current_id
                    });
                    #resource_path::new(MockedResource {
                        handle,
                        name: #wit_name.to_string(),
                    })
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
        let struct_wit_name = wit_func_name(module_path, &None, &struct_item.ident, &None);
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let (field_names, tys, field_wit_names) = match &struct_item.fields {
            syn::Fields::Unit => (Vec::new(), Vec::new(), Vec::new()),
            syn::Fields::Named(fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect();
                let field_tys = fields.named.iter().map(|f| &f.ty).collect();
                let field_wit_names = fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap().to_string().to_kebab_case())
                    .collect();
                (field_names, field_tys, field_wit_names)
            }
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        res.push(parse_quote! {
        impl #impl_generics Dialog for #struct_name #ty_generics #where_clause {
            fn read_value(dep: u32) -> Self {
                proxy::util::dialog::print(dep, &format!("provide value for struct {}", #struct_wit_name));
                #(
                    proxy::util::dialog::print(dep + 1, &format!("provide value for field {}: {}", #field_wit_names, <#tys as WitName>::name()));
                    let #field_names = Dialog::read_value(dep + 1);
                )*
                Self {
                    #(
                        #field_names,
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
            tag.to_string().to_kebab_case()
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
        let enum_wit_name = wit_func_name(module_path, &None, &enum_item.ident, &None);
        res.push(parse_quote! {
        impl #impl_generics Dialog for #enum_name #ty_generics #where_clause {
            fn read_value(dep: u32) -> Self {
                let idx = proxy::util::dialog::read_select(dep, &format!("Select a variant for {}", #enum_wit_name), &#tags) as usize;
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
        let flag_wit_name = wit_func_name(module_path, &None, &item.name, &None);
        let flags = &item.flags;
        let flag_names: Vec<_> = flags
            .iter()
            .map(|flag| flag.to_string().to_kebab_case())
            .collect();
        let flag_names = quote! { [#( #flag_names.to_string() ),*] };
        let flags = flags.iter().map(|flag| quote! { #flag_path::#flag });
        let idxs = 0..flags.len();
        res.push(parse_quote! {
        impl Dialog for #flag_path {
            fn read_value(dep: u32) -> Self {
                let selections = proxy::util::dialog::read_multi_select(dep, &format!("Select flags for {}", #flag_wit_name), &#flag_names);
                let mut res = #flag_path::empty();
                for idx in selections {
                    match idx as usize {
                        #(
                            #idxs => res |= #flags,
                        )*
                        _ => unreachable!(),
                    }
                }
                res
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
          thread_local! {
            static HANDLE_ID: std::cell::RefCell<u32> = std::cell::RefCell::new(1);
          }
          impl Dialog for () {
              fn read_value(_dep: u32) -> Self {
                  ()
              }
          }
          impl<T: Dialog> Dialog for Option<T> {
              fn read_value(dep: u32) -> Self {
                  let selection = proxy::util::dialog::read_select(dep, "Select an option tag", &["none".to_string(), "some".to_string()]);
                  if selection == 0 {
                      None
                  } else {
                      Some(Dialog::read_value(dep + 1))
                  }
              }
          }
          impl<T: Dialog + 'static> Dialog for Vec<T> {
              fn read_value(dep: u32) -> Self {
                use std::any::TypeId;
                if TypeId::of::<T>() == TypeId::of::<u8>() {
                    let hex = proxy::util::dialog::read_raw_string(dep, "Enter a string as list<u8>");
                    let bytes = hex.into_bytes();
                    unsafe {
                        std::mem::transmute::<Vec<u8>, Vec<T>>(bytes)
                    }
                } else {
                  let len = proxy::util::dialog::read_num(dep, "Enter the length of the list");
                  (0..len).map(|_| Dialog::read_value(dep + 1)).collect()
                }
              }
          }
          impl <O: Dialog, E: Dialog> Dialog for Result<O, E> {
              fn read_value(dep: u32) -> Self {
                  let selection = proxy::util::dialog::read_select(dep, "Select result", &["ok".to_string(), "err".to_string()]);
                  if selection == 0 {
                      Ok(Dialog::read_value(dep + 1))
                  } else {
                      Err(Dialog::read_value(dep + 1))
                  }
              }
          }
          impl Dialog for MockedResource {
              fn read_value(_dep: u32) -> Self {
                  Self {
                      handle: 42,
                      name: "mocked-resource".to_string(),
                  }
              }
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
          macro_rules! impl_dialog_tuple {
              ($($T:ident),+) => {
                  impl<$($T: Dialog),+> Dialog for ($($T,)+) {
                      fn read_value(dep: u32) -> Self {
                          ($($T::read_value(dep + 1),)+)
                      }
                  }
              };
          }
          impl_dialog_tuple!(T1);
          impl_dialog_tuple!(T1, T2);
          impl_dialog_tuple!(T1, T2, T3);
          impl_dialog_tuple!(T1, T2, T3, T4);
          impl_dialog_tuple!(T1, T2, T3, T4, T5);
          impl_dialog_tuple!(T1, T2, T3, T4, T5, T6);
          impl_dialog_tuple!(T1, T2, T3, T4, T5, T6, T7);
          impl_dialog_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
        };
        ast.items
    }
}
