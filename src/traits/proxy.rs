use crate::codegen::State;
use crate::traits::Trait;
use crate::util::{is_wrapped_module, make_path};
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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
                let call = if is_wrapped_module(&output_path[0]) {
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
                    unsafe { Self::Output::lift(self as *const _ as *const u8) }
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
                        // only possible after wit-bindgen 0.48
                        self.get::<T>()
                    }
                }
            });
        }
        res
    }
    fn struct_trait(&self, module_path: &[String], struct_item: &ItemStruct) -> Vec<Item> {
        let mut res = Vec::new();
        let name = struct_item.ident.to_string();
        let struct_name = make_path(module_path, &name);
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let output_path = self.get_proxy_path(module_path);
        if !self.state.has_type_def(&output_path, &name) {
            let identity_cast = identity_cast();
            return vec![parse_quote! {
                impl #impl_generics ToProxy for #struct_name #ty_generics #where_clause {
                    type Output = #struct_name #ty_generics;
                    fn to_proxy(self) -> Self::Output {
                        self
                    }
                    #identity_cast
                }
            }];
        }
        let output_path = make_path(&output_path, &name);
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
        let name = enum_item.ident.to_string();
        let enum_name = make_path(module_path, &name);
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let output_path = self.get_proxy_path(module_path);
        if !self.state.has_type_def(&output_path, &name) {
            let identity_cast = identity_cast();
            return vec![parse_quote! {
                impl #impl_generics ToProxy for #enum_name #ty_generics #where_clause {
                    type Output = #enum_name #ty_generics;
                    fn to_proxy(self) -> Self::Output {
                        self
                    }
                    #identity_cast
                }
            }];
        }
        let output_path = make_path(&output_path, &name);
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
        let name = item.name.to_string();
        let flag_name = make_path(module_path, &name);
        let output_path = self.get_proxy_path(module_path);
        if !self.state.has_type_def(&output_path, &name) {
            let identity_cast = identity_cast();
            return vec![parse_quote! {
                impl ToProxy for #flag_name {
                    type Output = #flag_name;
                    fn to_proxy(self) -> Self::Output {
                        self
                    }
                    #identity_cast
                }
            }];
        }
        let output_path = make_path(&output_path, &name);
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
        let identity_cast = identity_cast();
        let defs: File = parse_quote! {
        trait ToProxy: Sized {
          type Output;
          fn to_proxy(self) -> Self::Output;
          // True iff `Output == Self`, for this type and all types nested in it.
          const IS_ID: bool = false;
          // Converts `Self` to `Output` inside any type constructor `F` (e.g. `Vec<Option<_>>`)
          // without touching the value. Returns `Ok` iff `IS_ID`; identity impls return `Ok(x)`, which
          // type-checks only because `Output == Self` there.
          fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
              Err(x)
          }
        }
        // A type constructor, i.e. a function from types to types. Rust can only express this as
        // a trait with a generic associated type: `F::Apply<X>` is `F` applied to `X`.
        //
        // How to read the impls below: each struct is one constructor, and its `Apply` says what
        // it produces. `InVec<F>` means "a `Vec` around `X`, then `F` around that", so
        // constructors nest inside-out: the outermost struct is the innermost wrapper.
        //   Id::Apply<X>                  = X
        //   InVec<Id>::Apply<X>           = Vec<X>
        //   InOption<InVec<Id>>::Apply<X> = Vec<Option<X>>
        //   InResultErr<Id, u8>::Apply<X> = Result<u8, X>
        // Types with several slots (`Result`, tuples) need one constructor per slot, since each
        // slot is a different function of `X`.
        //
        // The same impls read two other ways:
        // - Fill: `F` is a type with one hole, and `Apply<X>` fills the hole with `X`.
        //   `InVec<Id>` is `Vec<_>`, `InOption<InVec<Id>>` is `Vec<Option<_>>`, and
        //   `InResultErr<Id, u8>` is `Result<u8, _>`.
        // - Family pattern: `F` names a family of types indexed by `X`, and `Apply<X>` is the
        //   member at index `X`. `InVec<Id>` is the family {`Vec<u8>`, `Vec<String>`, ...}, and
        //   `InVec<Id>::Apply<u8>` picks out `Vec<u8>`. See
        //   https://smallcultfollowing.com/babysteps/blog/2016/11/03/associated-type-constructors-part-2-family-traits/
        //
        // `ToProxy::cast` uses this as Leibniz equality: if `A == B`, then `F::Apply<A>` and
        // `F::Apply<B>` are the same type for every `F`. An identity impl (`Output == Self`) can
        // therefore return its `F::Apply<Self>` argument as `F::Apply<Self::Output>` unchanged, no
        // matter how deeply `Self` is nested in containers. `cast` is the Rust analog of Haskell's
        // `subst :: forall c. c a -> c b` in
        // https://hackage.haskell.org/package/eq/docs/Data-Eq-Type.html
        trait TypeCtor {
            type Apply<X>;
        }
        #[allow(dead_code)]
        struct Id;
        impl TypeCtor for Id {
            type Apply<X> = X;
        }
        #[allow(dead_code)]
        struct InVec<F>(core::marker::PhantomData<F>);
        impl<F: TypeCtor> TypeCtor for InVec<F> {
            type Apply<X> = F::Apply<Vec<X>>;
        }
        #[allow(dead_code)]
        struct InOption<F>(core::marker::PhantomData<F>);
        impl<F: TypeCtor> TypeCtor for InOption<F> {
            type Apply<X> = F::Apply<Option<X>>;
        }
        #[allow(dead_code)]
        struct InResultOk<F, E>(core::marker::PhantomData<(F, E)>);
        impl<F: TypeCtor, E> TypeCtor for InResultOk<F, E> {
            type Apply<X> = F::Apply<Result<X, E>>;
        }
        #[allow(dead_code)]
        struct InResultErr<F, T>(core::marker::PhantomData<(F, T)>);
        impl<F: TypeCtor, T> TypeCtor for InResultErr<F, T> {
            type Apply<X> = F::Apply<Result<T, X>>;
        }
        impl crate::ToProxy for String {
            type Output = String;
            fn to_proxy(self) -> Self::Output {
                self
            }
            #identity_cast
        }
        impl<T: crate::ToProxy> crate::ToProxy for Vec::<T> {
            type Output = Vec::<T::Output>;
            fn to_proxy(self) -> Self::Output {
                match T::cast::<InVec<Id>>(self) {
                    Ok(v) => v,
                    Err(v) => v.into_iter().map(|x| x.to_proxy()).collect(),
                }
            }
            const IS_ID: bool = T::IS_ID;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                T::cast::<InVec<F>>(x)
            }
        }
        impl<T, E> ToProxy for Result<T, E>
        where T: ToProxy, E: ToProxy {
            type Output = Result<T::Output, E::Output>;
            fn to_proxy(self) -> Self::Output {
                match self {
                    Ok(ok) => Ok(ok.to_proxy()),
                    Err(err) => Err(err.to_proxy()),
                }
            }
            const IS_ID: bool = T::IS_ID && E::IS_ID;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                // Check first, so that we never fail after casting only one side.
                if !Self::IS_ID {
                    return Err(x);
                }
                let Ok(x) = T::cast::<InResultOk<F, E>>(x) else { unreachable!() };
                let Ok(x) = E::cast::<InResultErr<F, T::Output>>(x) else { unreachable!() };
                Ok(x)
            }
        }
        impl<Inner> ToProxy for Option<Inner>
        where Inner: ToProxy {
            type Output = Option<Inner::Output>;
            fn to_proxy(self) -> Self::Output {
                self.map(|x| x.to_proxy())
            }
            const IS_ID: bool = Inner::IS_ID;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                Inner::cast::<InOption<F>>(x)
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
                        #identity_cast
                    }
                )*
            };
        }
        impl_to_import_export_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64, (), bool, char);
        };
        let mut items = defs.items;
        for n in 1..=9 {
            items.extend(tuple_trait(n));
        }
        items
    }
}
// Overrides `ToProxy::cast` for impls where `Output == Self`.
fn identity_cast() -> TokenStream {
    quote! {
        const IS_ID: bool = true;
        fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
            Ok(x)
        }
    }
}
// `ToProxy` for an n-tuple, plus one `TypeCtor` per position for `cast`.
fn tuple_trait(n: usize) -> Vec<Item> {
    let mut res = Vec::new();
    let tys: Vec<_> = (0..n).map(|i| format_ident!("T{i}")).collect();
    let idx = (0..n).map(syn::Index::from);
    let mut casts = Vec::new();
    for i in 0..n {
        let ctx = format_ident!("InTuple{n}_{i}");
        let params: Vec<_> = (0..n)
            .filter(|j| *j != i)
            .map(|j| format_ident!("A{j}"))
            .collect();
        let slots = (0..n).map(|j| {
            if j == i {
                quote! { X }
            } else {
                let a = format_ident!("A{j}");
                quote! { #a }
            }
        });
        res.push(parse_quote! {
            #[allow(dead_code, non_camel_case_types)]
            struct #ctx<F, #(#params),*>(core::marker::PhantomData<(F, #(#params),*)>);
        });
        res.push(parse_quote! {
            impl<F: TypeCtor, #(#params),*> TypeCtor for #ctx<F, #(#params),*> {
                type Apply<X> = F::Apply<(#(#slots,)*)>;
            }
        });
        // Positions before `i` have already been cast to their `Output`.
        let args = (0..n).filter(|j| *j != i).map(|j| {
            let t = &tys[j];
            if j < i {
                quote! { #t::Output }
            } else {
                quote! { #t }
            }
        });
        let ti = &tys[i];
        casts.push(quote! {
            let Ok(x) = #ti::cast::<#ctx<F, #(#args),*>>(x) else { unreachable!() };
        });
    }
    res.push(parse_quote! {
        impl<#(#tys: ToProxy),*> ToProxy for (#(#tys,)*) {
            type Output = (#(#tys::Output,)*);
            fn to_proxy(self) -> Self::Output {
                (#(self.#idx.to_proxy(),)*)
            }
            const IS_ID: bool = #(#tys::IS_ID)&&*;
            fn cast<F: TypeCtor>(x: F::Apply<Self>) -> Result<F::Apply<Self::Output>, F::Apply<Self>> {
                // Check first, so that we never fail after casting only some positions.
                if !Self::IS_ID {
                    return Err(x);
                }
                #(#casts)*
                Ok(x)
            }
        }
    });
    res
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
