use anyhow::Result;
use quote::ToTokens;
use std::path::Path;
use syn::{File, FnArg, Item, ItemFn, ItemMod, PatType};

pub fn analyze(file: &Path, func_path: &str) -> Result<()> {
    let file = std::fs::read_to_string(file)?;
    let ast = syn::parse_file(&file)?;
    let parts: Vec<_> = func_path.split("::").collect();
    find_function(&ast.items, &parts).unwrap();
    Ok(())
}

fn find_function<'a>(items: &'a [Item], path: &[&str]) -> Option<&'a ItemFn> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        for item in items {
            if let Item::Fn(func) = item {
                println!("{}", func.sig.to_token_stream());
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
