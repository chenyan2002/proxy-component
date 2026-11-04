use super::util::{
    ResourceFuncKind, extract_arg_info, get_return_type, init_params, is_wrapped_module,
};
use super::{State, get_proxy_path};
use quote::quote;
use syn::{Signature, parse_quote};

impl State {
    pub fn generate_record_func(
        &self,
        module_path: &[String],
        sig: &Signature,
        resource: Option<&str>,
    ) -> syn::ImplItemFn {
        let func_name = &sig.ident;
        let wit_func = self.expect_wit_func(module_path, resource, func_name);
        let kind = wit_func.kind;
        let args = extract_arg_info(sig);
        let import_path = get_proxy_path(module_path);
        let import_sig = self
            .find_function(&import_path, resource, func_name)
            .unwrap();
        let import_args = extract_arg_info(import_sig);
        let arg_names = args.iter().map(|arg| &arg.ident);
        let call_args = args
            .iter()
            .zip(import_args.iter())
            .map(|(arg, import_arg)| -> syn::Expr {
                let ident = &arg.ident;
                if import_arg.is_borrowed {
                    parse_quote! { &#ident }
                } else {
                    parse_quote! { #ident }
                }
            });
        let (func, res): (syn::Expr, _) = match (resource.is_some(), &kind) {
            (true, Some(ResourceFuncKind::Method)) => {
                (parse_quote! { self.#func_name }, quote! { res.to_proxy() })
            }
            (true, Some(ResourceFuncKind::Constructor)) => {
                (parse_quote! { Self::#func_name }, quote! { res })
            }
            (true, None) => (parse_quote! { Self::#func_name }, quote! { res.to_proxy() }),
            (false, _) => (
                syn::parse_str(&format!("{}::{}", import_path.join("::"), func_name)).unwrap(),
                quote! { res.to_proxy() },
            ),
        };
        let init_vec = init_params(kind.as_ref());
        let is_mut = if args.is_empty() {
            quote! {}
        } else {
            quote! { mut }
        };
        let display_name = &wit_func.display_name;
        let is_async = wit_func.is_async;
        // The host calls the exports, which the exports component numbers, and the guest the
        // imports, which the imports component numbers.
        let (table, caller, callee) = if is_wrapped_module(&module_path[1]) {
            (quote! { Imports }, quote! { Guest }, quote! { Host })
        } else {
            (quote! { Exports }, quote! { Host }, quote! { Guest })
        };
        let record_ret = if get_return_type(&sig.output).is_none() {
            quote! {
                #func(#(#call_args),*);
                proxy::recorder::record::record(&trace_return(Party::#callee, __id, None));
            }
        } else {
            quote! {
               let res = #func(#(#call_args),*);
               let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
               proxy::recorder::record::record(&trace_return(Party::#callee, __id, Some(wave_res)));
               #res
            }
        };
        parse_quote! {
            #sig {
                let __id = trace_task_id(Table::#table);
                let #is_mut params: Vec<String> = #init_vec;
                #(
                    let #arg_names = #arg_names.to_proxy();
                    params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                )*
                proxy::recorder::record::record(&trace_call(Party::#caller, __id, #display_name, #is_async, params));
                #record_ret
            }
        }
    }
    /// Overrides wit-bindgen's `resource_from_raw_`, which the `[dtor]` of an exported resource
    /// calls, to record the drop of the resource in `module_path` before the actual drop: by the
    /// guest, of a host resource the imports component wraps, or by the host, of a resource of
    /// the guest, which the exports component wraps.
    pub fn generate_record_drop(&self, module_path: &[String]) -> syn::ImplItemFn {
        let (by, table) = if is_wrapped_module(&module_path[1]) {
            (quote! { Guest }, quote! { Imports })
        } else {
            (quote! { Host }, quote! { Exports })
        };
        parse_quote! {
            unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
                // `None` if `into_inner` moved the resource out, e.g. to pass it on: not a drop.
                if let Some(res) = *unsafe { Box::from_raw(handle) } {
                    let name = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::record::record(&trace_drop(Party::#by, Table::#table, name));
                    drop(res);
                }
                None
            }
        }
    }
}
