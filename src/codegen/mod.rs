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
    /// The interfaces of the virtualized host, which the imports component exports
    pub host_interfaces: BTreeSet<InterfaceId>,
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
    if matches!(mode, Mode::Record | Mode::Replay | Mode::Fuzz) {
        state.output.extend(trace_defs());
    }
    let has_executor = state.has_executor();
    if matches!(mode, Mode::Replay | Mode::Fuzz) {
        state.output.extend(job_defs(mode, has_executor));
    }
    if matches!(has_executor, Some(HasExecutor::Imports)) {
        state.output.extend(export_call_defs(mode));
    }
    if matches!(mode, Mode::Record) {
        state.output.extend(record_defs(has_executor));
    }
    if matches!(mode, Mode::Replay | Mode::Fuzz) {
        let defs = state.replay_defs();
        state.output.extend(defs);
    }
    if matches!(mode, Mode::Fuzz | Mode::Dialog) {
        let defs = state.mock_defs();
        state.output.extend(defs);
    }
    let mut file = state.into_output_file();
    // The imports component's jobs run in the executor, and the exports component's on the task
    // of `start`.
    match has_executor {
        Some(HasExecutor::Exports) => {
            let executor = syn::parse_file(include_str!("../../assets/executor.rs"))?;
            file.items.extend(executor.items);
        }
        Some(HasExecutor::Imports) => file.items.extend(start_spawn_defs()),
        None => {}
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
            host_interfaces: wit::host_interfaces(&resolve, world),
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
                        methods.push(syn::ImplItem::Fn(self.generate_executor_func(&sig)));
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
            (Mode::Replay | Mode::Fuzz, Some(_)) => {
                methods.push(syn::ImplItem::Fn(self.generate_mock_drop()))
            }
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
            // In fuzz and dialog modes, the mock gets the next label (see `next_mock_label`); in
            // replay mode, the trace's.
            match self.mode {
                Mode::Fuzz | Mode::Dialog => quote! {
                    let _ = handle;
                    #resource::new(MockedResource { handle: next_mock_label(), name: #name.to_string() })
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
                    // The host's own functions, which the main component imports, aren't called,
                    // as the real host doesn't call them through the proxy. A trace could have
                    // such a call, but not one with streams or futures, e.g. `body.consume` of a
                    // body the host made: both ends would be in the generated components, whose
                    // copies replay can't place in their steps.
                    if self.host_interfaces.contains(&self.interfaces[path]) {
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
                        is_async: wit_func.is_async,
                        ret_async: wit_func.result_async,
                    });
                }
            }
        }
        res
    }
    /// Whether this component exports `proxy:conversion/executor` or imports it; `None` for
    /// bindings without it, e.g. of the tests.
    fn has_executor(&self) -> Option<HasExecutor> {
        let import = EXECUTOR_MODULE.strip_prefix("exports::").unwrap();
        self.module_paths
            .iter()
            .find_map(|path| match path.join("::") {
                path if path == EXECUTOR_MODULE => Some(HasExecutor::Exports),
                path if path == import => Some(HasExecutor::Imports),
                _ => None,
            })
    }
    /// The async `start` function running `body`. The imports component's jobs run in
    /// `proxy:conversion/executor` until the exports `start` called are done, and the jobs of this
    /// component, which `start` made, on the task of `start` (see `start_spawn_defs`). In dialog
    /// mode, `body` also waits for them after each export call (see `generate_dialog_start_func`),
    /// and runs `__executor` again.
    fn generate_start(&self, sig: &Signature, body: TokenStream) -> syn::ImplItemFn {
        let executor = match self.mode {
            Mode::Dialog => quote! { mut __executor },
            _ => quote! { __executor },
        };
        // Replay checks that it reached every line, once everything is done.
        let finish = match self.mode {
            Mode::Replay => quote! { proxy::recorder::replay::finish(); },
            _ => quote! {},
        };
        let panic_hook = panic_hook(&self.mode);
        parse_quote! {
            #sig {
                #panic_hook
                let #executor = wit_bindgen::spawn_local(proxy::conversion::executor::run());
                #body
                join_spawned().await;
                proxy::conversion::executor::stop();
                __executor.await;
                #finish
            }
        }
    }
    /// What an import's mock does with the arguments the guest moved to the host, in fuzz and
    /// replay modes: `consume`, the mode's, of each stream and future, and a drop of the other
    /// resources. These are the host's drops, not the guest's, so `MOVING` is set; and they happen
    /// before the call can wait, while no other task runs.
    fn take_moved_args(
        &self,
        wit_func: &wit::WitFunc,
        args: &[util::ArgInfo],
        consume: impl Fn(&Ident) -> TokenStream,
    ) -> TokenStream {
        let consumed = wit_func.consumed_args(args).into_iter().map(consume);
        let owned: Vec<_> = args
            .iter()
            .zip(&wit_func.params_own)
            .zip(&wit_func.params_async)
            .filter(|((arg, owns), is_async)| **owns && !**is_async && !arg.is_borrowed)
            .map(|((arg, _), _)| &arg.ident)
            .collect();
        let drop_owned = if owned.is_empty() {
            quote! {}
        } else {
            quote! { drop((#(#owned,)*)); }
        };
        quote! {
            MOVING.with(|moving| moving.set(true));
            #(#consumed;)*
            #drop_owned
            MOVING.with(|moving| moving.set(false));
        }
    }
    /// An export call of `start`, in fuzz and replay modes: `args` binds the arguments, `before`
    /// runs before the call, with `__id` the call's, and `after` gets the result, `res`. The
    /// guest's `return` is reported (see `trace_guest`), and the arguments the call borrowed are
    /// dropped at the end of their block, which, like those of the result, are the host's drops.
    fn start_call(
        &self,
        func: &ExportFunc,
        args: TokenStream,
        before: TokenStream,
        after: TokenStream,
    ) -> TokenStream {
        let ExportFunc {
            func,
            call_params,
            is_async,
            has_ret,
            ..
        } = func;
        // An async call that blocks gives the host a subtask, which the guest started.
        let call = if *is_async {
            quote! { call_export(__id, #func(#(#call_params),*)).await }
        } else {
            quote! { #func(#(#call_params),*) }
        };
        let value = if *has_ret {
            quote! { Some(wasm_wave::to_string(&res.to_value()).unwrap()) }
        } else {
            quote! { None }
        };
        quote! {
            let res = {
                #args
                #before
                let res = #call;
                trace_guest(&trace_return(Party::Guest, __id, #value));
                proxy::conversion::executor::host_drops(true);
                res
            };
            #after
            SCOPED_ALLOC.with(|alloc| alloc.borrow_mut().clear());
            proxy::conversion::executor::host_drops(false);
        }
    }
    /// The functions of `proxy:conversion/executor`, which the imports component exports, for the
    /// `start` function of the exports component to run its jobs (see `assets/executor.rs`).
    fn generate_executor_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let body = match (sig.ident.to_string().as_str(), &self.mode) {
            // In dialog mode, `start` runs the executor again after each export call.
            ("run", Mode::Dialog) => quote! {
                run_executor().await;
                EXECUTOR.with_borrow_mut(|e| e.stopped = false);
            },
            // In record mode, the exports component runs it for each call of an async export (see
            // `call_export_recorded`): a call while it runs, or still finishes its jobs after a
            // `stop`, keeps it running instead.
            ("run", Mode::Record) => quote! {
                thread_local! {
                    static RUNNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
                }
                if RUNNING.replace(true) {
                    EXECUTOR.with_borrow_mut(|e| e.stopped = false);
                    return;
                }
                run_executor().await;
                RUNNING.set(false);
                EXECUTOR.with_borrow_mut(|e| e.stopped = false);
            },
            ("run", _) => {
                let panic_hook = panic_hook(&self.mode);
                quote! {
                    #panic_hook
                    run_executor().await
                }
            }
            ("stop", _) => quote! { stop_executor() },
            ("step", _) => quote! { next_step() },
            ("current_step", _) => quote! { current_step() },
            ("wait_until_step", _) => quote! { wait_until_step(step).await },
            ("step_after", _) => quote! { step_after(step).await },
            ("host_drops", Mode::Replay | Mode::Fuzz) => {
                quote! { MOVING.with(|moving| moving.set(dropping)) }
            }
            ("host_drops", _) => quote! { let _ = dropping; },
            (name, _) => unreachable!("unexpected executor function {name}"),
        };
        parse_quote! {
            #sig {
                #body
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

/// How a component has `proxy:conversion/executor`, in the modes with `start`.
#[derive(Clone, Copy)]
enum HasExecutor {
    /// It exports it, to run its jobs: the imports component.
    Exports,
    /// It imports it, which its `start` runs: the exports component, whose jobs run on the task
    /// of `start`.
    Imports,
}

/// Items for the jobs of replay and fuzz modes, which differ by where the jobs run: the table the
/// component labels its handles in; `trace_at`, the step in which a job makes a host line, and
/// `trace_reach`, which waits for it in replay; and `host_dropping`, which runs `f`, dropping
/// mocks for the host, e.g. those in a value a job read from the guest: the host's drops, which
/// take no step, as the guest doesn't see them.
fn job_defs(mode: &Mode, has_executor: Option<HasExecutor>) -> Vec<Item> {
    let file: File = match has_executor {
        // The imports component's jobs run in the executor: a line's step is the executor's run.
        Some(HasExecutor::Exports) => parse_quote! {
            #[allow(dead_code)]
            const TRACE_TABLE: Table = Table::Imports;
            #[allow(dead_code)]
            fn trace_at() -> u32 { current_step() }
            #[allow(dead_code)]
            async fn trace_reach(at: Option<u32>) {
                if let Some(at) = at {
                    reach_step(at).await;
                }
            }
            #[allow(dead_code)]
            fn host_dropping<R>(f: impl FnOnce() -> R) -> R {
                MOVING.with(|moving| moving.set(true));
                let res = f();
                MOVING.with(|moving| moving.set(false));
                res
            }
        },
        // The exports component's jobs run on the task of `start`: a line's step is the last
        // step, which replay's waits for.
        Some(HasExecutor::Imports) => parse_quote! {
            #[allow(dead_code)]
            const TRACE_TABLE: Table = Table::Exports;
            #[allow(dead_code)]
            fn trace_at() -> u32 { proxy::conversion::executor::current_step() }
            #[allow(dead_code)]
            async fn trace_reach(at: Option<u32>) {
                if let Some(at) = at {
                    proxy::conversion::executor::wait_until_step(at).await;
                }
            }
            #[allow(dead_code)]
            fn host_dropping<R>(f: impl FnOnce() -> R) -> R {
                proxy::conversion::executor::host_drops(true);
                let res = f();
                proxy::conversion::executor::host_drops(false);
                res
            }
        },
        None => return Vec::new(),
    };
    let mut items = file.items;
    if matches!(mode, Mode::Replay) && matches!(has_executor, Some(HasExecutor::Exports)) {
        let file: File = parse_quote! {
            /// The value of the `return` that answers an async call: at once, if the trace has
            /// the call RETURNED, or, if STARTED, in the step of its `return`, which the mock
            /// waits for.
            async fn trace_answer(id: TaskId, answers: Vec<Line>) -> Option<String> {
                if !answers.iter().any(|line| matches!(line.event, Event::Started(_))) {
                    return trace_return_value(answers);
                }
                let line = proxy::recorder::replay::take_host_line(&LineKey::Task(id))
                    .expect("the trace doesn't return the call");
                let Event::Return(ret) = line.event else {
                    panic!("the trace answers the call with {:?}", line.event);
                };
                // A STARTED call blocks, so it returns in a later step than the one it was
                // called in; if replay is there already, it is ahead of the recording.
                let at = line.at.expect("a STARTED call returns in a step");
                let now = current_step();
                assert!(now < at, "replay is in step {now}, past step {at} of the return of {id:?}");
                wait_until_step(at).await;
                let now = current_step();
                assert_eq!(now, at, "replay returned {id:?} in step {now}, not in step {at}");
                ret.value
            }
        };
        items.extend(file.items);
    }
    items
}

/// Helpers of the exports component's calls of the main component's exports, which record,
/// fuzz and replay modes share, so that they stay alike, as replay requires: record and fuzz
/// modes record a guest line, which replay checks.
fn export_call_defs(mode: &Mode) -> Vec<Item> {
    let trace_guest = match mode {
        Mode::Record | Mode::Fuzz => quote! { proxy::recorder::record::record(line); },
        Mode::Replay => quote! { proxy::recorder::replay::check(line); },
        Mode::Dialog => return Vec::new(),
    };
    let file: File = parse_quote! {
        /// A line of the guest, which record and fuzz modes record, and replay checks.
        #[allow(dead_code)]
        fn trace_guest(line: &Line) {
            #trace_guest
        }
        /// Calls an async export, and reports that the guest started it, if it blocks.
        #[allow(dead_code)]
        async fn call_export<T>(id: TaskId, call: impl std::future::Future<Output = T>) -> T {
            let mut call = std::pin::pin!(call);
            let mut started = false;
            std::future::poll_fn(|cx| {
                let poll = call.as_mut().poll(cx);
                if poll.is_pending() && !started {
                    started = true;
                    trace_guest(&Line { by: Party::Guest, at: None, event: Event::Started(id) });
                }
                poll
            })
            .await
        }
    };
    file.items
}

/// Helpers of record mode, which forwards the main component's calls to the host, and the host's
/// to it, for each component.
fn record_defs(has_executor: Option<HasExecutor>) -> Vec<Item> {
    let file: File = match has_executor {
        // The imports component, whose executor awaits the host's async calls.
        Some(HasExecutor::Exports) => parse_quote! {
            /// The host's result of an async call: at once, if it is ready on the first poll, so
            /// that the guest sees RETURNED, and otherwise from an executor job, which awaits the
            /// host's subtask, with the step of the run it completed in, as replay returns it in
            /// that step. The call may borrow the guest's arguments, e.g. a method's `self`.
            #[allow(dead_code)]
            async fn host_result<T: 'static>(
                id: TaskId,
                call: impl std::future::Future<Output = T>,
            ) -> (T, Option<u32>) {
                let mut call = Box::pin(call);
                let first = std::future::poll_fn(|cx| std::task::Poll::Ready(call.as_mut().poll(cx))).await;
                if let std::task::Poll::Ready(value) = first {
                    return (value, None);
                }
                proxy::recorder::record::record(&Line { by: Party::Host, at: None, event: Event::Started(id) });
                let (value, at) = wait_for_job(call).await;
                (value, Some(at))
            }
        },
        // The exports component, which runs the imports component's executor while the host's
        // calls of async exports are pending: there is no `start` to run it.
        Some(HasExecutor::Imports) => parse_quote! {
            thread_local! {
                // The host's calls of async exports that haven't returned.
                static PENDING_CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
            }
            /// Calls an async export, as `call_export` does, running the executor until the last
            /// pending call returns, then stopping it, which lets it finish its jobs and return.
            #[allow(dead_code)]
            async fn call_export_recorded<T>(id: TaskId, call: impl std::future::Future<Output = T>) -> T {
                PENDING_CALLS.set(PENDING_CALLS.get() + 1);
                spawn(proxy::conversion::executor::run());
                let res = call_export(id, call).await;
                PENDING_CALLS.set(PENDING_CALLS.get() - 1);
                if PENDING_CALLS.get() == 0 {
                    proxy::conversion::executor::stop();
                }
                res
            }
        },
        None => return Vec::new(),
    };
    file.items
}

/// Sends the panics of the component to the recorder, which reports them (see `fail`), in
/// fuzz and replay modes: the generated components have no stderr of their own to print them,
/// e.g. when the main component's is virtualized.
fn panic_hook(mode: &Mode) -> TokenStream {
    let fail = match mode {
        Mode::Fuzz => quote! { proxy::recorder::record::fail },
        Mode::Replay => quote! { proxy::recorder::replay::fail },
        _ => return quote! {},
    };
    quote! {
        static PANIC_HOOK: std::sync::Once = std::sync::Once::new();
        PANIC_HOOK.call_once(|| std::panic::set_hook(Box::new(|info| #fail(&info.to_string()))));
    }
}

/// Helpers of record, replay and fuzz modes, which make the lines of `proxy:recorder/types`. Each
/// component numbers the calls it sees with its own counter: the exports component the host's
/// calls of exports, and the imports component the guest's calls of imports (see
/// `crates/trace`).
fn trace_defs() -> Vec<Item> {
    let file: File = parse_quote! {
        use bindings::proxy::recorder::types::*;
        thread_local! {
            // The number of the last call this component saw.
            static TASK_ID: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
        }
        #[allow(dead_code)]
        fn trace_task_id(table: Table) -> TaskId {
            let id = TASK_ID.with(|id| {
                id.set(id.get() + 1);
                id.get()
            });
            TaskId { table, id }
        }
        #[allow(dead_code)]
        fn trace_call(by: Party, id: TaskId, func: &str, is_async: bool, args: Vec<String>) -> Line {
            let call = Call { id, is_async, func: func.to_string(), args };
            Line { by, at: None, event: Event::Call(call) }
        }
        #[allow(dead_code)]
        fn trace_return(by: Party, id: TaskId, value: Option<String>) -> Line {
            let ret = ReturnValue { id, value };
            Line { by, at: None, event: Event::Return(ret) }
        }
        #[allow(dead_code)]
        fn trace_drop(by: Party, table: Table, name: String) -> Line {
            let label = Label { table, name };
            Line { by, at: None, event: Event::Drop(label) }
        }
        /// The value of the `return` among the host's answers to a call.
        #[allow(dead_code)]
        fn trace_return_value(answers: Vec<Line>) -> Option<String> {
            answers
                .into_iter()
                .find_map(|line| match line.event {
                    Event::Return(ret) => Some(ret.value),
                    _ => None,
                })
                .expect("the trace doesn't answer the call")
        }
    };
    file.items
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
        #[allow(dead_code)]
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
