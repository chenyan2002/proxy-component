use super::util::{extract_arg_info, get_return_type, make_path, params_to_wave};
use super::{ExportFunc, State, TypeInfo};
use quote::quote;
use syn::{Item, Signature, parse_quote};

impl State {
    pub fn generate_replay_import_func(
        &self,
        module_path: &[String],
        sig: &Signature,
        resource: Option<&str>,
    ) -> syn::ImplItemFn {
        let func_name = &sig.ident;
        let wit_func = self.expect_wit_func(module_path, resource, func_name);
        let kind = wit_func.kind;
        let display_name = &wit_func.display_name;
        let is_async = wit_func.is_async;
        let args = extract_arg_info(sig);
        let ret_ty = get_return_type(&sig.output);
        // Checks the call, and takes the value of the `return` that answers it: for an async
        // call, at once or in its step, as the trace has it RETURNED or STARTED.
        let answer = if is_async {
            quote! { trace_answer(__id, proxy::recorder::replay::check(&__call)).await }
        } else {
            quote! { trace_return_value(proxy::recorder::replay::check(&__call)) }
        };
        let check_call = quote! {
            let __id = trace_task_id(Table::Imports);
            let __call = trace_call(Party::Guest, __id, #display_name, #is_async, __params);
            let wave = #answer;
        };
        let replay_import = if let Some(ret_ty) = ret_ty {
            let to_rust = match wit_func.constructor_resource() {
                Some(name) => {
                    quote! { MockedResource { name: #name.to_string(), ..FromWave::from_wave(&ret) } }
                }
                None => quote! { FromWave::from_wave(&ret) },
            };
            quote! {
                #check_call
                let ret: Value = wasm_wave::from_str(&<#ret_ty as ValueTyped>::value_type(), &wave.unwrap()).unwrap();
                #to_rust
            }
        } else {
            quote! {
                #check_call
                assert!(wave.is_none());
            }
        };
        let params = params_to_wave(kind.as_ref(), &args);
        // The host takes the resources the arguments own, so dropping them is not a drop.
        let owned_args: Vec<_> = args
            .iter()
            .zip(&wit_func.params_own)
            .filter(|(_, owns)| **owns)
            .map(|(arg, _)| &arg.ident)
            .collect();
        if owned_args.is_empty() {
            return parse_quote! {
                #sig {
                    #params
                    #replay_import
                }
            };
        }
        parse_quote! {
            #sig {
                #params
                let __ret = { #replay_import };
                MOVING.with(|moving| moving.set(true));
                drop((#(#owned_args,)*));
                MOVING.with(|moving| moving.set(false));
                __ret
            }
        }
    }
    /// Items the replay functions use.
    pub fn replay_defs(&self) -> Vec<Item> {
        // `MOVING` is set while an import drops the resources its arguments own, which the guest
        // moved to the host, so that `generate_mock_drop` doesn't take them for the guest's drops.
        //
        // Only replay and fuzz modes need it. In record mode, the import passes the host resource
        // of each argument on to the host, taking it out with `into_inner` (see `ToProxy`), so the
        // destructor finds no resource to record. With mocks there's no host to take them: the
        // import drops its arguments, and the destructors find their mocked resources. A flag
        // spares a type-directed trait like `ToProxy`, to take the resources out of nested
        // arguments, e.g. `list<counter>`. A thread-local, as `[dtor]`s take no other arguments.
        let file: syn::File = parse_quote! {
            thread_local! {
                #[allow(dead_code)]
                static MOVING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
            }
        };
        file.items
    }
    /// The arm of the `start` function for the host's drop of a resource of the main component,
    /// which the trace labels by its handle in this component, e.g. `#session-1`.
    fn replay_export_drop(&self) -> Option<proc_macro2::TokenStream> {
        let resources: Vec<_> = self
            .types
            .iter()
            .filter(|(module_path, _)| module_path[0] != "exports")
            .flat_map(|(module_path, types)| types.iter().map(move |ty| (module_path, ty)))
            .filter_map(|(module_path, (id, info))| match info {
                // Host resources have a `proxy:conversion` function, as they can be mocked.
                TypeInfo::Resource(item) if !self.conversions.contains_key(id) => {
                    let name = self.resolve.types[*id].name.clone().unwrap();
                    Some((name, make_path(module_path, &item.ident.to_string())))
                }
                _ => None,
            })
            .collect();
        if resources.is_empty() {
            return None;
        }
        let arms = resources.iter().map(|(name, path)| {
            quote! {
                #name => drop(unsafe { #path::from_handle(handle) }),
            }
        });
        Some(quote! {
            Event::Drop(label) => {
                let (name, handle) = label
                    .name
                    .strip_prefix('#')
                    .and_then(|label| label.rsplit_once('-'))
                    .and_then(|(name, handle)| Some((name, handle.parse::<u32>().ok()?)))
                    .expect("invalid handle label");
                match name {
                    #(#arms)*
                    _ => unreachable!(),
                }
            }
        })
    }
    pub fn generate_replay_start_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let arms = self.export_funcs().into_iter().map(|func| {
            let ExportFunc {
                display_name,
                func,
                arg_names,
                arg_tys,
                call_params,
                has_ret,
                ret_owns,
                is_async,
                ..
            } = func;
            // An async call that blocks gives the host a subtask, which the guest started.
            let call = if is_async {
                quote! { call_export(__id, #func(#(#call_params),*)).await }
            } else {
                quote! { #func(#(#call_params),*) }
            };
            let arg_idx = (0..arg_names.len()).map(|idx| quote! { args[#idx] });
            // The export call's id is the trace's, as the trace makes the call.
            let assert_ret = if has_ret {
                quote! {
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::check(&trace_return(Party::Guest, __id, Some(wave_res)));
                }
            } else {
                quote! {
                    assert!(res == ());
                    proxy::recorder::replay::check(&trace_return(Party::Guest, __id, None));
                }
            };
            // The host keeps the resources it gets, until the trace drops them.
            let keep_ret = if ret_owns {
                quote! { std::mem::forget(res); }
            } else {
                quote! {}
            };
            // The arguments the call borrowed are dropped at the end of the block, which are
            // the host's drops, as in fuzz mode, not the guest's.
            quote! {
                #display_name => {
                    let res = {
                        #(
                            let arg_value: Value = wasm_wave::from_str(&<#arg_tys as ValueTyped>::value_type(), &#arg_idx).unwrap();
                            let #arg_names: #arg_tys = FromWave::from_wave(&arg_value);
                        )*
                        let res = #call;
                        proxy::conversion::executor::host_drops(true);
                        res
                    };
                    proxy::conversion::executor::host_drops(false);
                    #assert_ret
                    #keep_ret
                }
            }
        });
        let drop_arm = self.replay_export_drop();
        self.generate_start(
            sig,
            quote! {
                /// Calls an async export, and checks that the guest started it, if it blocks.
                #[allow(dead_code)]
                async fn call_export<T>(id: TaskId, call: impl std::future::Future<Output = T>) -> T {
                    let mut call = std::pin::pin!(call);
                    let mut started = false;
                    std::future::poll_fn(|cx| {
                        let poll = call.as_mut().poll(cx);
                        if poll.is_pending() && !started {
                            started = true;
                            proxy::recorder::replay::check(&Line { by: Party::Guest, at: None, event: Event::Started(id) });
                        }
                        poll
                    })
                    .await
                }
                while let Some(line) = proxy::recorder::replay::next_host_line() {
                    // A line in step `at` follows the step before it, and takes its own step, as
                    // an export call's arrival does.
                    if let Some(at) = line.at {
                        proxy::conversion::executor::wait_until_step(at - 1).await;
                        let step = proxy::conversion::executor::step();
                        assert_eq!(step, at, "replay came to {:?} in step {step}", line.event);
                    }
                    match line.event {
                        Event::Call(Call { id: __id, func, args, .. }) => {
                            match func.as_str() {
                                #(#arms)*
                                _ => unreachable!("the trace calls an unknown export {func}"),
                            }
                        }
                        #drop_arm
                        event => unreachable!("`start` doesn't perform {event:?}"),
                    }
                    // clean up borrowed resources from input args
                    proxy::conversion::executor::host_drops(true);
                    SCOPED_ALLOC.with(|alloc| {
                        alloc.borrow_mut().clear();
                    });
                    proxy::conversion::executor::host_drops(false);
                }
            },
        )
    }
}
