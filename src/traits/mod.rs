use crate::codegen::{GenerateMode, ItemFlag, State, TypeInfo};
use syn::{Item, ItemEnum, ItemStruct};

mod fuzz;
mod proxy;
mod wave;

pub struct TraitGenerator<'a> {
    state: &'a State,
    traits: Vec<Box<dyn Trait + 'a>>,
}

pub trait Trait {
    fn trait_defs(&self) -> Vec<Item>;
    fn resource_trait(&self, module_path: &[String], item: &ItemStruct) -> Vec<Item>;
    fn struct_trait(&self, module_path: &[String], item: &ItemStruct) -> Vec<Item>;
    fn enum_trait(&self, module_path: &[String], item: &ItemEnum) -> Vec<Item>;
    fn flag_trait(&self, module_path: &[String], item: &ItemFlag) -> Vec<Item>;
}

impl<'a> TraitGenerator<'a> {
    pub fn new(state: &'a State) -> TraitGenerator<'a> {
        let mut traits: Vec<Box<dyn Trait + 'a>> = Vec::new();
        match &state.mode {
            GenerateMode::Stubs => (),
            GenerateMode::Instrument => traits.push(Box::new(proxy::ProxyTrait::new(state))),
            GenerateMode::Record => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: false,
                    has_replay_table: false,
                }));
                traits.push(Box::new(proxy::ProxyTrait::new(state)));
            }
            GenerateMode::Replay => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: true,
                    has_replay_table: true,
                }));
            }
            GenerateMode::Fuzz => {
                traits.push(Box::new(wave::WaveTrait {
                    to_value: true,
                    to_rust: false,
                    has_replay_table: true,
                }));
                traits.push(Box::new(fuzz::FuzzTrait {}));
            }
        }
        TraitGenerator { state, traits }
    }

    pub fn generate(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for t in &self.traits {
            for (module_path, info) in &self.state.types {
                for ty in info {
                    match ty {
                        TypeInfo::Resource(item) => {
                            items.extend(t.resource_trait(module_path, item))
                        }
                        TypeInfo::Struct(item) => items.extend(t.struct_trait(module_path, item)),
                        TypeInfo::Enum(item) => items.extend(t.enum_trait(module_path, item)),
                        TypeInfo::Flag(item) => items.extend(t.flag_trait(module_path, item)),
                    }
                }
            }
            items.extend(t.trait_defs());
        }
        items
    }
}
