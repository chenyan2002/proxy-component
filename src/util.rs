use crate::codegen::{self, ItemFlag, TypeInfo};
use heck::ToKebabCase;
use quote::ToTokens;
use std::borrow::Cow;
use syn::{FnArg, Ident, Item, Signature, Type, Visibility, parse_quote, visit_mut::VisitMut};

pub struct FullTypePath<'a> {
    pub module_path: &'a [String],
}
pub struct ArgInfo {
    pub ident: Ident,
    pub is_borrowed: bool,
    pub ty: Type,
}
pub enum ResourceFuncKind {
    Method,
    Constructor,
}
impl ArgInfo {
    pub fn call_param(&self) -> syn::Expr {
        let ident = &self.ident;
        if self.is_borrowed {
            parse_quote! { &#ident }
        } else {
            parse_quote! { #ident }
        }
    }
}
pub fn make_path(module_path: &[String], name: &str) -> syn::Path {
    let path = format!("{}::{}", module_path.join("::"), name);
    syn::parse_str(&path).unwrap()
}
pub fn wit_func_name(
    module_path: &[String],
    resource: &Option<String>,
    func_name: &Ident,
    kind: &Option<ResourceFuncKind>,
) -> String {
    assert!(module_path.len() >= 3);
    let mut module_path = module_path.to_vec();
    if module_path[0] == "exports" {
        module_path.remove(0);
    }
    assert!(module_path.len() == 3);
    let mut res = String::new();
    match kind {
        Some(ResourceFuncKind::Constructor) => res.push_str("[constructor]"),
        Some(ResourceFuncKind::Method) => res.push_str("[method]"),
        _ => {}
    }
    let nonwrapped = module_path[0]
        .strip_prefix("wrapped_")
        .unwrap_or(&module_path[0]);
    res.push_str(&nonwrapped.to_kebab_case());
    res.push(':');
    res.push_str(&module_path[1].to_kebab_case());
    res.push('/');
    res.push_str(&module_path[2].to_kebab_case());
    if let Some(name) = resource {
        res.push_str(&format!("/{}", name.to_kebab_case()));
    }
    res.push('.');
    res.push_str(&func_name.to_string().to_kebab_case());
    res
}
pub fn get_return_type(ret: &syn::ReturnType) -> Option<Type> {
    match ret {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => match **ty {
            Type::Tuple(ref tuple) if tuple.elems.is_empty() => None,
            _ => Some(*ty.clone()),
        },
    }
}

pub fn extract_arg_info(sig: &Signature) -> (Option<ResourceFuncKind>, Vec<ArgInfo>) {
    let mut kind = None;
    let mut arg_infos = Vec::new();
    if sig.ident == "new"
        && sig.inputs.is_empty()
        && let Some(Type::Path(path)) = get_return_type(&sig.output)
        && path.path.is_ident("Self")
    {
        return (Some(ResourceFuncKind::Constructor), arg_infos);
    }
    for arg in sig.inputs.iter() {
        match arg {
            FnArg::Receiver(_) => {
                kind = Some(ResourceFuncKind::Method);
            }
            FnArg::Typed(pat_type) => {
                let ident = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => ident.ident.clone(),
                    _ => unreachable!(),
                };
                let ty = *pat_type.ty.clone();
                let is_borrowed = matches!(&*pat_type.ty, Type::Reference(_));
                arg_infos.push(ArgInfo {
                    ident,
                    is_borrowed,
                    ty,
                });
            }
        }
    }
    (kind, arg_infos)
}

const BUILTIN_TYPES: &[&str] = &[
    "Self", "Result", "Option", "Vec", "Box", "Rc", "Arc", "String", "str", "u8", "u16", "u32",
    "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64", "bool",
    "char", "_rt",
];
impl<'a> VisitMut for FullTypePath<'a> {
    fn visit_type_path_mut(&mut self, ty: &mut syn::TypePath) {
        if ty.qself.is_none() && !ty.path.segments.is_empty() && ty.path.leading_colon.is_none() {
            let ident = &ty.path.segments[0].ident.to_string();
            if BUILTIN_TYPES.contains(&ident.as_str()) {
                if ident == "_rt" {
                    assert!(ty.path.segments.len() == 2);
                    ty.path.segments = ty.path.segments.iter().skip(1).cloned().collect();
                }
                syn::visit_mut::visit_type_path_mut(self, ty);
                return;
            }
            let module_idents = self
                .module_path
                .iter()
                .map(|s| syn::parse_str::<syn::Ident>(s).unwrap());
            let original = &ty.path;
            *ty = parse_quote! {
                #(#module_idents)::*::#original
            };
        }
        syn::visit_mut::visit_type_path_mut(self, ty);
    }
}

impl codegen::State {
    pub fn find_all_items(&mut self, items: &[Item], current_path: Vec<String>) {
        for item in items {
            match item {
                // export functions
                Item::Trait(trait_item) if matches!(trait_item.vis, Visibility::Public(_)) => {
                    let trait_item = trait_item.clone();
                    let resource = get_resource_from_trait_name(&trait_item.ident.to_string());
                    let funcs = self
                        .funcs
                        .entry(current_path.clone())
                        .or_default()
                        .entry(resource)
                        .or_default();
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
                    let funcs = self
                        .funcs
                        .entry(current_path.clone())
                        .or_default()
                        .entry(Some(resource))
                        .or_default();
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
                    self.funcs
                        .entry(current_path.clone())
                        .or_default()
                        .entry(None)
                        .or_default()
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
                    self.types
                        .entry(current_path.clone())
                        .or_default()
                        .push(type_info);
                }
                // enum types
                Item::Enum(enum_item) if matches!(enum_item.vis, Visibility::Public(_)) => {
                    let mut enum_item = enum_item.clone();
                    let mut transformer = FullTypePath {
                        module_path: &current_path,
                    };
                    transformer.visit_item_enum_mut(&mut enum_item);
                    self.types
                        .entry(current_path.clone())
                        .or_default()
                        .push(TypeInfo::Enum(enum_item));
                }
                // flags
                Item::Macro(macro_item) => {
                    if let Some(enum_item) = extract_bitflag(macro_item) {
                        self.types
                            .entry(current_path.clone())
                            .or_default()
                            .push(TypeInfo::Flag(enum_item));
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
    pub fn find_function(
        &self,
        module_path: &[String],
        resource: &Option<String>,
        func: &Ident,
    ) -> Option<&Signature> {
        let module = self.funcs.get(module_path)?;
        let funcs = module.get(resource)?;
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

pub fn get_resource_from_trait_name(trait_name: &str) -> Option<String> {
    let resource = trait_name.strip_prefix("Guest").unwrap();
    match resource {
        "" => None,
        name => Some(name.to_string()),
    }
}

pub fn get_owned_type(ty: &Type) -> Option<Type> {
    match ty {
        Type::Reference(type_ref) => {
            match &*type_ref.elem {
                Type::Slice(type_slice) => {
                    let inner_ty = &*type_slice.elem;
                    Some(parse_quote! { Vec<#inner_ty> })
                }
                Type::Path(type_path) => {
                    if type_path.qself.is_none()
                        && type_path.path.segments.len() == 1
                        && type_path.path.segments[0].ident == "str"
                    {
                        Some(parse_quote! { String })
                    } else {
                        // TODO: need to handle nested borrow
                        Some(parse_quote! { #type_path })
                    }
                }
                _ => None,
            }
        }
        _ => None,
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

// utils for WIT names
pub fn ident(name: &str) -> Cow<'_, str> {
    if is_keyword(name) {
        Cow::Owned(format!("%{name}"))
    } else {
        Cow::Borrowed(name)
    }
}
// from https://docs.rs/wit-component/latest/src/wit_component/printing.rs.html#155-192
fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "use"
            | "type"
            | "func"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "s8"
            | "s16"
            | "s32"
            | "s64"
            | "f32"
            | "f64"
            | "float32"
            | "float64"
            | "char"
            | "resource"
            | "record"
            | "flags"
            | "variant"
            | "enum"
            | "bool"
            | "string"
            | "option"
            | "result"
            | "future"
            | "stream"
            | "list"
            | "own"
            | "borrow"
            | "_"
            | "as"
            | "from"
            | "static"
            | "interface"
            | "tuple"
            | "world"
            | "import"
            | "export"
            | "package"
            | "with"
            | "include"
            | "constructor"
            | "error-context"
            | "async"
    )
}
