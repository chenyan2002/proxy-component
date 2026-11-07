use crate::codegen::util::make_path;
use crate::codegen::wit::WitType;
use crate::traits::{Trait, host_end_defs, new_mock};
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
            // A resource of the guest, which the host gets from an export, and drops; the host
            // keeps its own resources, which the guest gives back, as no one sees their drop.
            let consume = if wit.conversion.is_none() {
                quote! {
                    fn consume(self, _u: &mut Unstructured<'_>) {
                        let name = wasm_wave::to_string(&ToValue::to_value(&self)).unwrap();
                        // `start` drops it, which takes a step, as an export call does.
                        let line = trace_drop(Party::Host, TRACE_TABLE, name);
                        let at = proxy::conversion::executor::step();
                        proxy::recorder::record::record(&Line { at: Some(at), ..line });
                    }
                }
            } else {
                quote! {}
            };
            res.push(parse_quote! {
            impl Fuzz for #resource_path {
                fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                    Ok(#mock)
                }
                #consume
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
                    Ok(#resource_path::new(MockedResource {
                        handle: next_mock_label(),
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
            #[allow(unused_variables)]
            fn consume(self, u: &mut Unstructured<'_>) {
                #(Fuzz::consume(self.#field_names, u);)*
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
        let consume_arms = enum_item.variants.iter().map(|variant| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! { #enum_name::#tag => {} },
                syn::Fields::Unnamed(_) => quote! { #enum_name::#tag(x) => Fuzz::consume(x, u) },
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
            #[allow(unused_variables)]
            fn consume(self, u: &mut Unstructured<'_>) {
                match self {
                    #(#consume_arms),*
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
              // What the virtualized host does with a value the guest gives it: by default, drop
              // it. Streams and futures override it, and the guest's resources, whose drop the
              // host records; records, variants and the like pass it on to their parts.
              #[allow(dead_code)]
              fn consume(self, _u: &mut Unstructured<'_>) {}
          }
          thread_local! {
              // The run's seed, which `fuzz_data` hashes with its text, so that runs differ.
              static FUZZ_SEED: u64 = {
                  let random = proxy::util::debug::get_random();
                  u64::from_le_bytes(random[..8].try_into().unwrap())
              };
          }
          /// Fuzz data drawn from `text`, e.g. a call's arguments: the same for the same text in
          /// a run, so that the host answers alike, and different in another run. The text's
          /// hash seeds splitmix64, which makes as much data as `get-random` gives.
          #[allow(dead_code)]
          fn fuzz_data(text: &str) -> Vec<u8> {
              use std::hash::{Hash, Hasher};
              let mut hasher = std::hash::DefaultHasher::new();
              FUZZ_SEED.with(|seed| seed.hash(&mut hasher));
              text.hash(&mut hasher);
              let mut state = hasher.finish();
              (0..128)
                  .flat_map(|_| {
                      state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
                      let mut z = state;
                      z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                      z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                      (z ^ (z >> 31)).to_le_bytes()
                  })
                  .collect()
          }
          impl Fuzz for MockedResource {
              fn fuzz(_u: &mut Unstructured<'_>) -> Result<Self> {
                  // A constructor's.
                  Ok(Self {
                      handle: next_mock_label(),
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
    /// `#stream-1`, are the handles of the ends the component keeps: the reader of one the guest
    /// gives, and the writer of one the host gives, which `label_by_writer` also prints its
    /// reader by.
    ///
    /// The virtualized host writes the streams and futures it gives the guest, and reads those
    /// the guest gives it, in jobs of `assets/executor.rs`, which record each copy, value and
    /// drop. What it writes, and how it reads, e.g. whether it reads to the end, is fuzzed when
    /// the value changes hands.
    fn async_defs(&self) -> Vec<Item> {
        let mut items = host_end_defs(self.has_streams, self.has_futures);
        if self.has_streams || self.has_futures {
            let ast: syn::File = parse_quote! {
              /// The label of a stream or future, by the end this component keeps.
              fn label_of(end: &impl ToValue) -> Label {
                  Label { table: TRACE_TABLE, name: handle_label(end) }
              }
              /// A host line, in step `at`, in which its job made it, e.g. issued the write the
              /// guest read; or, by default, in the current one.
              fn host_line_at(at: u32, event: Event) -> Line {
                  Line { by: Party::Host, at: Some(at), event }
              }
              fn host_line(event: Event) -> Line {
                  host_line_at(trace_at(), event)
              }
              fn guest_line(event: Event) -> Line {
                  Line { by: Party::Guest, at: None, event }
              }
            };
            items.extend(ast.items);
        }
        if self.has_streams {
            let ast: syn::File = parse_quote! {
              impl<T> Fuzz for wit_bindgen::StreamReader<T>
              where
                  T: Fuzz + ToValue + ValueTyped + wit_stream::StreamPayload,
              {
                  // Writes fuzzed chunks, then ends the stream. Each write waits until the guest
                  // reads, with a zero-length write, so that it completes in this run, and its
                  // line, the items the guest took, is recorded before the guest goes on, e.g.
                  // traps; a chunk the guest takes in parts is a line per part.
                  fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                      let (mut writer, reader, name) = host_stream::<T>(true);
                      let label = Label { table: TRACE_TABLE, name };
                      let chunks: Vec<Vec<T>> = Fuzz::fuzz(u)?;
                      spawn(async move {
                          let mut chunks = chunks.into_iter().filter(|chunk| !chunk.is_empty());
                          while let Some(chunk) = chunks.next() {
                              let waves: Vec<_> = chunk
                                  .iter()
                                  .map(|item| wasm_wave::to_string(&item.to_value()).unwrap())
                                  .collect();
                              let (mut items, mut offset) = (chunk, 0);
                              while !items.is_empty() {
                                  let (ready, _) = writer.write(Vec::new()).await;
                                  if let wit_bindgen::StreamResult::Dropped = ready {
                                      proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                                      host_dropping(move || drop((items, chunks.collect::<Vec<_>>())));
                                      return;
                                  }
                                  let (at, count) = (trace_at(), items.len());
                                  let (result, rest) = writer.write(items).await;
                                  let rest = rest.into_vec();
                                  let written = count - rest.len();
                                  if written > 0 {
                                      let items = format!("[{}]", waves[offset..offset + written].join(", "));
                                      let chunk = Chunk { stream: label.clone(), items };
                                      proxy::recorder::record::record(&host_line_at(at, Event::Write(chunk)));
                                      offset += written;
                                  }
                                  if let wit_bindgen::StreamResult::Dropped = result {
                                      proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                                      host_dropping(move || drop((rest, chunks.collect::<Vec<_>>())));
                                      return;
                                  }
                                  items = rest;
                              }
                          }
                          drop(writer);
                          proxy::recorder::record::record(&host_line(Event::Drop(label)));
                      });
                      Ok(reader)
                  }
                  // Mostly reads to the end; otherwise reads a few chunks, or none, and drops the
                  // stream. Out of fuzz data, it reads to the end.
                  fn consume(mut self, u: &mut Unstructured<'_>) {
                      let label = label_of(&self);
                      let reads: Option<u32> = match u.int_in_range(0..=9u8).unwrap_or(0) {
                          8 => Some(u.int_in_range(1..=4).unwrap_or(1)),
                          9 => Some(0),
                          _ => None,
                      };
                      let capacity = u.int_in_range(1..=4096usize).unwrap_or(4096);
                      spawn(async move {
                          let mut count = 0;
                          while reads != Some(count) {
                              // Once the guest writes, with a zero-length read, so that the read
                              // completes in this run, and its line is recorded before the guest
                              // goes on.
                              let (ready, _) = self.read(Vec::new()).await;
                              if let wit_bindgen::StreamResult::Dropped = ready {
                                  proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                                  return;
                              }
                              let at = trace_at();
                              let (result, items) = self.read(Vec::with_capacity(capacity)).await;
                              // The guest may write its last items as it drops the stream.
                              if !items.is_empty() {
                                  let wave = wasm_wave::to_string(&items.to_value()).unwrap();
                                  let chunk = Chunk { stream: label.clone(), items: wave };
                                  proxy::recorder::record::record(&host_line_at(at, Event::Read(chunk)));
                                  host_dropping(move || drop(items));
                              }
                              if let wit_bindgen::StreamResult::Dropped = result {
                                  proxy::recorder::record::record(&guest_line(Event::Drop(label)));
                                  return;
                              }
                              count += 1;
                          }
                          drop(self);
                          proxy::recorder::record::record(&host_line(Event::Drop(label)));
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
                  // Writes a fuzzed value, whose line is recorded as it is written, before the guest
                  // can take it and go on, e.g. trap. The guest may drop its end instead, which
                  // isn't recorded: the line ends the label, and replay writes the value alike.
                  fn fuzz(u: &mut Unstructured<'_>) -> Result<Self> {
                      let value = T::fuzz(u)?;
                      let (writer, reader, name) = host_future::<T>(true);
                      let label = Label { table: TRACE_TABLE, name };
                      let wave = wasm_wave::to_string(&value.to_value()).unwrap();
                      spawn(async move {
                          let line = FutureValue { future: label.clone(), value: wave };
                          proxy::recorder::record::record(&host_line(Event::Value(line)));
                          if let Err(unwritten) = writer.write(value).await {
                              host_dropping(move || drop(unwritten));
                          }
                      });
                      Ok(reader)
                  }
                  // Mostly reads the value, which the guest wrote; otherwise drops the future.
                  fn consume(self, u: &mut Unstructured<'_>) {
                      let label = label_of(&self);
                      let read = u.int_in_range(0..=9u8).unwrap_or(0) != 9;
                      spawn(async move {
                          if read {
                              let value = self.await;
                              let wave = wasm_wave::to_string(&value.to_value()).unwrap();
                              let line = guest_line(Event::Value(FutureValue { future: label, value: wave }));
                              proxy::recorder::record::record(&line);
                              host_dropping(move || drop(value));
                          } else {
                              drop(self);
                              proxy::recorder::record::record(&host_line(Event::Drop(label)));
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
              // The host drops one the guest gives it, which ends its label, as the trace says.
              fn consume(self, _u: &mut Unstructured<'_>) {
                  let name = wasm_wave::to_string(&self.to_value()).unwrap();
                  proxy::recorder::record::record(&trace_drop(Party::Host, TRACE_TABLE, name));
              }
          }
        });
        items
    }
}
