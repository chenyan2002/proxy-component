use crate::Mode;
use anyhow::Result;
use proc_macro2::TokenStream;
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
mod mock;
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
    /// What each WIT type has, e.g. own handles
    pub type_info: wit_bindgen_core::Types,
    pub traits: BTreeMap<Vec<String>, Vec<ItemTrait>>,
    pub types: BTreeMap<Vec<String>, Vec<(TypeId, TypeInfo)>>,
    pub funcs: BTreeMap<Vec<String>, BTreeMap<Option<String>, Vec<Signature>>>,
    pub module_paths: BTreeSet<Vec<String>>,
    /// Whether the bindings have streams, i.e. the `wit_stream` module
    pub has_streams: bool,
    /// Whether the bindings have futures, i.e. the `wit_future` module
    pub has_futures: bool,
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
    /// WIT name, e.g. `ns:pkg/iface@0.1.0.func`
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
    /// Whether the result owns a resource
    ret_owns: bool,
    is_async: bool,
    /// Whether the result has a stream or future
    ret_async: bool,
}

/// Interface for converting resources between the generated components.
const CONVERSION_MODULE: &str = "exports::proxy::conversion::conversion";
/// Interface whose `start` function drives the main component's exports in virtualized modes.
const START_REPLAY_MODULE: &str = "exports::proxy::recorder::start_replay";
/// Interface running the background work of the virtualized host, see `assets/executor.rs`.
const EXECUTOR_MODULE: &str = "exports::proxy::conversion::executor";

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
    if matches!(mode, Mode::Replay) {
        let defs = state.replay_defs();
        state.output.extend(defs);
    }
    if matches!(mode, Mode::Fuzz | Mode::Dialog) {
        let defs = state.mock_defs();
        state.output.extend(defs);
    }
    let exports_executor = state.exports_executor();
    let imports_executor = state.imports_executor();
    let mut file = state.into_output_file();
    if exports_executor {
        let executor = syn::parse_file(include_str!("../../assets/executor.rs"))?;
        file.items.extend(executor.items);
    }
    if imports_executor {
        file.items.extend(start_spawn_defs());
    }
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
impl ExportFunc {
    /// The call of the export, awaited if it's async.
    fn call(&self) -> TokenStream {
        let ExportFunc {
            func, call_params, ..
        } = self;
        if self.is_async {
            quote! { #func(#(#call_params),*).await }
        } else {
            quote! { #func(#(#call_params),*) }
        }
    }
}
impl State {
    fn new(mode: Mode, resolve: Resolve, world: WorldId) -> Self {
        State {
            mode,
            interfaces: wit::interface_modules(&resolve, world),
            conversions: wit::conversion_funcs(&resolve),
            has_streams: false,
            has_futures: false,
            type_info: {
                let mut types = wit_bindgen_core::Types::default();
                types.analyze(&resolve);
                types
            },
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
                    if module_full_name == EXECUTOR_MODULE {
                        methods.push(syn::ImplItem::Fn(generate_executor_func(&sig)));
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
        match (&self.mode, &resource) {
            (Mode::Record, Some(_)) => {
                methods.push(syn::ImplItem::Fn(self.generate_record_drop(module_path)))
            }
            (Mode::Replay, Some(_)) => methods.push(syn::ImplItem::Fn(self.generate_replay_drop())),
            (Mode::Fuzz | Mode::Dialog, Some(resource)) => methods.push(syn::ImplItem::Fn(
                self.generate_mock_resource_new(module_path, resource),
            )),
            _ => (),
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
        } else if func_name.starts_with("get_handle_mock_") {
            quote! { x.get::<MockedResource>().handle }
        } else if func_name.starts_with("get_handle_") {
            quote! { x.to_proxy().handle() }
        } else if func_name.starts_with("get_mock_") {
            let resource = get_return_type(&sig.output).unwrap();
            let name = self.conversion_resource(&sig.ident);
            // In fuzz and dialog modes, `_resource_new` labels the mock by its handle (see
            // `generate_mock_resource_new`); in replay mode, by the handle in the trace.
            match self.mode {
                Mode::Fuzz | Mode::Dialog => quote! {
                    let _ = handle;
                    #resource::new(MockedResource { handle: 0, name: #name.to_string() })
                },
                _ => quote! {
                    #resource::new(MockedResource { handle, name: #name.to_string() })
                },
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
                        ret_owns: wit_func.result_owns,
                        is_async: wit_func.is_async,
                        ret_async: wit_func.result_async,
                    });
                }
            }
        }
        res
    }
    /// Whether this is the imports component, which exports `proxy:conversion/executor` to run its
    /// jobs.
    fn exports_executor(&self) -> bool {
        self.module_paths
            .iter()
            .any(|path| path.join("::") == EXECUTOR_MODULE)
    }
    /// Whether this is the exports component, whose `start` runs `proxy:conversion/executor`.
    fn imports_executor(&self) -> bool {
        let import = EXECUTOR_MODULE.strip_prefix("exports::").unwrap();
        self.module_paths
            .iter()
            .any(|path| path.join("::") == import)
    }
    /// The async `start` function running `body`. The imports component's jobs run in
    /// `proxy:conversion/executor` until the exports `start` called are done, and the jobs of this
    /// component, which `start` made, on the task of `start` (see `start_spawn_defs`).
    fn generate_start(&self, sig: &Signature, body: TokenStream) -> syn::ImplItemFn {
        parse_quote! {
            #sig {
                let __executor = wit_bindgen::spawn_local(proxy::conversion::executor::run());
                #body
                join_spawned().await;
                proxy::conversion::executor::stop();
                __executor.await;
            }
        }
    }
    fn into_output_file(self) -> File {
        File {
            items: self.output,
            shebang: None,
            attrs: Vec::new(),
        }
    }
}

/// `spawn` of the exports component, whose jobs, e.g. reading a stream an export returned, are
/// only made by `start`, which is async: it spawns them on the task of `start`, which awaits them
/// with `join_spawned`. The imports component's `spawn` queues them instead, as sync functions
/// make them (see `assets/executor.rs`).
fn start_spawn_defs() -> Vec<Item> {
    let file: File = parse_quote! {
        thread_local! {
            static SPAWNED: std::cell::RefCell<Vec<wit_bindgen::JoinHandle<()>>> =
                const { std::cell::RefCell::new(Vec::new()) };
        }
        #[allow(dead_code)]
        fn spawn(job: impl std::future::Future<Output = ()> + 'static) {
            let handle = wit_bindgen::spawn_local(job);
            SPAWNED.with_borrow_mut(|spawned| spawned.push(handle));
        }
        // Awaits the spawned jobs, including those they spawn.
        async fn join_spawned() {
            loop {
                let spawned = SPAWNED.with_borrow_mut(std::mem::take);
                if spawned.is_empty() {
                    break;
                }
                for handle in spawned {
                    handle.await;
                }
            }
        }
    };
    file.items
}

/// The functions of `proxy:conversion/executor`, which the imports component exports, for the
/// `start` function of the exports component to run its jobs (see `assets/executor.rs`).
fn generate_executor_func(sig: &Signature) -> syn::ImplItemFn {
    let body = match sig.ident.to_string().as_str() {
        "run" => quote! { run_executor().await },
        "stop" => quote! { stop_executor() },
        name => unreachable!("unexpected executor function {name}"),
    };
    parse_quote! {
        #sig {
            #body
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
