use super::util::{extract_arg_info, get_return_type, params_to_wave};
use super::{ExportFunc, State};
use quote::quote;
use syn::{Signature, parse_quote};

impl State {
    pub fn generate_dialog_import_func(
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
        if ret_ty.is_some() {
            let params = params_to_wave(kind.as_ref(), &args);
            let ret_name = wit_func.result.as_ref().unwrap();
            // A constructor's mock has no handle yet, so `_resource_new` prints its `ret`.
            let (read_ret, print_ret) = match wit_func.constructor_resource() {
                Some(name) => (
                    quote! { MockedResource { name: #name.to_string(), ..Dialog::read_value(0) } },
                    quote! { PENDING_RET.with(|pending| pending.set(true)); },
                ),
                None => (
                    quote! { Dialog::read_value(0) },
                    quote! {
                        proxy::util::dialog::print(0, &format!("ret: {}", wasm_wave::to_string(&ToValue::to_value(&ret)).unwrap()));
                    },
                ),
            };
            parse_quote! {
                #sig {
                    #params
                    proxy::util::dialog::print(0, &format!("import: {}({})", #display_name, __params.join(", ")));
                    proxy::util::dialog::print(0, &format!("return type: {}", #ret_name));
                    let ret = #read_ret;
                    #print_ret
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
                let call = func.call();
                let ExportFunc {
                    display_name,
                    arg_names,
                    arg_wit_tys,
                    arg_tys,
                    ..
                } = func;
                quote! {
                    {
                        proxy::util::dialog::print(0, &format!("call export func {}", #display_name));
                        let mut __params: Vec<String> = Vec::new();
                        #(
                            proxy::util::dialog::print(0, &format!("provide argument for {}: {}", stringify!(#arg_names), #arg_wit_tys));
                            let #arg_names: #arg_tys = Dialog::read_value(0);
                            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                        )*
                        proxy::util::dialog::print(0, &format!("export: {}({})", #display_name, __params.join(", ")));
                        let _ = #call;
                    }
                }
            })
            .collect();
        let func_len = arms.len();
        let idxs = 1..=func_len;
        self.generate_start(
            sig,
            quote! {
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
            },
        )
    }
}
