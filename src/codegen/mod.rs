use crate::Mode;
use anyhow::Result;
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    File, Ident, Item, ItemEnum, ItemStruct, ItemTrait, Signature, TraitItem, Type, parse_quote,
    visit_mut::VisitMut,
};
use util::{
    FullTypePath, ResourceFuncKind, extract_arg_info, get_owned_type, get_resource_from_trait_name,
    get_return_type, make_path, toggle_wrapped_module,
};
use wit_parser::{InterfaceId, Resolve, TypeId, WorldId};

mod dialog;
mod fuzz;
mod parse;
mod record;
mod replay;
pub mod util;
pub mod wit;

pub struct State {
    pub mode: Mode,
    /// The WIT the bindings were generated from
    pub resolve: Resolve,
    /// The interface of each Rust module of the bindings
    pub interfaces: BTreeMap<Vec<String>, InterfaceId>,
    /// The `proxy:conversion` function returning each resource
    pub conversions: BTreeMap<TypeId, String>,
    pub traits: BTreeMap<Vec<String>, Vec<ItemTrait>>,
    pub types: BTreeMap<Vec<String>, Vec<(TypeId, TypeInfo)>>,
    pub funcs: BTreeMap<Vec<String>, BTreeMap<Option<String>, Vec<Signature>>>,
    pub module_paths: BTreeSet<Vec<String>>,
    pub output: Vec<Item>,
}
pub enum TypeInfo {
    Struct(ItemStruct),
    Enum(ItemEnum),
    Resource(ItemStruct),
    Flag(ItemFlag),
}
pub struct ItemFlag {
    pub name: Ident,
    pub flags: Vec<Ident>,
}
/// An exported function of the main component, see `State::export_funcs`.
struct ExportFunc {
    /// WIT name, e.g. `ns:pkg/iface.func`
    display_name: String,
    /// Rust path to the function
    func: syn::Path,
    arg_names: Vec<Ident>,
    /// WIT argument types, e.g. `list<u8>`
    arg_wit_tys: Vec<String>,
    /// Owned argument types, e.g. `String` for `&str`
    arg_tys: Vec<Type>,
    /// Arguments at the call site, borrowed where the function expects a reference
    call_params: Vec<syn::Expr>,
    has_ret: bool,
}

/// Interface for converting resources between the generated components.
const CONVERSION_MODULE: &str = "exports::proxy::conversion::conversion";
/// Interface whose `start` function drives the main component's exports in virtualized modes.
const START_REPLAY_MODULE: &str = "exports::proxy::recorder::start_replay";

/// Generates the `lib.rs` implementing the traits of `bindings`, which wit-bindgen generated
/// from `world` of `resolve`.
pub fn generate(
    mode: &Mode,
    resolve: Resolve,
    world: WorldId,
    bindings: &str,
    use_custom_allocator: bool,
) -> Result<String> {
    let ast = syn::parse_file(bindings)?;
    let mut state = State::new(mode.clone(), resolve, world);
    state.generate_preamble();
    state.find_all_items(&ast.items, vec![]);
    state.generate_stubs();
    let traits = crate::traits::TraitGenerator::new(&state).generate();
    state.output.extend(traits);
    let mut file = state.into_output_file();
    if use_custom_allocator {
        // dlmalloc with a memory source that grows by bytes, for `--page-size=1`.
        let alloc = syn::parse_file(include_str!("../../assets/one_byte_page_alloc.rs"))?;
        let items = alloc.items;
        file.items.push(parse_quote! {
            #[cfg(all(not(target_feature = "atomics"), target_arch = "wasm32"))]
            mod one_byte_page_alloc { #(#items)* }
        });
    }
    Ok(prettyplease::unparse(&file))
}
impl State {
    fn new(mode: Mode, resolve: Resolve, world: WorldId) -> Self {
        State {
            mode,
            interfaces: wit::interface_modules(&resolve, world),
            conversions: wit::conversion_funcs(&resolve),
            resolve,
            traits: BTreeMap::new(),
            types: BTreeMap::new(),
            funcs: BTreeMap::new(),
            module_paths: BTreeSet::new(),
            output: Vec::new(),
        }
    }
    fn generate_stubs(&mut self) {
        for (module_path, traits) in &self.traits {
            for trait_item in traits {
                let impl_item = self.generate_impl_with_methods(trait_item, module_path);
                self.output.push(impl_item);
            }
        }
    }
    fn generate_preamble(&mut self) {
        let file: File = parse_quote! {
          mod bindings;
          use bindings::*;
          struct Stub;
          bindings::export!(Stub with_types_in bindings);
        };
        self.output = file.items;
    }
    fn generate_impl_with_methods(&self, trait_item: &ItemTrait, module_path: &[String]) -> Item {
        let trait_name = &trait_item.ident.to_string();
        let trait_path = make_path(module_path, trait_name);
        let resource = get_resource_from_trait_name(trait_name);
        // Record mode forwards to the imports; the other modes are virtualized.
        let is_record = matches!(self.mode, Mode::Record);
        let stub: syn::Path = match (is_record, &resource) {
            (true, Some(resource)) => {
                let import_path = get_proxy_path(module_path);
                make_path(&import_path, resource)
            }
            (false, Some(_)) => parse_quote! { MockedResource },
            (_, None) => parse_quote! { Stub },
        };
        // Collect all method signatures from the trait
        let mut methods = Vec::new();
        for item in &trait_item.items {
            match item {
                TraitItem::Type(assoc_type) => {
                    let type_name = &assoc_type.ident;
                    let stub: syn::Path = if is_record {
                        let import_path = get_proxy_path(module_path);
                        make_path(&import_path, &type_name.to_string())
                    } else {
                        parse_quote! { MockedResource }
                    };
                    let impl_item = parse_quote! {
                        type #type_name = #stub;
                    };
                    methods.push(syn::ImplItem::Type(impl_item));
                }
                TraitItem::Fn(method) => {
                    let method_name = method.sig.ident.to_string();
                    if method_name.starts_with("_resource_")
                        || (method_name.starts_with("resource") && method_name.ends_with("_"))
                    {
                        continue;
                    }
                    let mut sig = method.sig.clone();
                    let mut transformer = FullTypePath { module_path };
                    transformer.visit_signature_mut(&mut sig);
                    let module_full_name = module_path.join("::");
                    if module_full_name == CONVERSION_MODULE {
                        let stub_impl = self.generate_conversion_func(&sig);
                        methods.push(syn::ImplItem::Fn(stub_impl));
                        continue;
                    }
                    let is_start = module_full_name == START_REPLAY_MODULE;
                    if is_start {
                        assert!(sig.ident == "start");
                    }
                    let stub_impl = match (&self.mode, is_start) {
                        (Mode::Record, _) => {
                            self.generate_record_func(module_path, &sig, resource.as_deref())
                        }
                        (Mode::Replay, false) => {
                            self.generate_replay_import_func(module_path, &sig, resource.as_deref())
                        }
                        (Mode::Replay, true) => self.generate_replay_start_func(&sig),
                        (Mode::Fuzz, false) => {
                            self.generate_fuzz_import_func(module_path, &sig, resource.as_deref())
                        }
                        (Mode::Fuzz, true) => self.generate_fuzz_start_func(&sig),
                        (Mode::Dialog, false) => {
                            self.generate_dialog_import_func(module_path, &sig, resource.as_deref())
                        }
                        (Mode::Dialog, true) => self.generate_dialog_start_func(&sig),
                    };
                    methods.push(syn::ImplItem::Fn(stub_impl));
                }
                _ => (),
            }
        }
        parse_quote! {
            impl #trait_path for #stub {
                #(#methods)*
            }
        }
    }

    fn generate_conversion_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let func_name = &sig.ident.to_string();
        let body = if func_name.starts_with("get_wrapped_") || func_name.starts_with("get_host_") {
            quote! { x.to_proxy() }
        } else if func_name.starts_with("get_mock_") {
            let resource = get_return_type(&sig.output).unwrap();
            let name = self.conversion_resource(&sig.ident);
            quote! {
                #resource::new(MockedResource { handle, name: #name.to_string() })
            }
        } else {
            unreachable!()
        };
        parse_quote! {
            #sig {
                #body
            }
        }
    }
    /// Functions exported by the main component, which the `start` function of the virtualized
    /// modes calls. Resource methods, and Rust functions that aren't WIT functions, are skipped.
    fn export_funcs(&self) -> Vec<ExportFunc> {
        let mut res = Vec::new();
        for (path, resources) in self
            .funcs
            .iter()
            .filter(|(path, _)| path[0] != "exports" && path[0] != "proxy")
        {
            for (resource, sigs) in resources {
                for sig in sigs {
                    let Some(wit_func) = self.wit_func(path, resource.as_deref(), &sig.ident)
                    else {
                        continue;
                    };
                    if matches!(wit_func.kind, Some(ResourceFuncKind::Method)) {
                        continue;
                    }
                    let args = extract_arg_info(sig);
                    let arg_tys = args
                        .iter()
                        .map(|arg| {
                            let mut ty = arg.ty.clone();
                            FullTypePath { module_path: path }.visit_type_mut(&mut ty);
                            get_owned_type(&ty).unwrap_or(ty)
                        })
                        .collect();
                    let func_name = match resource {
                        Some(resource) => format!("{}::{}", resource, sig.ident),
                        None => sig.ident.to_string(),
                    };
                    res.push(ExportFunc {
                        display_name: wit_func.display_name,
                        arg_wit_tys: wit_func.params,
                        func: make_path(path, &func_name),
                        arg_names: args.iter().map(|arg| arg.ident.clone()).collect(),
                        arg_tys,
                        call_params: args.iter().map(|arg| arg.call_param()).collect(),
                        has_ret: get_return_type(&sig.output).is_some(),
                    });
                }
            }
        }
        res
    }
    fn into_output_file(self) -> File {
        File {
            items: self.output,
            shebang: None,
            attrs: Vec::new(),
        }
    }
}

pub fn get_proxy_path(src_path: &[String]) -> Vec<String> {
    assert!(src_path.len() >= 3);
    let mut res = src_path.to_vec();
    let mut wrapped_idx = 0;
    if res[0] == "exports" {
        res.remove(0);
    } else {
        res.insert(0, "exports".to_string());
        wrapped_idx = 1;
    }
    res[wrapped_idx] = toggle_wrapped_module(&res[wrapped_idx]);
    res
}
