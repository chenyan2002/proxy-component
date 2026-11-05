use super::util::{extract_arg_info, get_return_type, params_to_wave};
use super::{ExportFunc, State};
use quote::quote;
use syn::{Signature, parse_quote};

impl State {
    pub fn generate_fuzz_import_func(
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
        // The streams and futures the guest gives the host, which the host reads or drops.
        let consumed = wit_func.consumed_args(&args);
        // The other resources the arguments own, which the guest moved to the host.
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
        let is_async = wit_func.is_async;
        let params = params_to_wave(kind.as_ref(), &args);
        // What the host does, rather than returns: how it reads the streams and futures, and when
        // an async import returns, at once (RETURNED), or after the executor ran a few times
        // (STARTED), in whose last run the `return` happens. Random, rather than drawn from the
        // text of the arguments, whose first byte is mostly the `#` of a label.
        let delay = if is_async {
            quote! {
                let __runs = __host.int_in_range(0..=3u8).unwrap_or(0);
                let __at = if __runs > 0 {
                    proxy::recorder::record::record(&Line { by: Party::Host, at: None, event: Event::Started(__id) });
                    Some(wait_for_runs(__runs as u32).await)
                } else {
                    None
                };
            }
        } else {
            quote! { let __at: Option<u32> = None; }
        };
        let preamble = quote! {
            #params
            let __id = trace_task_id(Table::Imports);
            proxy::recorder::record::record(&trace_call(Party::Guest, __id, #display_name, #is_async, __params.clone()));
            let __random = proxy::util::debug::get_random();
            let mut __host = Unstructured::new(&__random);
            // Dropping what the guest moved to the host isn't a drop of the guest's, so it's
            // done before the call can wait, while no other task runs.
            MOVING.with(|moving| moving.set(true));
            #(Fuzz::consume(#consumed, &mut __host);)*
            #drop_owned
            MOVING.with(|moving| moving.set(false));
            #delay
        };
        let record_ret = |value: proc_macro2::TokenStream| {
            quote! {
                proxy::recorder::record::record(&Line { at: __at, ..trace_return(Party::Host, __id, #value) });
            }
        };
        if ret_ty.is_some() {
            let gen_ret = match wit_func.constructor_resource() {
                Some(name) => {
                    quote! { MockedResource { name: #name.to_string(), ..Fuzz::fuzz(&mut u).unwrap() } }
                }
                None => quote! { Fuzz::fuzz(&mut u).unwrap() },
            };
            let record_ret = record_ret(quote! { Some(res_str) });
            let ret = quote! {
                let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
                #record_ret
            };
            parse_quote! {
                #[allow(unused_variables)]
                #sig {
                    #preamble
                    let __buf = __params.join(",") + #display_name;
                    let mut u = Unstructured::new(&__buf.as_bytes());
                    let res = #gen_ret;
                    #ret
                    res
                }
            }
        } else {
            let record_ret = record_ret(quote! { None });
            parse_quote! {
                #[allow(unused_variables)]
                #sig {
                    #preamble
                    #record_ret
                }
            }
        }
    }
    pub fn generate_fuzz_start_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let arms: Vec<_> = self
            .export_funcs()
            .into_iter()
            .map(|func| {
                let ExportFunc {
                    display_name,
                    arg_names,
                    arg_tys,
                    is_async,
                    ..
                } = &func;
                let args = quote! {
                    let mut __params: Vec<String> = Vec::new();
                    #(
                        let #arg_names: #arg_tys = Fuzz::fuzz(&mut u).unwrap();
                        __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                    )*
                };
                // The guest, and the host's background work, run a few steps before the call,
                // as they do before replay's, in the steps the trace has.
                let before = quote! {
                    let __id = trace_task_id(Table::Exports);
                    let __runs = u.int_in_range(0..=3u8).unwrap() as u32;
                    let __at = arrive_after(proxy::conversion::executor::current_step() + __runs).await;
                    proxy::recorder::record::record(&Line {
                        at: Some(__at),
                        ..trace_call(Party::Host, __id, #display_name, #is_async, __params)
                    });
                };
                // The streams and futures the guest returns, which the host reads or drops, and
                // its resources, which the host drops.
                let after = quote! { Fuzz::consume(res, &mut u); };
                let call = self.start_call(&func, args, before, after);
                quote! { { #call } }
            })
            .collect();
        let func_len = arms.len();
        let idxs = 1..=func_len;
        self.generate_start(
            sig,
            quote! {
                let __buf = proxy::util::debug::get_random();
                let mut u = Unstructured::new(&__buf);
                for _ in 0..10 {
                    let idx = u.int_in_range(1..=#func_len).unwrap();
                    match idx {
                        #(#idxs => #arms)*
                        _ => unreachable!(),
                    }
                }
            },
        )
    }
}
