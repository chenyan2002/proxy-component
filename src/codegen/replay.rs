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
        let args = extract_arg_info(sig);
        let ret_ty = get_return_type(&sig.output);
        let replay_import = if let Some(ret_ty) = ret_ty {
            let to_rust = match wit_func.constructor_resource() {
                Some(name) => {
                    quote! { MockedResource { name: #name.to_string(), ..ret.to_rust() } }
                }
                None => quote! { ret.to_rust() },
            };
            quote! {
                let wave = proxy::recorder::replay::replay_import(Some(#display_name), Some(&__params)).unwrap();
                let ret: Value = wasm_wave::from_str(&<#ret_ty as ValueTyped>::value_type(), &wave).unwrap();
                #to_rust
            }
        } else {
            quote! {
                let wave = proxy::recorder::replay::replay_import(Some(#display_name), Some(&__params));
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
    /// Overrides wit-bindgen's `resource_from_raw_`, which the `[dtor]` of a mocked resource
    /// calls, to check the drop with the trace.
    pub fn generate_replay_drop(&self) -> syn::ImplItemFn {
        let drop_name = trace::DROP;
        parse_quote! {
            unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
                // `None` if `into_inner` moved the resource out: not a drop.
                if let Some(res) = *unsafe { Box::from_raw(handle) } {
                    if !MOVING.with(|moving| moving.get()) {
                        let params = vec![wasm_wave::to_string(&res.to_value()).unwrap()];
                        let ret = proxy::recorder::replay::replay_import(Some(#drop_name), Some(&params));
                        assert!(ret.is_none());
                    }
                }
                None
            }
        }
    }
    /// Items the replay functions use.
    pub fn replay_defs(&self) -> Vec<Item> {
        // `MOVING` is set while an import drops the resources its arguments own, which the guest
        // moved to the host, so that `generate_replay_drop` doesn't check them as drops.
        //
        // Only replay needs it. In record mode, the import passes the host resource of each
        // argument on to the host, taking it out with `into_inner` (see `ToProxy`), so the
        // destructor finds no resource to record. In replay there's no host to take them: the
        // import drops its arguments, and the destructors find their mocked resources. A flag
        // spares replay a type-directed trait like `ToProxy`, to take the resources out of nested
        // arguments, e.g. `list<counter>`. A thread-local, as `[dtor]`s take no other arguments.
        let file: syn::File = parse_quote! {
            thread_local! {
                #[allow(dead_code)]
                static MOVING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
            }
        };
        file.items
    }
    /// The arm of the `start` function dropping a resource of the main component, which the
    /// trace labels by its handle in this component, e.g. `#session-1`.
    fn replay_export_drop(&self) -> Option<proc_macro2::TokenStream> {
        let drop_name = trace::DROP;
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
            #drop_name => {
                let (name, handle) = args[0]
                    .strip_prefix('#')
                    .and_then(|label| label.rsplit_once('-'))
                    .and_then(|(name, handle)| Some((name, handle.parse::<u32>().ok()?)))
                    .expect("invalid handle label");
                match name {
                    #(#arms)*
                    _ => unreachable!(),
                }
                proxy::recorder::replay::assert_export_ret(Some(#drop_name), None);
            }
        })
    }
    pub fn generate_replay_start_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let arms = self.export_funcs().into_iter().map(|func| {
            let call = func.call();
            let ExportFunc {
                display_name,
                arg_names,
                arg_tys,
                has_ret,
                ret_owns,
                ..
            } = func;
            let arg_idx = (0..arg_names.len()).map(|idx| quote! { args[#idx] });
            let assert_ret = if has_ret {
                quote! {
                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::assert_export_ret(Some(#display_name), Some(&wave_res));
                }
            } else {
                quote! {
                    assert!(res == ());
                    proxy::recorder::replay::assert_export_ret(Some(#display_name), None);
                }
            };
            // The host keeps the resources it gets, until the trace drops them.
            let keep_ret = if ret_owns {
                quote! { std::mem::forget(res); }
            } else {
                quote! {}
            };
            quote! {
                #display_name => {
                    #(
                        let arg_value: Value = wasm_wave::from_str(&<#arg_tys as ValueTyped>::value_type(), &#arg_idx).unwrap();
                        let #arg_names: #arg_tys = arg_value.to_rust();
                    )*
                    let res = #call;
                    #assert_ret
                    #keep_ret
                }
            }
        });
        let drop_arm = self.replay_export_drop();
        self.generate_start(
            sig,
            quote! {
                while let Some((method, args)) = proxy::recorder::replay::replay_export() {
                    match method.as_str() {
                        #(#arms)*
                        #drop_arm
                        _ => unreachable!(),
                    }
                    // clean up borrowed resources from input args
                    SCOPED_ALLOC.with(|alloc| {
                        alloc.borrow_mut().clear();
                    });
                }
            },
        )
    }
}
