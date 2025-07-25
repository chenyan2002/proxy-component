use anyhow::{Context, Result, bail};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use wit_bindgen_core::{Files, wit_parser};
use wit_bindgen_rust::Opts;
use wit_parser::{Resolve, WorldId};

#[derive(Parser)]
pub struct InstrumentArgs {
    /// The path to the wasm component file.
    #[arg()]
    wasm_file: PathBuf,
}

pub fn run(args: InstrumentArgs) -> Result<()> {
    // 1. Create a tmp directory and initialize a new Rust project in it.
    let tmp_dir = init_rust_project()?;
    let wit_dir = tmp_dir.join("wit");

    // 2. run `wasm-tools component wit {wasm_file from CLI} --out-dir {tmp_dir/wit}`
    let status = Command::new("wasm-tools")
        .arg("component")
        .arg("wit")
        .arg(&args.wasm_file)
        .arg("--out-dir")
        .arg(&wit_dir)
        .status()
        .context("Failed to execute wasm-tools. Is it installed and in your PATH?")?;

    if !status.success() {
        bail!("wasm-tools component wit failed with exit code: {}", status);
    }

    // 3. Parse the main wit file from tmp_dir/wit and feed into opts.generate_component
    let (resolve, world) = parse_wit(&wit_dir, None)?;
    let opts = crate::ast::Opt::new();
    let mut files = Files::default();
    opts.generate_component(&resolve, world, &mut files)?;

    // 4. Write generated files to the temp directory.
    for (name, content) in files.iter() {
        let path = wit_dir.as_path().join(name);
        eprintln!("Generating: {}", path.display());
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write generated file to {}", path.display()))?;
    }

    // 5. Generate Rust binding for both import and export interface
    bindgen(&tmp_dir, &wit_dir, "imports", "record_imports/lib.rs")?;
    bindgen(&tmp_dir, &wit_dir, "exports", "record_exports/lib.rs")?;

    // 6. cargo build
    let status = Command::new("cargo")
        .arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .current_dir(tmp_dir.as_path())
        .status()
        .context(
            "Failed to execute cargo build. Is rustup target `wasm32-unknown-unknown` installed?",
        )?;
    if !status.success() {
        bail!("cargo build failed with exit code: {}", status);
    }
    let imports_wasm_path =
        component_new(&tmp_dir, &wit_dir, "imports", "debug/record_imports.wasm")?;
    let exports_wasm_path =
        component_new(&tmp_dir, &wit_dir, "exports", "debug/record_exports.wasm")?;

    // 7. run wac
    let output_file = "composed.wasm";
    /*
    let status = Command::new("wac")
        .arg("plug")
        .arg(&args.wasm_file)
        .arg("--plug")
        .arg(&imports_wasm_path)
        .arg("-o")
        .arg(&output_file)
        .status()?;
    assert!(status.success());
    let status = Command::new("wac")
        .arg("plug")
        .arg(&exports_wasm_path)
        .arg("--plug")
    //.arg(&args.wasm_file)
        .arg(&output_file)
        .arg("-o")
        .arg(&output_file)
        .status()?;
    assert!(status.success());*/

    let imports = format!("import:proxy={}", imports_wasm_path.display());
    let exports = format!("export:proxy={}", exports_wasm_path.display());
    let root = format!("root:component={}", args.wasm_file.display());
    let status = Command::new("wac")
        .arg("compose")
        .arg("--dep")
        .arg(&imports)
        .arg("--dep")
        .arg(&exports)
        .arg("--dep")
        .arg(&root)
        .arg(tmp_dir.join("compose.wac"))
        .arg("-o")
        .arg(output_file)
        .status()?;
    assert!(status.success());
    eprintln!("Generated component: {output_file}");

    Ok(())
}

fn parse_wit(dir: &Path, world: Option<&str>) -> Result<(Resolve, WorldId)> {
    let mut resolve = Resolve::default();
    let (pkg, _files) = resolve
        .push_dir(dir)
        .with_context(|| format!("Failed to parse wit files in {}", dir.display()))?;

    let world = resolve
        .select_world(pkg, world)
        .context("Failed to select a world from the parsed wit files")?;
    Ok((resolve, world))
}
fn bindgen(tmp_dir: &Path, wit_dir: &Path, world_name: &str, dest_name: &str) -> Result<()> {
    // We could use `generate!` to make code simpler, but it increases build time.
    let mut opts = Opts::default();
    opts.proxy_component = true;
    opts.stubs = true;
    opts.runtime_path = Some("wit_bindgen_rt".to_owned());
    opts.generate_all = true;
    opts.format = true;
    let mut generator = opts.build();
    let mut files = Files::default();
    let (resolve, world) = parse_wit(wit_dir, Some(world_name))?;
    generator.generate(&resolve, world, &mut files)?;
    for (name, content) in files.iter() {
        assert!(name.starts_with(world_name));
        let path = tmp_dir.join(dest_name);
        eprintln!("Generating: {}", path.display());
        fs::write(&path, content)?;
    }
    Ok(())
}
fn component_new(
    tmp_dir: &Path,
    wit_dir: &Path,
    world_name: &str,
    wasm_file: &str,
) -> Result<PathBuf> {
    let wasm_path = tmp_dir
        .join("target/wasm32-unknown-unknown/")
        .join(wasm_file);
    let world = "component:proxy/".to_string() + world_name;
    let status = Command::new("wasm-tools")
        .arg("component")
        .arg("embed")
        .arg(wit_dir)
        .arg(&wasm_path)
        .arg("-o")
        .arg(&wasm_path)
        .arg("--world")
        .arg(&world)
        .status()?;
    assert!(status.success());
    let status = Command::new("wasm-tools")
        .arg("component")
        .arg("new")
        .arg(&wasm_path)
        .arg("-o")
        .arg(&wasm_path)
        .status()?;
    assert!(status.success());
    Ok(wasm_path)
}
fn init_rust_project() -> Result<PathBuf> {
    /*let tmp_dir = tempfile::Builder::new()
    .prefix("proxy-component-")
    .disable_cleanup(true)
    .tempdir_in("/tmp")?;*/
    let tmp_dir = PathBuf::from("/tmp/proxy-component");
    // create tmp_dir, if exists, empty the dir
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir_all(&tmp_dir)?;
    fs::write(
        tmp_dir.join("Cargo.toml"),
        include_str!("../assets/workspace_cargo.toml"),
    )?;
    fs::write(
        tmp_dir.join("compose.wac"),
        include_str!("../assets/compose.wac"),
    )?;

    let wit_dir = tmp_dir.join("wit");
    let import_src_dir = tmp_dir.join("record_imports");
    let export_src_dir = tmp_dir.join("record_exports");
    fs::create_dir_all(&wit_dir)?;
    fs::create_dir_all(&import_src_dir)?;
    fs::create_dir_all(&export_src_dir)?;
    let toml = include_str!("../assets/proj_cargo.toml");
    fs::write(
        import_src_dir.join("Cargo.toml"),
        toml.replace("{proj_name}", "record_imports"),
    )?;
    fs::write(
        export_src_dir.join("Cargo.toml"),
        toml.replace("{proj_name}", "record_exports"),
    )?;
    Ok(tmp_dir)
}
