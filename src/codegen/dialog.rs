use super::util::{
    constructor_resource_name, extract_arg_info, get_return_type, params_to_wave, wit_func_name,
};
use super::{ExportFunc, State};
use quote::quote;
use syn::{Signature, parse_quote};

impl State {
    pub fn generate_dialog_import_func(
        &self,
        module_path: &[String],
        sig: &Signature,
        resource: &Option<String>,
    ) -> syn::ImplItemFn {
        let func_name = &sig.ident;
        let (kind, args) = extract_arg_info(sig);
        let display_name = wit_func_name(module_path, resource, func_name, &kind);
        let ret_ty = get_return_type(&sig.output);
        if let Some(ty) = ret_ty {
            let params = params_to_wave(&kind, &args);
            let (ret_name, read_ret) = match constructor_resource_name(resource, &kind) {
                Some(name) => (
                    quote! { #name },
                    quote! { MockedResource { name: #name.to_string(), ..Dialog::read_value(0) } },
                ),
                None => (
                    quote! { <#ty as WitName>::name() },
                    quote! { Dialog::read_value(0) },
                ),
            };
            parse_quote! {
                #sig {
                    #params
                    proxy::util::dialog::print(0, &format!("import: {}({})", #display_name, __params.join(", ")));
                    proxy::util::dialog::print(0, &format!("return type: {}", #ret_name));
                    let ret = #read_ret;
                    proxy::util::dialog::print(0, &format!("ret: {}", wasm_wave::to_string(&ToValue::to_value(&ret)).unwrap()));
                    ret
                }
            }
        } else {
            parse_quote! {
                #[allow(unused_variables)]
                #sig {}
            }
        }
    }
    pub fn generate_dialog_start_func(&self, sig: &Signature) -> syn::ImplItemFn {
        let funcs = self.export_funcs();
        let display_names = funcs.iter().map(|func| &func.display_name);
        let display_names = quote! { ["All done".to_string(), #(#display_names.to_string()),*] };
        let arms: Vec<_> = funcs
            .iter()
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
                        proxy::util::dialog::print(0, &format!("call export func {}", #display_name));
                        let mut __params: Vec<String> = Vec::new();
                        #(
                            proxy::util::dialog::print(0, &format!("provide argument for {}: {}", stringify!(#arg_names), <#arg_tys as WitName>::name()));
                            let #arg_names: #arg_tys = Dialog::read_value(0);
                            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                        )*
                        proxy::util::dialog::print(0, &format!("export: {}({})", #display_name, __params.join(", ")));
                        let _ = #func(#(#call_params),*);
                    }
                }
            })
            .collect();
        let func_len = arms.len();
        let idxs = 1..=func_len;
        parse_quote! {
          #sig {
            loop {
              let idx = proxy::util::dialog::read_select(0, "Select an export function to call", &#display_names) as usize;
              match idx {
                      0 => break,
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
