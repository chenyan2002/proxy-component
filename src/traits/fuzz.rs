use crate::codegen::util::make_path;
use crate::codegen::wit::WitType;
use crate::traits::{Trait, new_mock};
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct FuzzTrait {
    /// Whether the bindings have streams, whose `Fuzz` impl needs their `wit_stream` module
    pub has_streams: bool,
    /// Whether the bindings have futures, whose `Fuzz` impl needs their `wit_future` module
    pub has_futures: bool,
}

impl Trait for FuzzTrait {
    fn resource_trait(
        &self,
        module_path: &[String],
        resource: &ItemStruct,
        wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = &wit.name;
        let in_import = module_path[0] != "exports";
        if in_import {
            // The imports component, which makes the mock, labels it by its handle.
            let mock = new_mock(wit, quote! { 0 });
            res.push(parse_quote! {
            impl Fuzz for #resource_path {
                fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                    Ok(#mock)
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Fuzz for &'a #resource_path {
                fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                    SCOPED_ALLOC.with(|alloc| {
                        let mut alloc = alloc.borrow_mut();
                        Ok(alloc.alloc(#mock))
                    })
                }
            }
            });
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl Fuzz for #resource_path {
                fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                    // `_resource_new` gives it its handle.
                    Ok(#resource_path::new(MockedResource {
                        handle: 0,
                        name: #wit_name.to_string(),
                    }))
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Fuzz for #borrow_path {
                fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                    unreachable!()
                }
            }
            });
        }
        res
    }
    fn struct_trait(
        &self,
        module_path: &[String],
        struct_item: &ItemStruct,
        _wit: &WitType,
    ) -> Vec<Item> {
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let field_names: Vec<_> = match &struct_item.fields {
            syn::Fields::Unit => Vec::new(),
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.clone().unwrap())
                .collect(),
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        vec![parse_quote! {
        impl #impl_generics Fuzz for #struct_name #ty_generics #where_clause {
            fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                Ok(#struct_name {
                    #(#field_names: Fuzz::fuzz(u)?),*
                })
            }
        }
        }]
    }
    fn enum_trait(
        &self,
        module_path: &[String],
        enum_item: &ItemEnum,
        _wit: &WitType,
    ) -> Vec<Item> {
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let arms = enum_item.variants.iter().enumerate().map(|(idx, variant)| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! {
                    #idx => Ok(#enum_name::#tag)
                },
                syn::Fields::Unnamed(_) => quote! {
                    #idx => Ok(#enum_name::#tag(Fuzz::fuzz(u)?))
                },
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        let variant_len = enum_item.variants.len();
        vec![parse_quote! {
        impl #impl_generics Fuzz for #enum_name #ty_generics #where_clause {
            fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                let idx = u.choose_index(#variant_len)?;
                match idx {
                    #(#arms),*,
                    _ => unreachable!(),
                }
            }
        }
        }]
    }
    fn flag_trait(
        &self,
        module_path: &[String],
        item: &crate::codegen::ItemFlag,
        _wit: &WitType,
    ) -> Vec<Item> {
        let flag_path = make_path(module_path, &item.name.to_string());
        let flag_num = item.flags.len() - 1;
        let flags = item.flags.iter().map(|f| {
            quote! { #flag_path::#f }
        });
        vec![parse_quote! {
        impl Fuzz for #flag_path {
            fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                let flag_count = u.int_in_range(0..=#flag_num)?;
                let mut res = #flag_path::empty();
                let choices = [#(#flags),*];
                for _ in 0..flag_count {
                    let flag = u.choose(&choices)?;
                    res |= *flag;
                }
                Ok(res)
            }
        }
        }]
    }
    fn trait_defs(&self) -> Vec<Item> {
        let ast: syn::File = parse_quote! {
          #[allow(unused_imports)]
          use arbitrary::{Arbitrary, Unstructured, Result};
          // A local trait rather than `Arbitrary`, so that it can be implemented for wit-bindgen's
          // streams and futures.
          trait Fuzz: Sized {
              // A value the virtualized host gives the guest.
              fn fuzz(u: &mut Unstructured<'_>) -> Result<Self>;
              // What the virtualized host does with a value the guest gives it: by default, drop it.
              // Only streams and futures override it, which p2 worlds don't have.
              #[allow(dead_code)]
              fn consume(self, _u: &mut Unstructured<'_>) {}
          }
          impl Fuzz for MockedResource {
              fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                  // A constructor's, which `_resource_new` gives its handle.
                  Ok(Self {
                      handle: 0,
                      name: "mocked-resource".to_string(),
                  })
              }
          }
          macro_rules! impl_fuzz_primitive {
              ($($t:ty),*) => {
                  $(
                      impl Fuzz for $t {
                          fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                              u.arbitrary()
                          }
                      }
                  )*
              };
          }
          impl_fuzz_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool, char, String, ());
          impl<T: Fuzz> Fuzz for Option<T> {
              fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                  Ok(if u.arbitrary()? { Some(Fuzz::fuzz(u)?) } else { None })
              }
              fn consume(self, u: &mut Unstructured<'_>) {
                  if let Some(x) = self {
                      x.consume(u);
                  }
              }
          }
          impl<T: Fuzz> Fuzz for Vec<T> {
              // Like `Arbitrary` for `Vec`, which decides to continue before each element.
              fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                  let mut res = Vec::new();
                  while u.arbitrary()? {
                      res.push(Fuzz::fuzz(u)?);
                  }
                  Ok(res)
              }
              fn consume(self, u: &mut Unstructured<'_>) {
                  for x in self {
                      x.consume(u);
                  }
              }
          }
          impl<T: Fuzz, E: Fuzz> Fuzz for core::result::Result<T, E> {
              fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                  Ok(if u.arbitrary()? { Ok(Fuzz::fuzz(u)?) } else { Err(Fuzz::fuzz(u)?) })
              }
              fn consume(self, u: &mut Unstructured<'_>) {
                  match self {
                      Ok(x) => x.consume(u),
                      Err(x) => x.consume(u),
                  }
              }
          }
          macro_rules! impl_fuzz_tuple {
              ($($T:ident),+) => {
                  impl<$($T: Fuzz),+> Fuzz for ($($T,)+) {
                      fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                          Ok(($($T::fuzz(u)?,)+))
                      }
                      #[allow(non_snake_case)]
                      fn consume(self, u: &mut Unstructured<'_>) {
                          let ($($T,)+) = self;
                          $($T.consume(u);)+
                      }
                  }
              };
          }
          impl_fuzz_tuple!(T1);
          impl_fuzz_tuple!(T1, T2);
          impl_fuzz_tuple!(T1, T2, T3);
          impl_fuzz_tuple!(T1, T2, T3, T4);
          impl_fuzz_tuple!(T1, T2, T3, T4, T5);
          impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6);
          impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6, T7);
          impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
          impl_fuzz_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        };
        let mut items = ast.items;
        items.extend(self.async_defs());
        items
    }
}

impl FuzzTrait {
    /// `Fuzz` for streams, futures and error-contexts. The bindings only have the `wit_stream`
    /// and `wit_future` modules for them if the world has streams or futures. Their labels, e.g.
    /// `#stream-1`, are their handles, as for resources.
    ///
    /// The virtualized host writes the streams and futures it gives the guest, and reads those
    /// the guest gives it, in jobs of `assets/executor.rs`. What it writes, and how it reads,
    /// e.g. whether it reads to the end, is fuzzed when the value changes hands.
    fn async_defs(&self) -> Vec<Item> {
        let mut items = Vec::new();
        if self.has_streams {
            let ast: syn::File = parse_quote! {
              impl<T> Fuzz for wit_bindgen::StreamReader<T>
              where
                  T: Fuzz + ToValue + ValueTyped + wit_stream::StreamPayload,
              {
                  // Writes fuzzed chunks, then ends the stream.
                  fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                      let (mut writer, reader) = wit_stream::new::<T>();
                      let label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
                      let chunks: Vec<Vec<T>> = Fuzz::fuzz(u)?;
                      spawn(async move {
                          for chunk in chunks.into_iter().filter(|chunk| !chunk.is_empty()) {
                              let items = wasm_wave::to_string(&chunk.to_value()).unwrap();
                              if !writer.write_all(chunk).await.is_empty() {
                                  proxy::util::debug::print(&format!("{label} dropped by guest"));
                                  return;
                              }
                              proxy::util::debug::print(&format!("{label}.read() = {items}"));
                          }
                          proxy::util::debug::print(&format!("{label}.read() = dropped"));
                      });
                      Ok(reader)
                  }
                  // Mostly reads to the end; otherwise reads a few chunks, or none, and drops the
                  // stream. Out of fuzz data, it reads to the end.
                  fn consume(mut self, u: &mut Unstructured<'_>) {
                      let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
                      let reads: Option<u32> = match u.int_in_range(0..=9u8).unwrap_or(0) {
                          8 => Some(u.int_in_range(1..=4).unwrap_or(1)),
                          9 => Some(0),
                          _ => None,
                      };
                      let capacity = u.int_in_range(1..=4096usize).unwrap_or(4096);
                      spawn(async move {
                          let mut count = 0;
                          while reads != Some(count) {
                              let (result, items) = self.read(Vec::with_capacity(capacity)).await;
                              if let wit_bindgen::StreamResult::Dropped = result {
                                  proxy::util::debug::print(&format!("{label} -> dropped"));
                                  return;
                              }
                              let items = wasm_wave::to_string(&items.to_value()).unwrap();
                              proxy::util::debug::print(&format!("{label} -> {items}"));
                              count += 1;
                          }
                          proxy::util::debug::print(&format!("{label} dropped by host"));
                      });
                  }
              }
            };
            items.extend(ast.items);
        }
        if self.has_futures {
            let ast: syn::File = parse_quote! {
              impl<T> Fuzz for wit_bindgen::FutureReader<T>
              where
                  T: Fuzz + ToValue + ValueTyped + wit_future::FuturePayload,
              {
                  // Writes a fuzzed value.
                  fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                      let value = T::fuzz(u)?;
                      let (writer, reader) =
                          wit_future::new::<T>(|| unreachable!("the value is always written"));
                      let label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
                      let wave = wasm_wave::to_string(&value.to_value()).unwrap();
                      spawn(async move {
                          if writer.write(value).await.is_ok() {
                              proxy::util::debug::print(&format!("{label}.read() = {wave}"));
                          } else {
                              proxy::util::debug::print(&format!("{label} dropped by guest"));
                          }
                      });
                      Ok(reader)
                  }
                  // Mostly reads the value; otherwise drops the future.
                  fn consume(self, u: &mut Unstructured<'_>) {
                      let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
                      let read = u.int_in_range(0..=9u8).unwrap_or(0) != 9;
                      spawn(async move {
                          if read {
                              let value = wasm_wave::to_string(&self.await.to_value()).unwrap();
                              proxy::util::debug::print(&format!("{label} -> {value}"));
                          } else {
                              drop(self);
                              proxy::util::debug::print(&format!("{label} dropped by host"));
                          }
                      });
                  }
              }
            };
            items.extend(ast.items);
        }
        items.push(parse_quote! {
          impl Fuzz for wit_bindgen::rt::async_support::ErrorContext {
              fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                  Ok(Self::new(&String::fuzz(u)?))
              }
          }
        });
        items
    }
}
