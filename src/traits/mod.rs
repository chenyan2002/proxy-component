use crate::Mode;
use crate::codegen::wit::WitType;
use crate::codegen::{ItemFlag, State, TypeInfo};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct};

mod dialog;
mod fuzz;
mod proxy;
mod wave;

pub struct TraitGenerator<'a> {
    state: &'a State,
    traits: Vec<Box<dyn Trait + 'a>>,
}

/// Code making a mock of a host resource with `handle`, through its `proxy:conversion`
/// function. The main component's own resources have none: only its constructors create them.
fn new_mock(wit: &WitType, handle: TokenStream) -> TokenStream {
    match &wit.conversion {
        Some(call) => quote! { proxy::conversion::conversion::#call(#handle) },
        // TODO: It's exported resource from the main component. We cannot mock it, instead
        // we need an instance table to track these resources.
        None => {
            let message = format!(
                "cannot mock `{}`, a resource of the main component",
                wit.name
            );
            // A function rather than `unimplemented!()` itself, which would make the code
            // around it unreachable.
            quote! {{
                fn cannot_mock<T>() -> T {
                    unimplemented!(#message)
                }
                let _ = #handle;
                cannot_mock()
            }}
        }
    }
}

/// Items for the virtualized host's ends of the streams and futures it gives the guest, which
/// the fuzz and dialog traits make: `host_stream` and `host_future` make the pair, keep its
/// writer, which labels it for its lifetime, and hand out its reader. The bindings only have the
/// `wit_stream` and `wit_future` modules if the world has streams or futures.
fn host_end_defs(has_streams: bool, has_futures: bool) -> Vec<Item> {
    let mut items = Vec::new();
    if !has_streams && !has_futures {
        return items;
    }
    let file: syn::File = syn::parse_quote! {
        thread_local! {
            // The readers of the streams and futures this component made, by handle, and the
            // writers that label them (see `label_by_writer`).
            static READER_LABELS: std::cell::RefCell<std::collections::HashMap<u32, u32>> =
                std::cell::RefCell::new(std::collections::HashMap::new());
        }
        /// Labels a stream or future this component made by the handle of its `writer`, which it
        /// keeps for the stream's lifetime, also where its `reader` is printed: in the value that
        /// hands the reader out, after which the reader moves on, and its handle can be reused.
        /// So the entry is gone once the reader is printed.
        fn label_by_writer(reader: u32, writer: u32) {
            static HOOK: std::sync::Once = std::sync::Once::new();
            HOOK.call_once(|| {
                wasm_wave::value::convert::set_handle_label(|_kind, handle| {
                    READER_LABELS.with_borrow_mut(|labels| labels.remove(&handle))
                })
            });
            READER_LABELS.with_borrow_mut(|labels| {
                // A handle the new writer reuses is no longer a reader's.
                labels.remove(&writer);
                labels.insert(reader, writer);
            });
        }
        /// The label of a stream or future, e.g. `#stream-7`, by the end this component keeps.
        fn handle_label(end: &impl ToValue) -> String {
            wasm_wave::to_string(&end.to_value()).unwrap()
        }
    };
    items.extend(file.items);
    if has_streams {
        let file: syn::File = syn::parse_quote! {
            /// A stream the host writes: its writer, its reader, to hand out, and its label.
            fn host_stream<T: wit_stream::StreamPayload + 'static>(
            ) -> (wit_bindgen::StreamWriter<T>, wit_bindgen::StreamReader<T>, String) {
                let (writer, reader) = wit_stream::new::<T>();
                label_by_writer(reader.handle(), writer.handle());
                let label = handle_label(&writer);
                (writer, reader, label)
            }
        };
        items.extend(file.items);
    }
    if has_futures {
        let file: syn::File = syn::parse_quote! {
            /// A future the host writes, which it always does: its writer, its reader, to hand
            /// out, and its label.
            fn host_future<T: wit_future::FuturePayload + 'static>(
            ) -> (wit_bindgen::FutureWriter<T>, wit_bindgen::FutureReader<T>, String) {
                let (writer, reader) =
                    wit_future::new::<T>(|| unreachable!("the value is always written"));
                label_by_writer(reader.handle(), writer.handle());
                let label = handle_label(&writer);
                (writer, reader, label)
            }
        };
        items.extend(file.items);
    }
    items
}

pub trait Trait {
    fn trait_defs(&self) -> Vec<Item>;
    // `wit` has the WIT names of the item and of its fields, cases or flags, in order.
    fn resource_trait(&self, module_path: &[String], item: &ItemStruct, wit: &WitType)
    -> Vec<Item>;
    fn struct_trait(&self, module_path: &[String], item: &ItemStruct, wit: &WitType) -> Vec<Item>;
    fn enum_trait(&self, module_path: &[String], item: &ItemEnum, wit: &WitType) -> Vec<Item>;
    fn flag_trait(&self, module_path: &[String], item: &ItemFlag, wit: &WitType) -> Vec<Item>;
}

impl<'a> TraitGenerator<'a> {
    pub fn new(state: &'a State) -> TraitGenerator<'a> {
        let mut traits: Vec<Box<dyn Trait + 'a>> = Vec::new();
        match &state.mode {
            Mode::Record => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: false,
                    has_replay_table: false,
                    wrapped_labels: true,
                    mock_labels: false,
                }));
                traits.push(Box::new(proxy::ProxyTrait::new(state)));
            }
            Mode::Replay => {
                // A mock carries the trace's label, which the exports component prints it by too.
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: true,
                    has_replay_table: true,
                    wrapped_labels: false,
                    mock_labels: true,
                }));
            }
            Mode::Fuzz => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: false,
                    has_replay_table: true,
                    wrapped_labels: false,
                    mock_labels: true,
                }));
                traits.push(Box::new(fuzz::FuzzTrait {
                    has_streams: state.has_streams,
                    has_futures: state.has_futures,
                }));
            }
            Mode::Dialog => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: true,
                    has_replay_table: true,
                    wrapped_labels: false,
                    mock_labels: true,
                }));
                traits.push(Box::new(dialog::DialogTrait {
                    has_streams: state.has_streams,
                    has_futures: state.has_futures,
                }));
            }
        }
        TraitGenerator { state, traits }
    }

    pub fn generate(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for t in &self.traits {
            for (module_path, info) in &self.state.types {
                for (id, ty) in info {
                    let wit = self.state.wit_type(module_path, *id);
                    match ty {
                        TypeInfo::Resource(item) => {
                            items.extend(t.resource_trait(module_path, item, &wit))
                        }
                        TypeInfo::Struct(item) => {
                            items.extend(t.struct_trait(module_path, item, &wit))
                        }
                        TypeInfo::Enum(item) => items.extend(t.enum_trait(module_path, item, &wit)),
                        TypeInfo::Flag(item) => items.extend(t.flag_trait(module_path, item, &wit)),
                    }
                }
            }
            items.extend(t.trait_defs());
        }
        items
    }
}
