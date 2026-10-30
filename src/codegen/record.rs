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
        let is_export = !is_wrapped_module(&module_path[1]);
        let record_ret = if get_return_type(&sig.output).is_none() {
            quote! {
                #func(#(#call_args),*);
                proxy::recorder::record::record_ret(Some(#display_name), None, #is_export);
            }
        } else {
            quote! {
               let res = #func(#(#call_args),*);
               let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
               proxy::recorder::record::record_ret(Some(#display_name), Some(&wave_res), #is_export);
               #res
            }
        };
        parse_quote! {
            #sig {
                let #is_mut params: Vec<String> = #init_vec;
                #(
                    let #arg_names = #arg_names.to_proxy();
                    params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                )*
                proxy::recorder::record::record_args(Some(#display_name), &params, #is_export);
                #record_ret
            }
        }
    }
    /// Overrides wit-bindgen's `resource_from_raw_`, which the `[dtor]` of an exported resource
    /// calls, to record the drop of the resource in `module_path` around the actual drop.
    pub fn generate_record_drop(&self, module_path: &[String]) -> syn::ImplItemFn {
        let is_export = !is_wrapped_module(&module_path[1]);
        let drop_name = trace::DROP;
        parse_quote! {
            unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
                // `None` if `into_inner` moved the resource out, e.g. to pass it on: not a drop.
                if let Some(res) = *unsafe { Box::from_raw(handle) } {
                    let params = vec![wasm_wave::to_string(&res.to_value()).unwrap()];
                    proxy::recorder::record::record_args(Some(#drop_name), &params, #is_export);
                    drop(res);
                    proxy::recorder::record::record_ret(Some(#drop_name), None, #is_export);
                }
                None
            }
        }
    }
}
