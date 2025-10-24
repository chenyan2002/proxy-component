use anyhow::Result;
use std::path::Path;
use syn::{File, FnArg, Item, ItemFn, ItemMod, ItemTrait, PatType, parse_quote};

struct TraitInfo {
    module_path: Vec<String>,
    trait_item: ItemTrait,
}

pub fn analyze(file: &Path) -> Result<()> {
    let file = std::fs::read_to_string(file)?;
    let ast = syn::parse_file(&file)?;
    let traits = find_all_traits(&ast.items, vec![]);
    let impls = generate_impls(&traits);
    let output = prettyplease::unparse(&impls);
    println!("{output}");
    Ok(())
}

fn find_all_traits(items: &[Item], current_path: Vec<String>) -> Vec<TraitInfo> {
    let mut traits = Vec::new();
    for item in items {
        match item {
            Item::Trait(trait_item) => {
                traits.push(TraitInfo {
                    module_path: current_path.clone(),
                    trait_item: trait_item.clone(),
                });
            }
            Item::Mod(module) => {
                if let Some((_, ref mod_items)) = module.content {
                    let mut new_path = current_path.clone();
                    new_path.push(module.ident.to_string());
                    traits.extend(find_all_traits(mod_items, new_path));
                }
            }
            _ => {}
        }
    }
    traits
}

fn generate_impls(traits: &[TraitInfo]) -> File {
    let mut items = Vec::new();
    for trait_info in traits {
        let impl_with_methods = generate_impl_with_methods(trait_info);
        items.push(impl_with_methods);
    }
    File {
        shebang: None,
        attrs: vec![],
        items,
    }
}

fn generate_impl_with_methods(trait_info: &TraitInfo) -> Item {
    let trait_name = &trait_info.trait_item.ident;
    let full_path = if trait_info.module_path.is_empty() {
        trait_name.to_string()
    } else {
        format!("{}::{}", trait_info.module_path.join("::"), trait_name)
    };
    let trait_path: syn::Path = syn::parse_str(&full_path).unwrap();

    // Collect all method signatures from the trait
    let mut methods = Vec::new();

    for item in &trait_info.trait_item.items {
        if let syn::TraitItem::Fn(method) = item {
            let sig = &method.sig;
            let method_name = &sig.ident;

            // Generate a stub implementation
            let stub_impl: syn::ImplItemFn = parse_quote! {
                #sig {
                    unimplemented!()
                }
            };

            methods.push(syn::ImplItem::Fn(stub_impl));
        }
    }

    // Create the impl block
    parse_quote! {
        impl #trait_path for Stub {
            #(#methods)*
        }
    }
}

fn find_function<'a>(items: &'a [Item], path: &[&str]) -> Option<&'a ItemFn> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        for item in items {
            if let Item::Fn(func) = item {
                if func.sig.ident.to_string() == path[0] {
                    return Some(func);
                }
            }
        }
        return None;
    }
    for item in items {
        if let Item::Mod(module) = item {
            if module.ident == path[0] {
                if let Some((_, ref items)) = module.content {
                    return find_function(items, &path[1..]);
                } else {
                    return None;
                }
            }
        }
    }
    None
}
