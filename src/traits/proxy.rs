use crate::codegen::{State, make_path};
use crate::traits::Trait;
use heck::ToSnakeCase;
use quote::quote;
use syn::{File, Item, ItemEnum, ItemStruct, parse_quote};

pub struct ProxyTrait<'a> {
    state: &'a State<'a>,
}
impl<'a> ProxyTrait<'a> {
    pub fn new(state: &'a State<'a>) -> Self {
        ProxyTrait { state }
    }
}

impl Trait for ProxyTrait<'_> {
    fn resource_trait(&self, module_path: &[String], resource: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let (output_path, trait_name, func) = self.get_proxy_path(module_path);
        let output_owned = make_path(&output_path, &resource.ident.to_string());
        let in_import = module_path[0] != "exports";
        if in_import {
            let is_import_only = output_path[0] != "exports";
            if is_import_only {
                let call = if output_path[0].starts_with("wrapped_") {
                    "get_wrapped_"
                } else {
                    "get_host_"
                }
                .to_string()
                    + &resource.ident.to_string().to_snake_case();
                let call: syn::Path = syn::parse_str(&call).unwrap();
                let output_path = make_path(&output_path, &resource.ident.to_string());
                res.push(parse_quote! {
                    impl #trait_name for #resource_path {
                        type Output = #output_path;
                        fn #func(self) -> Self::Output {
                            proxy::conversion::conversion::#call(self)
                        }
                    }
                });
            } else {
                let export_borrow =
                    make_path(&output_path, &format!("{}Borrow<'a>", &resource.ident));
                res.push(parse_quote! {
                impl ToExport for #resource_path {
                  type Output = #output_owned;
                  fn to_export(self) -> Self::Output {
                    Self::Output::new(self)
                  }
                }});
                res.push(parse_quote! {
                impl<'a> ToExport for &'a #resource_path {
                  type Output = #export_borrow;
                  fn to_export(self) -> Self::Output {
                    unsafe { Self::Output::lift(self as *const _ as usize) }
                  }
                }});
            }
        } else {
            let export_borrow = make_path(module_path, &format!("{}Borrow<'a>", &resource.ident));
            res.push(parse_quote! {
                impl ToImport for #resource_path {
                    type Output = #output_owned;
                    fn to_import(self) -> Self::Output {
                        self.into_inner()
                    }
                }
            });
            res.push(parse_quote! {
                impl<'a> ToImport for #export_borrow {
                    type Output = &'a #output_owned;
                    fn to_import(self) -> Self::Output {
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
        let (output_path, trait_name, func) = self.get_proxy_path(module_path);
        let output_path = make_path(&output_path, &struct_item.ident.to_string());
        let fields = match &struct_item.fields {
            syn::Fields::Unit => quote! { Self::Output },
            syn::Fields::Named(fields) => {
                let field_names = fields.named.iter().map(|f| &f.ident);
                quote! { Self::Output { #(#field_names: self.#field_names.#func()),* } }
            }
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        res.push(parse_quote! {
            impl #impl_generics #trait_name for #struct_name #ty_generics #where_clause {
                type Output = #output_path;
                fn #func(self) -> Self::Output {
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
        let (output_path, trait_name, func) = self.get_proxy_path(module_path);
        let output_path = make_path(&output_path, &enum_item.ident.to_string());
        let match_arms = enum_item.variants.iter().map(|variant| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! { Self::#tag => Self::Output::#tag },
                syn::Fields::Unnamed(_) => {
                    quote! { Self::#tag(e) => Self::Output::#tag(e.#func()) }
                }
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        res.push(parse_quote! {
            impl #impl_generics #trait_name for #enum_name #ty_generics #where_clause {
                type Output = #output_path;
                fn #func(self) -> Self::Output {
                    match self {
                        #(#match_arms),*
                    }
                }
            }
        });
        res
    }
    fn trait_defs(&self) -> Vec<Item> {
        let defs: File = parse_quote! {
        trait ToExport {
          type Output;
          fn to_export(self) -> Self::Output;
        }
        trait ToImport {
          type Output;
          fn to_import(self) -> Self::Output;
        }
        impl crate::ToExport for String {
            type Output = String;
            fn to_export(self) -> Self::Output {
                self
            }
        }
        impl crate::ToImport for String {
          type Output = String;
          fn to_import(self) -> Self::Output {
            self
          }
        }
        impl<T: crate::ToExport> crate::ToExport for Vec::<T> {
            type Output = Vec::<T::Output>;
            fn to_export(self) -> Self::Output {
                self.into_iter().map(|x| x.to_export()).collect()
            }
        }
        impl<T: crate::ToImport> crate::ToImport for Vec::<T> {
            type Output = Vec::<T::Output>;
            fn to_import(self) -> Self::Output {
                self.into_iter().map(|x| x.to_import()).collect()
            }
        }
        impl<Ok, Err> ToExport for Result<Ok, Err>
        where Ok: ToExport, Err: ToExport {
            type Output = Result<Ok::Output, Err::Output>;
            fn to_export(self) -> Self::Output {
                match self {
                    Ok(ok) => Ok(ok.to_export()),
                    Err(err) => Err(err.to_export()),
                }
            }
        }
        impl<Ok, Err> ToImport for Result<Ok, Err>
        where Ok: ToImport, Err: ToImport {
            type Output = Result<Ok::Output, Err::Output>;
            fn to_import(self) -> Self::Output {
                match self {
                    Ok(ok) => Ok(ok.to_import()),
                    Err(err) => Err(err.to_import()),
                }
            }
        }
        impl<Inner> ToExport for Option<Inner>
        where Inner: ToExport {
            type Output = Option<Inner::Output>;
            fn to_export(self) -> Self::Output {
                self.map(|x| x.to_export())
            }
        }
        impl<Inner> ToImport for Option<Inner>
        where Inner: ToImport {
            type Output = Option<Inner::Output>;
            fn to_import(self) -> Self::Output {
                self.map(|x| x.to_import())
            }
        }
        macro_rules! impl_to_import_export_for_primitive {
            ($($t:ty),*) => {
                $(
                    impl ToImport for $t {
                        type Output = $t;
                        fn to_import(self) -> Self::Output {
                            self
                        }
                    }
                    impl ToExport for $t {
                        type Output = $t;
                        fn to_export(self) -> Self::Output {
                            self
                        }
                    }
                )*
            };
        }
        impl_to_import_export_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64, (), bool, char);

        macro_rules! impl_to_import_export_for_tuple {
            ( $($T:ident, $i:tt),* ) => {
                impl<$($T: ToExport),*> ToExport for ($($T,)*) {
                    type Output = ($($T::Output,)*);
                    fn to_export(self) -> Self::Output {
                        ($(self.$i.to_export(),)*)
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
        };
        defs.items
    }
}
impl ProxyTrait<'_> {
    fn get_proxy_path(&self, src_path: &[String]) -> (Vec<String>, syn::Path, syn::Path) {
        let from_export = src_path[0] == "exports";
        let mut res = crate::codegen::get_proxy_path(src_path);
        let (trait_name, func) = if from_export {
            assert!(self.state.module_paths.contains(&res));
            ("ToImport", "to_import")
        } else if !self.state.module_paths.contains(&res) {
            res.remove(0);
            assert!(self.state.module_paths.contains(&res));
            if res[0].starts_with("wrapped_") {
                ("ToImport", "to_import")
            } else {
                ("ToExport", "to_export")
            }
        } else {
            ("ToExport", "to_export")
        };
        (
            res,
            syn::parse_str(trait_name).unwrap(),
            syn::parse_str(func).unwrap(),
        )
    }
}
