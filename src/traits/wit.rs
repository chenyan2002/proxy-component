use crate::traits::Trait;
use crate::util::{make_path, wit_func_name};
use heck::ToKebabCase;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct WitTrait;

impl Trait for WitTrait {
    fn resource_trait(&self, module_path: &[String], resource: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = resource.ident.to_string().to_kebab_case();
        res.push(parse_quote! {
            impl WitName for #resource_path {
                fn name() -> String {
                    #wit_name.to_string()
                }
            }
        });
        let in_import = module_path[0] != "exports";
        if in_import {
            res.push(parse_quote! {
                impl WitName for &#resource_path {
                    fn name() -> String {
                        format!("borrow<{}>", #wit_name)
                    }
                }
            });
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
                impl<'a> WitName for #borrow_path {
                    fn name() -> String {
                        format!("borrow<{}>", #wit_name)
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
        res.push(parse_quote! {
            impl #impl_generics WitName for #struct_name #ty_generics #where_clause {
                fn name() -> String {
                    #struct_wit_name.to_string()
                }
            }
        });
        res
    }
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum) -> Vec<Item> {
        let mut res = Vec::new();
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let enum_wit_name = wit_func_name(module_path, &None, &enum_item.ident, &None);
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        res.push(parse_quote! {
            impl #impl_generics WitName for #enum_name #ty_generics #where_clause {
                fn name() -> String {
                    #enum_wit_name.to_string()
                }
            }
        });
        res
    }
    fn flag_trait(&self, module_path: &[String], item: &crate::codegen::ItemFlag) -> Vec<Item> {
        let mut res = Vec::new();
        let flag_path = make_path(module_path, &item.name.to_string());
        let flag_wit_name = wit_func_name(module_path, &None, &item.name, &None);
        res.push(parse_quote! {
            impl WitName for #flag_path {
                fn name() -> String {
                    #flag_wit_name.to_string()
                }
            }
        });
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let ast: syn::File = parse_quote! {
          trait WitName {
            fn name() -> String;
          }
          impl<T: WitName> WitName for Option<T> {
            fn name() -> String {
                format!("option<{}>", T::name())
            }
          }
          impl<T: WitName> WitName for Vec<T> {
              fn name() -> String {
                  format!("list<{}>", T::name())
              }
          }
          impl <O: WitName, E: WitName> WitName for Result<O, E> {
              fn name() -> String {
                  format!("result<{}, {}>", O::name(), E::name())
              }
          }
          impl WitName for () {
              fn name() -> String {
                  "_".to_string()
              }
          }
          impl WitName for MockedResource {
              fn name() -> String {
                  "mocked-resource".to_string()
              }
          }
          macro_rules! impl_wit_primitive {
              ($($ty:ty),*) => {
                  $(
                      impl WitName for $ty {
                          fn name() -> String {
                              <Self as ValueTyped>::value_type().to_string()
                          }
                      }
                  )*
              };
          }
          impl_wit_primitive! {
              bool,
              u8,
              u16,
              u32,
              u64,
              i8,
              i16,
              i32,
              i64,
              f32,
              f64,
              char,
              String
          }
          macro_rules! impl_wit_tuple {
              ($($T:ident),+) => {
                  impl<$($T: WitName),+> WitName for ($($T,)+) {
                      fn name() -> String {
                          format!("tuple<{}>", vec![$($T::name()),+].join(", "))
                      }
                  }
              };
          }
          impl_wit_tuple!(T1);
          impl_wit_tuple!(T1, T2);
          impl_wit_tuple!(T1, T2, T3);
          impl_wit_tuple!(T1, T2, T3, T4);
          impl_wit_tuple!(T1, T2, T3, T4, T5);
          impl_wit_tuple!(T1, T2, T3, T4, T5, T6);
          impl_wit_tuple!(T1, T2, T3, T4, T5, T6, T7);
          impl_wit_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
        };
        ast.items
    }
}
