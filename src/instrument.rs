use crate::{Mode, codegen};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use wit_bindgen_core::{Files, wit_parser};
use wit_component::{ComponentEncoder, embed_component_metadata};
use wit_parser::{Resolve, WorldId};

#[derive(Parser)]
pub struct InstrumentArgs {
    /// The path to the wasm component file.
    pub wasm_file: PathBuf,
    /// Instrumentation mode
    #[arg(short, long)]
    pub mode: Mode,
    /// Whether to use the host recorder implementation or link the recorder component
    #[arg(long)]
    pub use_host_recorder: bool,
}

const DEBUG_WASM: &[u8] = include_bytes!("../assets/debug.wasm");
const RECORDER_WASM: &[u8] = include_bytes!("../assets/recorder.wasm");

pub fn run(args: InstrumentArgs) -> Result<()> {
    if args.use_host_recorder && !matches!(args.mode, Mode::Record | Mode::Replay) {
        anyhow::bail!("--use-host-recorder only works in record or replay mode");
    }
    // 1. Create a tmp directory and initialize a new Rust project in it.
    let tmp_dir = init_rust_project()?;
    let wit_dir = tmp_dir.join("wit");

    // 2. Extract WIT from the wasm component into {tmp_dir/wit}
    // TODO: no need to write to disk
    crate::util::extract_wit(&args.wasm_file, &wit_dir)?;

    // 3. Parse the main wit file from tmp_dir/wit and feed into opts.generate_component
    let (resolve, world) = parse_wit(&wit_dir, None)?;
    let mut opts = crate::ast::Opt::new(&args);
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
    fs::write(tmp_dir.join("debug.wasm"), DEBUG_WASM)?;
    let debug = format!("import:debug={}/debug.wasm", tmp_dir.display());
    let wac_path = tmp_dir.join("wit/compose.wac");
    let mut cmd = Command::new("wac");
    cmd.arg("compose")
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
        .arg(output_file);
    if !args.use_host_recorder {
        let wasm_path = tmp_dir.join("recorder.wasm");
        fs::write(&wasm_path, RECORDER_WASM)?;
        let recorder = format!("import:recorder={}", wasm_path.display());
        cmd.arg("--dep").arg(&recorder);
    }
    let status = cmd.status()?;
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
    crate::util::generate_bindings(wit_dir, world_name, &out_dir)?;
    let binding_file = out_dir.join(world_name.to_owned() + ".rs");
    let codegen_mode = match mode {
        Mode::Record => codegen::GenerateMode::Record,
        Mode::Replay => codegen::GenerateMode::Replay,
        Mode::Fuzz => codegen::GenerateMode::Fuzz,
        Mode::Dialog => codegen::GenerateMode::Dialog,
    };
    let codegen_opt = codegen::GenerateArgs {
        bindings: binding_file.clone(),
        output_file: out_dir.join("lib.rs"),
        mode: codegen_mode,
    };
    codegen_opt.generate()?;
    fs::rename(&binding_file, out_dir.join("bindings.rs"))?;
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
    // embed component type metadata
    let mut wasm = fs::read(&wasm_path)?;
    let (resolve, world_id) = parse_wit(wit_dir, Some(&world))?;
    embed_component_metadata(
        &mut wasm,
        &resolve,
        world_id,
        wit_component::StringEncoding::UTF8,
    )?;
    // create component from the embedded module
    let component = ComponentEncoder::default()
        .module(&wasm)?
        .encode()
        .context("failed to encode a component from module")?;
    fs::write(&wasm_path, component)?;
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
