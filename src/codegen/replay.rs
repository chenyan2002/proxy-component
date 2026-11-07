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
        let params = params_to_wave(kind.as_ref(), &args);
        // The streams and futures the guest gives the host, which the trace reads, after the
        // call has checked, and so paired, their labels.
        let moved = self.take_moved_args(&wit_func, &args, |arg| quote! { Replay::consume(#arg) });
        // The value of the `return` that answers the call: for an async call, at once or in its
        // step, as the trace has it RETURNED or STARTED.
        let answer = if is_async {
            quote! {{
                let __cancel = TraceCancel(Some(__id));
                let answer = trace_answer(__id, __answers).await;
                __cancel.done();
                answer
            }}
        } else {
            quote! { trace_return_value(__answers) }
        };
        let ret = if let Some(ret_ty) = ret_ty {
            let replay = match wit_func.constructor_resource() {
                Some(name) => {
                    quote! { MockedResource { name: #name.to_string(), ..Replay::replay(&ret) } }
                }
                None => quote! { Replay::replay(&ret) },
            };
            quote! {
                let ret: Value = wasm_wave::from_str(&<#ret_ty as ValueTyped>::value_type(), &wave.unwrap()).unwrap();
                #replay
            }
        } else {
            quote! { assert!(wave.is_none()); }
        };
        parse_quote! {
            #[allow(unused_variables)]
            #sig {
                #params
                let __id = trace_task_id(Table::Imports);
                let __call = trace_call(Party::Guest, __id, #display_name, #is_async, __params);
                let __answers = proxy::recorder::replay::check(&__call);
                #moved
                let wave = #answer;
                #ret
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
                // E.g. `#session-1`, or `#session-1-2` for a later binding of the number (see
                // `crates/trace`'s renaming): the name ends before the first number.
                let label = label.name.strip_prefix('#').expect("invalid handle label");
                let words: Vec<&str> = label.split('-').collect();
                let number = words
                    .iter()
                    .position(|word| word.parse::<u32>().is_ok())
                    .expect("invalid handle label");
                let (name, handle) = (words[..number].join("-"), words[number].parse::<u32>().unwrap());
                match name.as_str() {
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
                arg_names,
                arg_tys,
                ..
            } = &func;
            let arg_idx = (0..arg_names.len()).map(|idx| quote! { args[#idx] });
            let args = quote! {
                #(
                    let arg_value: Value = wasm_wave::from_str(&<#arg_tys as ValueTyped>::value_type(), &#arg_idx).unwrap();
                    let #arg_names: #arg_tys = Replay::replay(&arg_value);
                )*
            };
            // The streams and futures the guest returns, which the trace reads, and its resources,
            // which the host keeps, until the trace drops them.
            let after = quote! { Replay::consume(res); };
            // The call's id is the trace's, as the trace makes the call, in its step (see
            // below).
            let call = self.start_call(&func, args, quote! {}, after);
            quote! { #display_name => { #call } }
        });
        let drop_arm = self.replay_export_drop();
        self.generate_start(
            sig,
            quote! {
                while let Some(line) = proxy::recorder::replay::next_host_line() {
                    // A line in step `at` follows the step before it, and takes its own step, as
                    // an export call's arrival does.
                    if let Some(at) = line.at {
                        let step = proxy::conversion::executor::step_after(at - 1).await;
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
                }
            },
        )
    }
}
