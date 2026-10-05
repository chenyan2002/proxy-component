use super::util::{
    constructor_resource_name, extract_arg_info, get_return_type, self_wave, wit_func_name,
};
use super::{ExportFunc, State};
use quote::quote;
use syn::{Signature, parse_quote};

impl State {
    pub fn generate_replay_import_func(
        &self,
        module_path: &[String],
        sig: &Signature,
        resource: &Option<String>,
    ) -> syn::ImplItemFn {
        let func_name = &sig.ident;
        let (kind, args) = extract_arg_info(sig);
        let arg_names = args.iter().map(|arg| &arg.ident);
        let display_name = wit_func_name(module_path, resource, func_name, &kind);
        let ret_ty = get_return_type(&sig.output);
        let replay_import = if let Some(ret_ty) = ret_ty {
            let to_rust = match constructor_resource_name(resource, &kind) {
                Some(name) => {
                    quote! { MockedResource { name: #name.to_string(), ..ret.to_rust() } }
                }
                None => quote! { ret.to_rust() },
            };
            quote! {
                let wave = proxy::recorder::replay::replay_import(Some(#display_name), Some(&args)).unwrap();
                let ret: Value = wasm_wave::from_str(&<#ret_ty as ValueTyped>::value_type(), &wave).unwrap();
                #to_rust
            }
        } else {
            quote! {
                let wave = proxy::recorder::replay::replay_import(Some(#display_name), Some(&args));
                assert!(wave.is_none());
            }
        };
        let self_value = self_wave(&kind).map(|self_wave| quote! { #self_wave, });
        parse_quote! {
            #sig {
                let args = vec![#self_value #( wasm_wave::to_string(&#arg_names.to_value()).unwrap() ),*];
                #replay_import
            }
        }
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
            quote! {
                #display_name => {
                    #(
                        let arg_value: Value = wasm_wave::from_str(&<#arg_tys as ValueTyped>::value_type(), &#arg_idx).unwrap();
                        let #arg_names: #arg_tys = arg_value.to_rust();
                    )*
                    let res = #func(#(#call_params),*);
                    #assert_ret
                }
            }
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
