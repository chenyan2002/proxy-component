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
            if let Some(ty) = ret_ty {
                let init_vec = if matches!(kind, Some(ResourceFuncKind::Method)) {
                    quote! { vec![wasm_wave::to_string(&ToValue::to_value(&self)).unwrap()] }
                } else {
                    quote! { Vec::new() }
                };
                parse_quote! {
                    #sig {
                        let mut __params: Vec<String> = #init_vec;
                        #(
                            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
                        )*
                        proxy::util::dialog::print(0, &format!("import: {}({})", #display_name, __params.join(", ")));
                        proxy::util::dialog::print(0, &format!("return type: {}", <#ty as WitName>::name()));
                        let ret = Dialog::read_value(0);
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
                            Some((quote! {
                                {
                                    proxy::util::dialog::print(0, &format!("call export func {}", #display_name));
                                    let mut __params: Vec<String> = Vec::new();
                                    #(
                                        proxy::util::dialog::print(0, &format!("provide argument for {}: {}", stringify!(#arg_name), <#ty as WitName>::name()));
                                        let #arg_name: #ty = Dialog::read_value(0);
                                        __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_name)).unwrap());
                                    )*
                                    proxy::util::dialog::print(0, &format!("export: {}({})", #display_name, __params.join(", ")));
                                    let _ = #func(#(#call_param),*);
                                }
                            }, display_name))
                        })
                    })
                })
                .collect();
            let (arms, display_names): (Vec<_>, Vec<_>) = arms.into_iter().unzip();
            let display_names =
                quote! { ["All done".to_string(), #(#display_names.to_string()),*] };
            let func_len = arms.iter().len();
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
}
