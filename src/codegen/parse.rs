use super::util::{FullTypePath, get_resource_from_trait_name};
use super::{ItemFlag, State, TypeInfo};
use quote::ToTokens;
use syn::{Ident, Item, Signature, Type, Visibility, visit_mut::VisitMut};

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
                // resource and struct types
                Item::Struct(struct_item) if matches!(struct_item.vis, Visibility::Public(_)) => {
                    let has_repr_transparent = struct_item.attrs.iter().any(|attr| {
                        attr.path().is_ident("repr")
                            && attr.to_token_stream().to_string().contains("transparent")
                    });
                    let type_info = if has_repr_transparent {
                        if struct_item.ident.to_string().ends_with("Borrow")
                            && current_path[0] == "exports"
                        {
                            continue;
                        }
                        TypeInfo::Resource(struct_item.clone())
                    } else {
                        let mut struct_item = struct_item.clone();
                        let mut transformer = FullTypePath {
                            module_path: &current_path,
                        };
                        transformer.visit_item_struct_mut(&mut struct_item);
                        TypeInfo::Struct(struct_item)
                    };
                    self.push_type(&current_path, type_info);
                }
                // enum types
                Item::Enum(enum_item) if matches!(enum_item.vis, Visibility::Public(_)) => {
                    let mut enum_item = enum_item.clone();
                    let mut transformer = FullTypePath {
                        module_path: &current_path,
                    };
                    transformer.visit_item_enum_mut(&mut enum_item);
                    self.push_type(&current_path, TypeInfo::Enum(enum_item));
                }
                // type aliases, e.g. the resources used by the conversion interface
                Item::Type(type_item) if matches!(type_item.vis, Visibility::Public(_)) => {
                    let mut path = current_path.clone();
                    path.push(type_item.ident.to_string());
                    self.type_aliases.insert(path, (*type_item.ty).clone());
                }
                // flags
                Item::Macro(macro_item) => {
                    if let Some(enum_item) = extract_bitflag(macro_item) {
                        self.push_type(&current_path, TypeInfo::Flag(enum_item));
                    }
                }
                // traverse down the modules
                Item::Mod(module) if matches!(module.vis, Visibility::Public(_)) => {
                    if let Some((_, ref mod_items)) = module.content {
                        let mut new_path = current_path.clone();
                        let mod_name = module.ident.to_string();
                        if current_path.is_empty() && mod_name == "_rt" {
                            continue;
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
    fn push_type(&mut self, module_path: &[String], type_info: TypeInfo) {
        self.types
            .entry(module_path.to_vec())
            .or_default()
            .push(type_info);
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
        for type_info in types {
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
fn extract_bitflag(macro_item: &syn::ItemMacro) -> Option<ItemFlag> {
    use syn::{Attribute, Token, parse::Parser};
    if macro_item.mac.path.segments.last()?.ident == "bitflags" {
        let tokens = macro_item.mac.tokens.clone();
        let parser = |input: syn::parse::ParseStream| {
            input.call(Attribute::parse_outer)?;
            input.parse::<syn::Visibility>()?;
            input.parse::<Token![struct]>()?;
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            input.parse::<Type>()?;
            let content;
            syn::braced!(content in input);
            let mut flags = vec![];
            while !content.is_empty() {
                content.parse::<Token![const]>()?;
                let flag_ident: Ident = content.parse()?;
                flags.push(flag_ident);
                while !content.peek(Token![;]) && !content.is_empty() {
                    content.parse::<proc_macro2::TokenTree>()?;
                }
                content.parse::<Token![;]>()?;
            }
            Ok((ident, flags))
        };
        let (name, flags) = parser.parse2(tokens).unwrap();
        return Some(ItemFlag { name, flags });
    }
    None
}
