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
                }));
                traits.push(Box::new(proxy::ProxyTrait::new(state)));
            }
            Mode::Replay => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: true,
                    has_replay_table: true,
                    wrapped_labels: false,
                }));
            }
            Mode::Fuzz => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: false,
                    has_replay_table: true,
                    wrapped_labels: false,
                }));
                traits.push(Box::new(fuzz::FuzzTrait {}));
            }
            Mode::Dialog => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: true,
                    has_replay_table: true,
                    wrapped_labels: false,
                }));
                traits.push(Box::new(dialog::DialogTrait {}));
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
