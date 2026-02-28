use super::State;
use crate::util::{
    FullTypePath, ResourceFuncKind, extract_arg_info, get_owned_type, get_return_type, make_path,
    wit_func_name,
};
use quote::quote;
use syn::{Signature, parse_quote, visit_mut::VisitMut};

impl State {
    pub fn generate_dialog_func(
        &self,
        module_path: &[String],
        sig: &Signature,
        resource: &Option<String>,
    ) -> syn::ImplItemFn {
        let func_name = &sig.ident;
        let is_export = module_path.join("::") == "exports::proxy::recorder::start_replay";
        if !is_export {
            let (kind, args) = extract_arg_info(sig);
            let arg_names = args.iter().map(|arg| &arg.ident);
            let display_name = wit_func_name(module_path, resource, func_name, &kind);
            let ret_ty = get_return_type(&sig.output);
            if ret_ty.is_some() {
                parse_quote! {
                    #sig {
                        let mut __params: Vec<String> = Vec::new();
                        #(
                            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                        )*
                        let mut __buf = __params.join(",");
                        proxy::util::dialog::prompt(&format!("import: {}({})", #display_name, __buf));
                        __buf += #display_name;
                        let mut u = Unstructured::new(&__buf.as_bytes());
                        let res = u.arbitrary().unwrap();
                        let res_str = wasm_wave::to_string(&ToValue::to_value(&res)).unwrap();
                        proxy::util::dialog::prompt(&format!("ret: {}", res_str));
                        res
                    }
                }
            } else {
                parse_quote! {
                    #sig {}
                }
            }
        } else {
            assert!(func_name == "start");
            let arms: Vec<_> = self
                .funcs
                .iter()
                .filter(|(path, _)| path[0] != "exports" && path[0] != "proxy")
                .flat_map(|(path, resources)| {
                    resources.iter().flat_map(move |(resource, sigs)| {
                        sigs.iter().filter_map(move |sig| {
                            let (kind, args) = extract_arg_info(sig);
                            if matches!(kind, Some(ResourceFuncKind::Method)) {
                                return None;
                            }
                            let arg_name: Vec<_> = args.iter().map(|arg| &arg.ident).collect();
                            let call_param = args.iter().map(|arg| arg.call_param());
                            let ty = args.iter().map(|arg| {
                                let mut ty = arg.ty.clone();
                                FullTypePath { module_path: path }.visit_type_mut(&mut ty);
                                if let Some(owned) = get_owned_type(&ty) {
                                    owned
                                } else {
                                    ty
                                }
                            });
                            let func_name = if let Some(resource) = resource {
                                format!("{}::{}", resource, sig.ident)
                            } else {
                                sig.ident.to_string()
                            };
                            let func = make_path(path, &func_name);
                            let display_name = wit_func_name(path, resource, &sig.ident, &kind);
                            Some(quote! {
                                {
                                    let mut __params: Vec<String> = Vec::new();
                                    #(
                                        let #arg_name: #ty = u.arbitrary().unwrap();
                                        __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_name)).unwrap());
                                    )*
                                    proxy::util::dialog::prompt(&format!("export: {}({})", #display_name, __params.join(", ")));
                                    let _ = #func(#(#call_param),*);
                                }
                            })
                        })
                    })
                })
                .collect();
            let func_len = arms.iter().len();
            let idxs = 1..=func_len;
            parse_quote! {
                #sig {
                    let __buf = (0..4096).map(|i| i.to_string()).collect::<Vec<_>>().join("").as_bytes().to_vec();
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
}
