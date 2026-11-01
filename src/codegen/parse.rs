use super::util::{FullTypePath, get_resource_from_trait_name};
use super::{ItemFlag, State, TypeInfo};
use heck::ToShoutySnakeCase;
use proc_macro2::TokenTree;
use quote::{ToTokens, format_ident};
use syn::{Ident, Item, Signature, Type, Visibility, visit_mut::VisitMut};
use wit_parser::{TypeDefKind, TypeId};

impl State {
    pub fn find_all_items(&mut self, items: &[Item], current_path: Vec<String>) {
        for item in items {
            match item {
                // export functions
                Item::Trait(trait_item) if matches!(trait_item.vis, Visibility::Public(_)) => {
                    let trait_item = trait_item.clone();
                    let resource = get_resource_from_trait_name(&trait_item.ident.to_string());
                    let funcs = self.funcs_mut(&current_path, resource);
                    for item in &trait_item.items {
                        if let syn::TraitItem::Fn(method) = item {
                            funcs.push(method.sig.clone());
                        }
                    }
                    self.traits
                        .entry(current_path.clone())
                        .or_default()
                        .push(trait_item);
                }
                // import resource functions
                Item::Impl(impl_item) if impl_item.trait_.is_none() => {
                    let resource = if let Type::Path(type_path) = &*impl_item.self_ty {
                        type_path.path.segments.last().unwrap().ident.to_string()
                    } else {
                        unreachable!()
                    };
                    let funcs = self.funcs_mut(&current_path, Some(resource));
                    for item in &impl_item.items {
                        if let syn::ImplItem::Fn(method) = item {
                            if !matches!(method.vis, Visibility::Public(_)) {
                                continue;
                            }
                            if has_doc_hidden(&method.attrs) {
                                continue;
                            }
                            funcs.push(method.sig.clone());
                        }
                    }
                }
                // import top-level functions
                Item::Fn(func_item)
                    if matches!(func_item.vis, Visibility::Public(_))
                        && current_path.len() >= 3 =>
                {
                    self.funcs_mut(&current_path, None)
                        .push(func_item.sig.clone());
                }
                // records and resources. Rust types without a WIT type, e.g. the `XBorrow`
                // of an exported resource, are skipped.
                Item::Struct(struct_item) if matches!(struct_item.vis, Visibility::Public(_)) => {
                    let Some(id) = self.wit_type_id(&current_path, &struct_item.ident.to_string())
                    else {
                        continue;
                    };
                    let type_info = if matches!(self.resolve.types[id].kind, TypeDefKind::Resource)
                    {
                        TypeInfo::Resource(struct_item.clone())
                    } else {
                        let mut struct_item = struct_item.clone();
                        let mut transformer = FullTypePath {
                            module_path: &current_path,
                        };
                        transformer.visit_item_struct_mut(&mut struct_item);
                        TypeInfo::Struct(struct_item)
                    };
                    self.push_type(&current_path, id, type_info);
                }
                // variants and enums
                Item::Enum(enum_item) if matches!(enum_item.vis, Visibility::Public(_)) => {
                    let Some(id) = self.wit_type_id(&current_path, &enum_item.ident.to_string())
                    else {
                        continue;
                    };
                    let mut enum_item = enum_item.clone();
                    let mut transformer = FullTypePath {
                        module_path: &current_path,
                    };
                    transformer.visit_item_enum_mut(&mut enum_item);
                    self.push_type(&current_path, id, TypeInfo::Enum(enum_item));
                }
                // flags
                Item::Macro(macro_item) => {
                    let Some(name) = bitflags_name(macro_item) else {
                        continue;
                    };
                    let Some(id) = self.wit_type_id(&current_path, &name.to_string()) else {
                        continue;
                    };
                    let TypeDefKind::Flags(flags) = &self.resolve.types[id].kind else {
                        unreachable!()
                    };
                    // wit-bindgen names the constants of flags in SHOUTY_SNAKE_CASE.
                    let flags = flags
                        .flags
                        .iter()
                        .map(|f| format_ident!("{}", f.name.to_shouty_snake_case()))
                        .collect();
                    self.push_type(&current_path, id, TypeInfo::Flag(ItemFlag { name, flags }));
                }
                // traverse down the modules
                Item::Mod(module) if matches!(module.vis, Visibility::Public(_)) => {
                    if let Some((_, ref mod_items)) = module.content {
                        let mut new_path = current_path.clone();
                        let mod_name = module.ident.to_string();
                        // wit-bindgen's runtime, and its `wit_stream`/`wit_future` modules, whose
                        // payload traits aren't WIT items.
                        if current_path.is_empty() {
                            match mod_name.as_str() {
                                "_rt" => continue,
                                "wit_stream" => {
                                    self.has_streams = true;
                                    continue;
                                }
                                "wit_future" => {
                                    self.has_futures = true;
                                    continue;
                                }
                                _ => (),
                            }
                        }
                        new_path.push(mod_name);
                        self.module_paths.insert(new_path.clone());
                        self.find_all_items(mod_items, new_path);
                    }
                }
                _ => {}
            }
        }
    }
    fn funcs_mut(
        &mut self,
        module_path: &[String],
        resource: Option<String>,
    ) -> &mut Vec<Signature> {
        self.funcs
            .entry(module_path.to_vec())
            .or_default()
            .entry(resource)
            .or_default()
    }
    fn push_type(&mut self, module_path: &[String], id: TypeId, type_info: TypeInfo) {
        self.types
            .entry(module_path.to_vec())
            .or_default()
            .push((id, type_info));
    }
    pub fn find_function(
        &self,
        module_path: &[String],
        resource: Option<&str>,
        func: &Ident,
    ) -> Option<&Signature> {
        let module = self.funcs.get(module_path)?;
        let funcs = module.get(&resource.map(str::to_string))?;
        funcs.iter().find(|sig| sig.ident == *func)
    }
    pub fn has_type_def(&self, module_path: &[String], name: &str) -> bool {
        let types = match self.types.get(module_path) {
            Some(types) => types,
            None => return false,
        };
        for (_, type_info) in types {
            match type_info {
                TypeInfo::Resource(struct_item) | TypeInfo::Struct(struct_item) => {
                    if struct_item.ident == name {
                        return true;
                    }
                }
                TypeInfo::Enum(enum_item) => {
                    if enum_item.ident == name {
                        return true;
                    }
                }
                TypeInfo::Flag(item_flag) => {
                    if item_flag.name == name {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn has_doc_hidden(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("doc")
            && let syn::Meta::List(meta_list) = &attr.meta
            && meta_list.to_token_stream().to_string().contains("hidden")
        {
            return true;
        }
    }
    false
}
/// The name of the type a `bitflags!` macro defines, e.g. `Name` in
/// `bitflags! { #[attrs] pub struct Name: u8 { const A = 1 << 0; } }`.
fn bitflags_name(macro_item: &syn::ItemMacro) -> Option<Ident> {
    if macro_item.mac.path.segments.last()?.ident != "bitflags" {
        return None;
    }
    let mut tokens = macro_item.mac.tokens.clone().into_iter();
    tokens.find(|t| matches!(t, TokenTree::Ident(ident) if ident == "struct"))?;
    match tokens.next()? {
        TokenTree::Ident(name) => Some(name),
        _ => None,
    }
}
