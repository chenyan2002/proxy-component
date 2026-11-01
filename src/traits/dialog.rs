use crate::codegen::util::make_path;
use crate::codegen::wit::WitType;
use crate::traits::{Trait, new_mock};
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct DialogTrait {
    /// Whether the bindings have streams, whose `Dialog` impl needs their `wit_stream` module
    pub has_streams: bool,
    /// Whether the bindings have futures, whose `Dialog` impl needs their `wit_future` module
    pub has_futures: bool,
}

impl Trait for DialogTrait {
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
            impl Dialog for #resource_path {
                fn read_value(_dep: u32) -> Self {
                    #mock
                }
            }
            });
            res.push(parse_quote! {
            impl<'a> Dialog for &'a #resource_path {
                fn read_value(_dep: u32) -> Self {
                    SCOPED_ALLOC.with(|alloc| {
                        let mut alloc = alloc.borrow_mut();
                        alloc.alloc(#mock)
                    })
                }
            }
            });
        } else {
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            res.push(parse_quote! {
            impl Dialog for #resource_path {
                fn read_value(_dep: u32) -> Self {
                    // `_resource_new` gives it its handle.
                    #resource_path::new(MockedResource {
                        handle: 0,
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
    fn struct_trait(
        &self,
        module_path: &[String],
        struct_item: &ItemStruct,
        wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let struct_wit_name = &wit.qualified_name;
        let field_wit_names = &wit.members;
        let field_wit_tys = &wit.field_types;
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
        res.push(parse_quote! {
        impl #impl_generics Dialog for #struct_name #ty_generics #where_clause {
            fn read_value(dep: u32) -> Self {
                proxy::util::dialog::print(dep, &format!("provide value for struct {}", #struct_wit_name));
                #(
                    proxy::util::dialog::print(dep + 1, &format!("provide value for field {}: {}", #field_wit_names, #field_wit_tys));
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
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum, wit: &WitType) -> Vec<Item> {
        let mut res = Vec::new();
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let tags = &wit.members;
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
        let enum_wit_name = &wit.qualified_name;
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
    fn flag_trait(
        &self,
        module_path: &[String],
        item: &crate::codegen::ItemFlag,
        wit: &WitType,
    ) -> Vec<Item> {
        let mut res = Vec::new();
        let flag_path = make_path(module_path, &item.name.to_string());
        let flag_wit_name = &wit.qualified_name;
        let flags = &item.flags;
        let flag_names = &wit.members;
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
          trait Dialog: Sized {
            // A value the virtualized host gives the guest, which the user enters.
            fn read_value(dep: u32) -> Self;
            // What the virtualized host does with a value the guest gives it: by default, drop it.
            // Only streams and futures override it, which p2 worlds don't have.
            #[allow(dead_code)]
            fn consume(self, _dep: u32) {}
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
              fn consume(self, dep: u32) {
                  if let Some(x) = self {
                      x.consume(dep);
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
              fn consume(self, dep: u32) {
                  for x in self {
                      x.consume(dep);
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
              fn consume(self, dep: u32) {
                  match self {
                      Ok(x) => x.consume(dep),
                      Err(x) => x.consume(dep),
                  }
              }
          }
          impl Dialog for MockedResource {
              fn read_value(_dep: u32) -> Self {
                  // A constructor's, which `_resource_new` gives its handle.
                  Self {
                      handle: 0,
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
                      #[allow(non_snake_case)]
                      fn consume(self, dep: u32) {
                          let ($($T,)+) = self;
                          $($T.consume(dep);)+
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
        let mut items = ast.items;
        items.extend(self.async_defs());
        items
    }
}

impl DialogTrait {
    /// `Dialog` for streams, futures and error-contexts. The bindings only have the `wit_stream`
    /// and `wit_future` modules for them if the world has streams or futures.
    ///
    /// The virtualized host writes the streams and futures it gives the guest, and reads those
    /// the guest gives it, in jobs of `assets/executor.rs`, which ask the user, one chunk at a
    /// time, what to write, or whether to keep reading.
    fn async_defs(&self) -> Vec<Item> {
        let mut items = Vec::new();
        if self.has_streams {
            let ast: syn::File = parse_quote! {
              impl<T> Dialog for wit_bindgen::StreamReader<T>
              where
                  T: Dialog + ToValue + ValueTyped + wit_stream::StreamPayload + 'static,
              {
                  fn read_value(_dep: u32) -> Self {
                      let (mut writer, reader) = wit_stream::new::<T>();
                      // The host keeps the writer, so it labels the stream for its lifetime; the
                      // reader moves on to the guest, which frees its handle.
                      let label = wasm_wave::to_string(&ToValue::to_value(&writer)).unwrap();
                      let reader_label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
                      proxy::util::dialog::print(0, &format!("new (writer: {label}, reader: {reader_label})"));
                      spawn(async move {
                          let choices = ["write a chunk".to_string(), "end the stream".to_string()];
                          loop {
                              let prompt = format!("The host writes {label}");
                              if proxy::util::dialog::read_select(0, &prompt, &choices) == 1 {
                                  proxy::util::dialog::print(0, &format!("{label}.read() = dropped"));
                                  return;
                              }
                              let chunk: Vec<T> = Dialog::read_value(1);
                              let items = wasm_wave::to_string(&chunk.to_value()).unwrap();
                              if !writer.write_all(chunk).await.is_empty() {
                                  proxy::util::dialog::print(0, &format!("{label} dropped by guest"));
                                  return;
                              }
                              proxy::util::dialog::print(0, &format!("{label}.read() = {items}"));
                          }
                      });
                      reader
                  }
                  fn consume(mut self, _dep: u32) {
                      let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
                      spawn(async move {
                          let choices = ["read a chunk".to_string(), "drop the stream".to_string()];
                          loop {
                              let prompt = format!("The host reads {label}");
                              if proxy::util::dialog::read_select(0, &prompt, &choices) == 1 {
                                  proxy::util::dialog::print(0, &format!("{label} dropped by host"));
                                  return;
                              }
                              let (result, items) = self.read(Vec::with_capacity(4096)).await;
                              if let wit_bindgen::StreamResult::Dropped = result {
                                  proxy::util::dialog::print(0, &format!("{label} -> dropped"));
                                  return;
                              }
                              let items = wasm_wave::to_string(&items.to_value()).unwrap();
                              proxy::util::dialog::print(0, &format!("{label} -> {items}"));
                          }
                      });
                  }
              }
            };
            items.extend(ast.items);
        }
        if self.has_futures {
            let ast: syn::File = parse_quote! {
              impl<T> Dialog for wit_bindgen::FutureReader<T>
              where
                  T: Dialog + ToValue + ValueTyped + wit_future::FuturePayload,
              {
                  fn read_value(_dep: u32) -> Self {
                      let (writer, reader) =
                          wit_future::new::<T>(|| unreachable!("the value is always written"));
                      // As for streams, the host keeps the writer, which labels the future.
                      let label = wasm_wave::to_string(&ToValue::to_value(&writer)).unwrap();
                      let reader_label = wasm_wave::to_string(&ToValue::to_value(&reader)).unwrap();
                      proxy::util::dialog::print(0, &format!("new (writer: {label}, reader: {reader_label})"));
                      spawn(async move {
                          proxy::util::dialog::print(0, &format!("The host writes {label}"));
                          let value: T = Dialog::read_value(1);
                          let wave = wasm_wave::to_string(&value.to_value()).unwrap();
                          if writer.write(value).await.is_ok() {
                              proxy::util::dialog::print(0, &format!("{label}.read() = {wave}"));
                          } else {
                              proxy::util::dialog::print(0, &format!("{label} dropped by guest"));
                          }
                      });
                      reader
                  }
                  fn consume(self, _dep: u32) {
                      let label = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
                      spawn(async move {
                          let choices = ["read the value".to_string(), "drop the future".to_string()];
                          let prompt = format!("The host reads {label}");
                          if proxy::util::dialog::read_select(0, &prompt, &choices) == 1 {
                              drop(self);
                              proxy::util::dialog::print(0, &format!("{label} dropped by host"));
                              return;
                          }
                          let value = wasm_wave::to_string(&self.await.to_value()).unwrap();
                          proxy::util::dialog::print(0, &format!("{label} -> {value}"));
                      });
                  }
              }
            };
            items.extend(ast.items);
        }
        items.push(parse_quote! {
          impl Dialog for wit_bindgen::rt::async_support::ErrorContext {
              fn read_value(dep: u32) -> Self {
                  Self::new(&proxy::util::dialog::read_raw_string(dep, "Enter the message of an error-context"))
              }
          }
        });
        items
    }
}
