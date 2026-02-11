use crate::{Mode, codegen};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use wit_bindgen_core::{Files, wit_parser};
use wit_parser::{Resolve, WorldId};

#[derive(Parser)]
pub struct InstrumentArgs {
    /// The path to the wasm component file.
    wasm_file: PathBuf,
    /// Instrumentation mode
    #[arg(short, long)]
    mode: Mode,
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
    assert!(status.success());

    // 3. Parse the main wit file from tmp_dir/wit and feed into opts.generate_component
    let (resolve, world) = parse_wit(&wit_dir, None)?;
    let mut opts = crate::ast::Opt::new(args.mode.clone());
    opts.generate_wrapped_wits(&wit_dir)?;
    let mut files = Files::default();
    opts.generate_component(&resolve, world, &mut files)?;

    // 4. Write generated files to the temp directory.
    for (name, content) in files.iter() {
        let path = wit_dir.as_path().join(name);
        std::fs::write(&path, content)?;
    }
    // Re-generate exports world to bring in extra imports
    let (export_resolve, export_world) = parse_wit(&wit_dir, Some("tmp-exports"))?;
    opts.generate_exports_world(&export_resolve, export_world, &mut files);
    for (name, content) in files.iter() {
        let path = wit_dir.as_path().join(name);
        eprintln!("Generating: {}", path.display());
        std::fs::write(&path, content)?;
    }

    // 5. Generate Rust binding for both import and export interface
    bindgen(&tmp_dir, &wit_dir, &args.mode, "imports", "record_imports")?;
    bindgen(&tmp_dir, &wit_dir, &args.mode, "exports", "record_exports")?;
    // 6. cargo build
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .current_dir(tmp_dir.as_path());
    let status = cmd.status()?;
    assert!(status.success());

    let exports_wasm_path =
        component_new(&tmp_dir, &wit_dir, "exports", "debug/record_exports.wasm")?;
    let imports_wasm_path =
        component_new(&tmp_dir, &wit_dir, "imports", "debug/record_imports.wasm")?;
    // 7. run wac
    opts.generate_wac(&imports_wasm_path, &exports_wasm_path, &wit_dir)?;
    let output_file = "composed.wasm";
    let imports = format!("import:proxy={}", imports_wasm_path.display());
    let exports = format!("export:proxy={}", exports_wasm_path.display());
    let root = format!("root:component={}", args.wasm_file.display());
    let debug = format!(
        "import:debug={}/target/wasm32-wasip2/debug/debug.wasm",
        env!("CARGO_MANIFEST_DIR")
    );
    let wac_path = tmp_dir.join("wit/compose.wac");
    let status = Command::new("wac")
        .arg("compose")
        .arg("--dep")
        .arg(&imports)
        .arg("--dep")
        .arg(&exports)
        .arg("--dep")
        .arg(&debug)
        .arg("--dep")
        .arg(&root)
        .arg(&wac_path)
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
        .select_world(&[pkg], world)
        .context("Failed to select a world from the parsed wit files")?;
    Ok((resolve, world))
}
fn bindgen(
    tmp_dir: &Path,
    wit_dir: &Path,
    mode: &Mode,
    world_name: &str,
    dest_name: &str,
) -> Result<()> {
    let out_dir = tmp_dir.join(dest_name);
    let status = Command::new("wit-bindgen")
        .arg("rust")
        .arg(wit_dir)
        .arg("--world")
        .arg(world_name)
        .arg("--generate-all")
        //.arg("--merge-structurally-equal-types=true")
        .arg("--out-dir")
        .arg(&out_dir)
        .status()?;
    assert!(status.success());
    let binding_file = out_dir.join(world_name.to_owned() + ".rs");
    let codegen_mode = match mode {
        Mode::Record => codegen::GenerateMode::Record,
        Mode::Replay => codegen::GenerateMode::Replay,
        Mode::Fuzz => codegen::GenerateMode::Fuzz,
    };
    let codegen_opt = codegen::GenerateArgs {
        bindings: binding_file.clone(),
        output_file: out_dir.join("lib.rs"),
        mode: codegen_mode,
    };
    codegen_opt.generate()?;
    let status = Command::new("mv")
        .arg(&binding_file)
        .arg(out_dir.join("bindings.rs"))
        .status()?;
    assert!(status.success());
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
