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
        let consumed: Vec<_> = args
            .iter()
            .zip(&wit_func.params_async)
            .filter(|(arg, is_async)| **is_async && !arg.is_borrowed)
            .map(|(arg, _)| &arg.ident)
            .collect();
        let is_async = wit_func.is_async;
        if ret_ty.is_none() && consumed.is_empty() && !is_async {
            return parse_quote! {
                #[allow(unused_variables)]
                #sig {}
            };
        }
        let params = params_to_wave(kind.as_ref(), &args);
        // What the host does, rather than returns: how it reads the streams and futures, and when
        // an async import returns, at once (RETURNED), or after the guest and the host's
        // background work ran a while (STARTED). Random, rather than drawn from the text of the
        // arguments, whose first byte is mostly the `#` of a label.
        let host = if consumed.is_empty() && !is_async {
            quote! {}
        } else {
            let delay = if is_async {
                quote! {
                    for _ in 0..__host.int_in_range(0..=3u8).unwrap_or(0) {
                        wit_bindgen::yield_async().await;
                    }
                }
            } else {
                quote! {}
            };
            quote! {
                let __random = proxy::util::debug::get_random();
                let mut __host = Unstructured::new(&__random);
                #(Fuzz::consume(#consumed, &mut __host);)*
                #delay
            }
        };
        let preamble = quote! {
            #params
            proxy::util::debug::print(&format!("import: {}({})", #display_name, __params.join(",")));
            #host
        };
        if ret_ty.is_some() {
            // A constructor's mock has no handle yet, so `_resource_new` prints its `ret`.
            let (gen_ret, print_ret) = match wit_func.constructor_resource() {
                Some(name) => (
                    quote! { MockedResource { name: #name.to_string(), ..Fuzz::fuzz(&mut u).unwrap() } },
                    quote! { PENDING_RET.with(|pending| pending.set(true)); },
                ),
                None => (
                    quote! { Fuzz::fuzz(&mut u).unwrap() },
                    quote! {
                        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
                        proxy::util::debug::print(&format!("ret: {}", res_str));
                    },
                ),
            };
            parse_quote! {
                #sig {
                    #preamble
                    let __buf = __params.join(",") + #display_name;
                    let mut u = Unstructured::new(&__buf.as_bytes());
                    let res = #gen_ret;
                    #print_ret
                    res
                }
            }
        } else {
            parse_quote! {
                #sig {
                    #preamble
                }
            }
        }
    }
    pub fn generate_fuzz_start_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let arms: Vec<_> = self
            .export_funcs()
            .into_iter()
            .map(|func| {
                let call = func.call();
                let ExportFunc {
                    display_name,
                    arg_names,
                    arg_tys,
                    ret_async,
                    ..
                } = func;
                // The streams and futures the guest returns, which the host reads or drops.
                let ret = if ret_async {
                    quote! { Fuzz::consume(#call, &mut u); }
                } else {
                    quote! { let _ = #call; }
                };
                quote! {
                    {
                        let mut __params: Vec<String> = Vec::new();
                        #(
                            let #arg_names: #arg_tys = Fuzz::fuzz(&mut u).unwrap();
                            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                        )*
                        proxy::util::debug::print(&format!("export: {}({})", #display_name, __params.join(", ")));
                        #ret
                        // Lets the guest, and the host's background work, run between calls.
                        for _ in 0..u.int_in_range(0..=3u8).unwrap() {
                            wit_bindgen::yield_async().await;
                        }
                    }
                }
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
                    // clean up borrowed resources from input args
                    SCOPED_ALLOC.with(|alloc| {
                        alloc.borrow_mut().clear();
                    });
                }
            },
        )
    }
}
