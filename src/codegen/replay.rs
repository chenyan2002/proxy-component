use super::State;
use crate::util::{
    FullTypePath, ResourceFuncKind, extract_arg_info, get_owned_type, get_return_type, make_path,
    wit_func_name,
};
use quote::quote;
use syn::{Signature, parse_quote, visit_mut::VisitMut};

impl State {
    pub fn generate_replay_func(
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
            let replay_import = if let Some(ret_ty) = ret_ty {
                quote! {
                    let wave = proxy::recorder::replay::replay_import(Some(#display_name), Some(&args)).unwrap();
                    let ret: Value = wasm_wave::from_str(&<#ret_ty as ValueTyped>::value_type(), &wave).unwrap();
                    ret.to_rust()
                }
            } else {
                quote! {
                    let wave = proxy::recorder::replay::replay_import(Some(#display_name), Some(&args));
                    assert!(wave.is_none());
                }
            };
            let self_value = if matches!(kind, Some(ResourceFuncKind::Method)) {
                // Use ToValue::to_value to avoid the auto-deref from self.to_value()
                quote! { wasm_wave::to_string(&ToValue::to_value(&self)).unwrap(), }
            } else {
                quote! {}
            };
            parse_quote! {
                #sig {
                    let args = vec![#self_value #( wasm_wave::to_string(&#arg_names.to_value()).unwrap() ),*];
                    #replay_import
                }
            }
        } else {
            assert!(func_name == "start");
            let arms = self
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
                            let arg_idx = args.iter().enumerate().map(|(idx, _)| quote! { args[#idx] });
                            let call_param = args.iter().map(|arg| arg.call_param());
                            let ty = args.iter().map(|arg| {
                                let mut ty = arg.ty.clone();
                                FullTypePath {
                                    module_path: path,
                                }.visit_type_mut(&mut ty);
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
                            let assert_ret = if get_return_type(&sig.output).is_none() {
                                quote! {
                                    assert!(res == ());
                                    proxy::recorder::replay::assert_export_ret(Some(#display_name), None);
                                }
                            } else {
                                quote! {
                                    let wave_res = wasm_wave::to_string(&res.to_value()).unwrap();
                                    proxy::recorder::replay::assert_export_ret(Some(#display_name), Some(&wave_res));
                                }
                            };
                            Some(quote! {
                                #display_name => {
                                    #(
                                        let arg_value: Value = wasm_wave::from_str(&<#ty as ValueTyped>::value_type(), &#arg_idx).unwrap();
                                        let #arg_name: #ty = arg_value.to_rust();
                                    )*
                                    let res = #func(#(#call_param),*);
                                    #assert_ret
                                }
                            })
                        })
                    })
                });
            parse_quote! {
                #sig {
                    while let Some((method, args)) = proxy::recorder::replay::replay_export() {
                        match method.as_str() {
                            #(#arms)*
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
