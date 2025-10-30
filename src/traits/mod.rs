use crate::codegen::{GenerateMode, State, TypeInfo};
use syn::{Item, ItemEnum, ItemStruct};

mod proxy;
mod wave;

pub struct TraitGenerator<'a> {
    state: &'a State<'a>,
    traits: Vec<Box<dyn Trait + 'a>>,
}

pub trait Trait {
    fn trait_defs(&self) -> Vec<Item>;
    fn resource_trait(&self, module_path: &[String], item: &ItemStruct) -> Vec<Item>;
    fn struct_trait(&self, module_path: &[String], item: &ItemStruct) -> Vec<Item>;
    fn enum_trait(&self, module_path: &[String], item: &ItemEnum) -> Vec<Item>;
}

impl<'a> TraitGenerator<'a> {
    pub fn new(state: &'a State<'a>) -> TraitGenerator<'a> {
        let mut traits: Vec<Box<dyn Trait + 'a>> = Vec::new();
        if matches!(state.mode, GenerateMode::Instrument) {
            traits.push(Box::new(wave::TypeTrait));
            traits.push(Box::new(proxy::ProxyTrait::new(state)));
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
                    }
                }
            }
            items.extend(t.trait_defs());
        }
        items
    }
}
