use heck::ToKebabCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, Signature, Type, parse_quote, visit_mut::VisitMut};
pub struct FullTypePath<'a> {
    pub module_path: &'a [String],
}
pub struct ArgInfo {
    pub ident: Ident,
    pub is_borrowed: bool,
    pub ty: Type,
}
pub enum ResourceFuncKind {
    Method,
    Constructor,
}
impl ArgInfo {
    pub fn call_param(&self) -> syn::Expr {
        let ident = &self.ident;
        if self.is_borrowed {
            parse_quote! { &#ident }
        } else {
            parse_quote! { #ident }
        }
    }
}
pub fn make_path(module_path: &[String], name: &str) -> syn::Path {
    let path = format!("{}::{}", module_path.join("::"), name);
    syn::parse_str(&path).unwrap()
}
pub fn wit_func_name(
    module_path: &[String],
    resource: &Option<String>,
    func_name: &Ident,
    kind: &Option<ResourceFuncKind>,
) -> String {
    assert!(module_path.len() >= 3);
    let mut module_path = module_path.to_vec();
    if module_path[0] == "exports" {
        module_path.remove(0);
    }
    assert!(module_path.len() == 3);
    let mut res = String::new();
    match kind {
        Some(ResourceFuncKind::Constructor) => res.push_str("[constructor]"),
        Some(ResourceFuncKind::Method) => res.push_str("[method]"),
        _ => {}
    }
    res.push_str(&unwrapped_module(&module_path[0]).to_kebab_case());
    res.push(':');
    res.push_str(&module_path[1].to_kebab_case());
    res.push('/');
    res.push_str(&module_path[2].to_kebab_case());
    if let Some(name) = resource {
        res.push_str(&format!("/{}", name.to_kebab_case()));
    }
    res.push('.');
    res.push_str(&func_name.to_string().to_kebab_case());
    res
}
/// In virtualized components, constructors return `Self`, i.e. `MockedResource`, so type-directed
/// traits like `Dialog` or `Arbitrary` cannot recover the resource name. Returns the WIT name of
/// the resource to patch into the `MockedResource`, if `kind` is a constructor.
pub fn constructor_resource_name(
    resource: &Option<String>,
    kind: &Option<ResourceFuncKind>,
) -> Option<String> {
    match kind {
        Some(ResourceFuncKind::Constructor) => Some(resource.as_ref()?.to_kebab_case()),
        _ => None,
    }
}
pub fn get_return_type(ret: &syn::ReturnType) -> Option<Type> {
    match ret {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => match **ty {
            Type::Tuple(ref tuple) if tuple.elems.is_empty() => None,
            _ => Some(*ty.clone()),
        },
    }
}

pub fn extract_arg_info(sig: &Signature) -> (Option<ResourceFuncKind>, Vec<ArgInfo>) {
    let mut kind = None;
    let mut arg_infos = Vec::new();
    for arg in sig.inputs.iter() {
        match arg {
            FnArg::Receiver(_) => {
                kind = Some(ResourceFuncKind::Method);
            }
            FnArg::Typed(pat_type) => {
                let ident = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => ident.ident.clone(),
                    _ => unreachable!(),
                };
                let ty = *pat_type.ty.clone();
                let is_borrowed = matches!(&*pat_type.ty, Type::Reference(_));
                arg_infos.push(ArgInfo {
                    ident,
                    is_borrowed,
                    ty,
                });
            }
        }
    }
    if sig.ident == "new"
        && let Some(Type::Path(path)) = get_return_type(&sig.output)
        && path.path.is_ident("Self")
    {
        kind = Some(ResourceFuncKind::Constructor);
    }
    (kind, arg_infos)
}

const BUILTIN_TYPES: &[&str] = &[
    "Self", "Result", "Option", "Vec", "Box", "Rc", "Arc", "String", "str", "u8", "u16", "u32",
    "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64", "bool",
    "char", "_rt",
];
impl<'a> VisitMut for FullTypePath<'a> {
    fn visit_type_path_mut(&mut self, ty: &mut syn::TypePath) {
        if ty.qself.is_none() && !ty.path.segments.is_empty() && ty.path.leading_colon.is_none() {
            let ident = &ty.path.segments[0].ident.to_string();
            if BUILTIN_TYPES.contains(&ident.as_str()) {
                if ident == "_rt" {
                    assert!(ty.path.segments.len() == 2);
                    ty.path.segments = ty.path.segments.iter().skip(1).cloned().collect();
                }
                syn::visit_mut::visit_type_path_mut(self, ty);
                return;
            }
            let module_idents = self
                .module_path
                .iter()
                .map(|s| syn::parse_str::<syn::Ident>(s).unwrap());
            let original = &ty.path;
            *ty = parse_quote! {
                #(#module_idents)::*::#original
            };
        }
        syn::visit_mut::visit_type_path_mut(self, ty);
    }
}

/// WAVE string of `self`, the first param of a resource method.
// Use ToValue::to_value to avoid the auto-deref from self.to_value()
pub fn self_wave(kind: &Option<ResourceFuncKind>) -> Option<TokenStream> {
    matches!(kind, Some(ResourceFuncKind::Method))
        .then(|| quote! { wasm_wave::to_string(&ToValue::to_value(&self)).unwrap() })
}
/// Initial `Vec<String>` of WAVE params: `self` for methods, empty otherwise.
pub fn init_params(kind: &Option<ResourceFuncKind>) -> TokenStream {
    match self_wave(kind) {
        Some(self_wave) => quote! { vec![#self_wave] },
        None => quote! { Vec::new() },
    }
}
/// Binds `__params` to the WAVE strings of `self` (for methods) and each arg.
pub fn params_to_wave(kind: &Option<ResourceFuncKind>, args: &[ArgInfo]) -> TokenStream {
    let init = init_params(kind);
    let arg_names = args.iter().map(|arg| &arg.ident);
    quote! {
        let mut __params: Vec<String> = #init;
        #(
            __params.push(wasm_wave::to_string(&ToValue::to_value(&#arg_names)).unwrap());
        )*
    }
}

pub fn get_resource_from_trait_name(trait_name: &str) -> Option<String> {
    let resource = trait_name.strip_prefix("Guest").unwrap();
    match resource {
        "" => None,
        name => Some(name.to_string()),
    }
}

pub fn get_owned_type(ty: &Type) -> Option<Type> {
    match ty {
        Type::Reference(type_ref) => {
            match &*type_ref.elem {
                Type::Slice(type_slice) => {
                    let inner_ty = &*type_slice.elem;
                    Some(parse_quote! { Vec<#inner_ty> })
                }
                Type::Path(type_path) => {
                    if type_path.qself.is_none()
                        && type_path.path.segments.len() == 1
                        && type_path.path.segments[0].ident == "str"
                    {
                        Some(parse_quote! { String })
                    } else {
                        // TODO: need to handle nested borrow
                        Some(parse_quote! { #type_path })
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

// In record mode, the guest sees a copy of each imported package with its namespace
// prefixed by `wrapped-` in WIT (see `crate::wit_util::WRAPPED_WIT`), which becomes
// `wrapped_` in the generated Rust modules.
const WRAPPED_RUST: &str = "wrapped_";
pub fn is_wrapped_module(module: &str) -> bool {
    module.starts_with(WRAPPED_RUST)
}
/// Strips the `wrapped_` prefix from a Rust module name if present.
pub fn unwrapped_module(module: &str) -> &str {
    module.strip_prefix(WRAPPED_RUST).unwrap_or(module)
}
/// Maps a Rust module name between its host and wrapped copy.
pub fn toggle_wrapped_module(module: &str) -> String {
    match module.strip_prefix(WRAPPED_RUST) {
        Some(name) => name.to_string(),
        None => format!("{WRAPPED_RUST}{module}"),
    }
}
