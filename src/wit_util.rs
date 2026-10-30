use anyhow::{Context, Result};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use wit_component::WitPrinter;
use wit_parser::{Resolve, WorldId};

// In record mode, the guest sees a copy of each imported package with its namespace
// prefixed by `wrapped-` in WIT, which becomes `wrapped_` in the generated Rust modules
// (see `crate::codegen::util::is_wrapped_module`).
const WRAPPED_WIT: &str = "wrapped-";
/// `ns:pkg/iface` -> `wrapped-ns:pkg/iface`
pub fn wrapped_name(name: &str) -> String {
    format!("{WRAPPED_WIT}{name}")
}
/// `wrapped-ns:pkg/iface` -> `Some("ns:pkg/iface")`
pub fn strip_wrapped(name: &str) -> Option<&str> {
    name.strip_prefix(WRAPPED_WIT)
}

// utils for WIT names
pub fn ident(name: &str) -> Cow<'_, str> {
    if is_keyword(name) {
        Cow::Owned(format!("%{name}"))
    } else {
        Cow::Borrowed(name)
    }
}
// from https://docs.rs/wit-component/latest/src/wit_component/printing.rs.html#155-192
fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "use"
            | "type"
            | "func"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "s8"
            | "s16"
            | "s32"
            | "s64"
            | "f32"
            | "f64"
            | "float32"
            | "float64"
            | "char"
            | "resource"
            | "record"
            | "flags"
            | "variant"
            | "enum"
            | "bool"
            | "string"
            | "option"
            | "result"
            | "future"
            | "stream"
            | "list"
            | "own"
            | "borrow"
            | "_"
            | "as"
            | "from"
            | "static"
            | "interface"
            | "tuple"
            | "world"
            | "import"
            | "export"
            | "package"
            | "with"
            | "include"
            | "constructor"
            | "error-context"
            | "async"
    )
}

/// Extract WIT definitions from a wasm component and write them to `out_dir`.
/// Equivalent to `wasm-tools component wit <wasm_file> --out-dir <out_dir>`.
pub fn extract_wit(wasm_file: &Path, out_dir: &Path) -> Result<()> {
    let wasm = std::fs::read(wasm_file)
        .with_context(|| format!("failed to read {}", wasm_file.display()))?;
    let decoded = wit_component::decode(&wasm)
        .with_context(|| format!("failed to decode component {}", wasm_file.display()))?;
    let resolve = decoded.resolve();
    let main_pkg = decoded.package();

    std::fs::create_dir_all(out_dir)?;

    let mut names: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    for (_id, pkg) in resolve.packages.iter() {
        *names
            .entry(&pkg.name.name)
            .or_default()
            .entry(&pkg.name.namespace)
            .or_insert(0) += 1;
    }

    for (id, pkg) in resolve.packages.iter() {
        let is_main = id == main_pkg;
        let mut printer = WitPrinter::default();
        printer.print(resolve, id, &[])?;
        let output = printer.output.to_string();
        let dir = if is_main {
            out_dir.to_path_buf()
        } else {
            out_dir.join("deps")
        };
        let packages_with_same_name = &names[pkg.name.name.as_str()];
        let packages_with_same_namespace = packages_with_same_name[pkg.name.namespace.as_str()];
        let stem = if packages_with_same_name.len() == 1 {
            if packages_with_same_namespace == 1 {
                pkg.name.name.clone()
            } else {
                pkg.name
                    .version
                    .as_ref()
                    .map(|ver| format!("{}@{}", pkg.name.name, ver))
                    .unwrap_or_else(|| pkg.name.name.clone())
            }
        } else if packages_with_same_namespace == 1 {
            format!("{}:{}", pkg.name.namespace, pkg.name.name)
        } else {
            pkg.name.to_string()
        };
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{stem}.wit"));
        std::fs::write(&path, &output)?;
    }
    Ok(())
}

/// Generate Rust bindings for a world, returning the bindings file.
/// Equivalent to `wit-bindgen rust --generate-all --merge-structurally-equal-types=true`.
pub fn generate_bindings(resolve: &mut Resolve, world: WorldId) -> Result<String> {
    use wit_bindgen_core::{Files, WorldGenerator};
    let opts = wit_bindgen_rust::Opts {
        generate_all: true,
        merge_structurally_equal_types: Some(Some(true)),
        ..Default::default()
    };
    let mut generator = opts.build();
    let mut files = Files::default();
    generator.generate(resolve, world, &mut files)?;
    // A single file, `{world}.rs`
    let (_, content) = files
        .iter()
        .next()
        .context("wit-bindgen generated no file")?;
    Ok(String::from_utf8(content.to_vec())?)
}
