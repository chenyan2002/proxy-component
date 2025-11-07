use crate::traits::Trait;
use crate::{codegen::State, util::make_path};
use heck::ToSnakeCase;
use quote::quote;
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
    fn resource_trait(&self, module_path: &[String], resource: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let output_path = self.get_proxy_path(module_path);
        let output_owned = make_path(&output_path, &resource.ident.to_string());
        let in_import = module_path[0] != "exports";
        if in_import {
            let is_import_only = output_path[0] != "exports";
            if is_import_only {
                let call = if output_path[0].starts_with("wrapped_") {
                    "get_"
                } else {
                    "get_host_"
                }
                .to_string()
                    + &format!("{}_{}", output_path.join("_"), resource.ident).to_snake_case();
                let call: syn::Path = syn::parse_str(&call).unwrap();
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
                    unsafe { Self::Output::lift(self as *const _ as usize) }
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
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
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
        let output_path = self.get_proxy_path(module_path);
        let output_path = make_path(&output_path, &struct_item.ident.to_string());
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
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum) -> Vec<Item> {
        let mut res = Vec::new();
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let output_path = self.get_proxy_path(module_path);
        let output_path = make_path(&output_path, &enum_item.ident.to_string());
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
    fn flag_trait(&self, module_path: &[String], item: &crate::codegen::ItemFlag) -> Vec<Item> {
        let mut res = Vec::new();
        let flag_name = make_path(module_path, &item.name.to_string());
        let output_path = self.get_proxy_path(module_path);
        let output_path = make_path(&output_path, &item.name.to_string());
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
        let defs: File = parse_quote! {
        trait ToProxy {
          type Output;
          fn to_proxy(self) -> Self::Output;
        }
        impl crate::ToProxy for String {
            type Output = String;
            fn to_proxy(self) -> Self::Output {
                self
            }
        }
        impl<T: crate::ToProxy> crate::ToProxy for Vec::<T> {
            type Output = Vec::<T::Output>;
            fn to_proxy(self) -> Self::Output {
                self.into_iter().map(|x| x.to_proxy()).collect()
            }
        }
        impl<Ok, Err> ToProxy for Result<Ok, Err>
        where Ok: ToProxy, Err: ToProxy {
            type Output = Result<Ok::Output, Err::Output>;
            fn to_proxy(self) -> Self::Output {
                match self {
                    Ok(ok) => Ok(ok.to_proxy()),
                    Err(err) => Err(err.to_proxy()),
                }
            }
        }
        impl<Inner> ToProxy for Option<Inner>
        where Inner: ToProxy {
            type Output = Option<Inner::Output>;
            fn to_proxy(self) -> Self::Output {
                self.map(|x| x.to_proxy())
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
                    }
                )*
            };
        }
        impl_to_import_export_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64, (), bool, char);

        macro_rules! impl_to_import_export_for_tuple {
            ( $($T:ident, $i:tt),* ) => {
                impl<$($T: ToProxy),*> ToProxy for ($($T,)*) {
                    type Output = ($($T::Output,)*);
                    fn to_proxy(self) -> Self::Output {
                        ($(self.$i.to_proxy(),)*)
                    }
                }
            };
        }
        impl_to_import_export_for_tuple!(T0, 0);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2, T3, 3);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2, T3, 3, T4, 4);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2, T3, 3, T4, 4, T5, 5);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2, T3, 3, T4, 4, T5, 5, T6, 6);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2, T3, 3, T4, 4, T5, 5, T6, 6, T7, 7);
        impl_to_import_export_for_tuple!(T0, 0, T1, 1, T2, 2, T3, 3, T4, 4, T5, 5, T6, 6, T7, 7, T8, 8);
        };
        defs.items
    }
}
impl ProxyTrait<'_> {
    fn get_proxy_path(&self, src_path: &[String]) -> Vec<String> {
        let from_export = src_path[0] == "exports";
        let mut res = crate::codegen::get_proxy_path(src_path);
        if from_export {
            assert!(self.state.module_paths.contains(&res));
        } else if !self.state.module_paths.contains(&res) {
            res.remove(0);
            assert!(self.state.module_paths.contains(&res));
        } else {
            assert!(self.state.module_paths.contains(&res));
        };
        res
    }
}
