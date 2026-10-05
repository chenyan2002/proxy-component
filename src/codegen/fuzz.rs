use super::util::{
    constructor_resource_name, extract_arg_info, get_return_type, params_to_wave, wit_func_name,
};
use super::{ExportFunc, State};
use quote::quote;
use syn::{Signature, parse_quote};

impl State {
    pub fn generate_fuzz_import_func(
        &self,
        module_path: &[String],
        sig: &Signature,
        resource: &Option<String>,
    ) -> syn::ImplItemFn {
        let func_name = &sig.ident;
        let (kind, args) = extract_arg_info(sig);
        let display_name = wit_func_name(module_path, resource, func_name, &kind);
        let ret_ty = get_return_type(&sig.output);
        if ret_ty.is_some() {
            let params = params_to_wave(&kind, &args);
            let gen_ret = match constructor_resource_name(resource, &kind) {
                Some(name) => {
                    quote! { MockedResource { name: #name.to_string(), ..u.arbitrary().unwrap() } }
                }
                None => quote! { u.arbitrary().unwrap() },
            };
            parse_quote! {
                #sig {
                    #params
                    let mut __buf = __params.join(",");
                    proxy::util::debug::print(&format!("import: {}({})", #display_name, __buf));
                    __buf += #display_name;
                    let mut u = Unstructured::new(&__buf.as_bytes());
                    let res = #gen_ret;
                    let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
                    proxy::util::debug::print(&format!("ret: {}", res_str));
                    res
                }
            }
        } else {
            parse_quote! {
                #[allow(unused_variables)]
                #sig {}
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
                    func,
                    arg_names,
                    arg_tys,
                    call_params,
                    ..
                } = func;
                quote! {
                    {
                        let mut __params: Vec<String> = Vec::new();
                        #(
                            let #arg_names: #arg_tys = u.arbitrary().unwrap();
                            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                        )*
                        proxy::util::debug::print(&format!("export: {}({})", #display_name, __params.join(", ")));
                        let _ = #func(#(#call_params),*);
                    }
                }
            })
            .collect();
        let func_len = arms.len();
        let idxs = 1..=func_len;
        parse_quote! {
            #sig {
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
            }
        }
    }
}
