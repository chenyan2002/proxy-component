use crate::{Mode, codegen};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use wit_bindgen_core::Files;
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
    /// Use 1-byte wasm pages for the generated and bundled components
    #[arg(long)]
    pub one_byte_page_size: bool,
}

const DEBUG_WASM: &[u8] = include_bytes!("../assets/debug.wasm");
const RECORDER_WASM: &[u8] = include_bytes!("../assets/recorder.wasm");
// Built with `--features one-byte-page` (see Makefile `build-components`).
const DEBUG_WASM_ONE_BYTE_PAGE: &[u8] = include_bytes!("../assets/debug.one_byte_page.wasm");
const RECORDER_WASM_ONE_BYTE_PAGE: &[u8] = include_bytes!("../assets/recorder.one_byte_page.wasm");

pub fn run(args: InstrumentArgs) -> Result<()> {
    if args.use_host_recorder && !matches!(args.mode, Mode::Record | Mode::Replay) {
        anyhow::bail!("--use-host-recorder only works in record or replay mode");
    }
    // 1. Create a tmp directory and initialize a new Rust project in it.
    let tmp_dir = init_rust_project()?;
    let wit_dir = tmp_dir.join("wit");

    // 2. Extract WIT from the wasm component into {tmp_dir/wit}
    // TODO: no need to write to disk
    crate::wit_util::extract_wit(&args.wasm_file, &wit_dir)?;

    // 3-5. Generate the WIT and Rust sources of the proxy components.
    let mut opts = generate_sources(&args, &tmp_dir)?;
    // 6. cargo build
    let mut cmd = Command::new("cargo");
    if args.one_byte_page_size {
        cmd.env("RUSTFLAGS", "-C link-arg=--page-size=1");
    }
    cmd.arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--release")
        .current_dir(tmp_dir.as_path());
    let status = cmd.status()?;
    assert!(status.success());

    let exports_wasm_path =
        component_new(&tmp_dir, &wit_dir, "exports", "release/record_exports.wasm")?;
    let imports_wasm_path =
        component_new(&tmp_dir, &wit_dir, "imports", "release/record_imports.wasm")?;
    // 7. run wac
    opts.generate_wac(&imports_wasm_path, &exports_wasm_path, &wit_dir)?;
    let output_file = "composed.wasm";
    let imports = format!("import:proxy={}", imports_wasm_path.display());
    let exports = format!("export:proxy={}", exports_wasm_path.display());
    let root = format!("root:component={}", args.wasm_file.display());
    let (debug_wasm, recorder_wasm) = if args.one_byte_page_size {
        (DEBUG_WASM_ONE_BYTE_PAGE, RECORDER_WASM_ONE_BYTE_PAGE)
    } else {
        (DEBUG_WASM, RECORDER_WASM)
    };
    fs::write(tmp_dir.join("debug.wasm"), debug_wasm)?;
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
        fs::write(&wasm_path, recorder_wasm)?;
        let recorder = format!("import:recorder={}", wasm_path.display());
        cmd.arg("--dep").arg(&recorder);
    }
    let status = cmd.status()?;
    assert!(status.success());
    eprintln!("Generated component: {output_file}");
    Ok(())
}

/// Generates the WIT of the proxy components into `dir/wit`, next to the main component's
/// WIT, and their Rust sources into `dir/record_imports` and `dir/record_exports`.
pub fn generate_sources<'a>(args: &'a InstrumentArgs, dir: &Path) -> Result<crate::ast::Opt<'a>> {
    let wit_dir = dir.join("wit");
    // 3. Parse the main wit file from dir/wit and feed into opts.generate_component
    let (resolve, world) = parse_wit(&wit_dir, None)?;
    let mut opts = crate::ast::Opt::new(args);
    let mut files = Files::default();
    opts.generate_component(&resolve, world, &mut files)?;

    // 4. Write generated files to the temp directory.
    for (name, content) in files.iter() {
        let path = wit_dir.as_path().join(name);
        // e.g. the `deps/conversion` directory of the `proxy:conversion` package
        fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, content)?;
    }
    // Re-generate exports world to bring in extra imports
    let (export_resolve, export_world) = parse_wit(&wit_dir, Some("tmp-exports"))?;
    opts.generate_exports_world(&export_resolve, export_world, &mut files)?;
    for (name, content) in files.iter() {
        let path = wit_dir.as_path().join(name);
        eprintln!("Generating: {}", path.display());
        fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, content)?;
    }

    // 5. Generate Rust binding for both import and export interface
    bindgen(
        dir,
        &wit_dir,
        &args.mode,
        "imports",
        "record_imports",
        args.one_byte_page_size,
    )?;
    bindgen(
        dir,
        &wit_dir,
        &args.mode,
        "exports",
        "record_exports",
        args.one_byte_page_size,
    )?;
    Ok(opts)
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
    use_custom_allocator: bool,
) -> Result<()> {
    let out_dir = tmp_dir.join(dest_name);
    fs::create_dir_all(&out_dir)?;
    let (mut resolve, world) = parse_wit(wit_dir, Some(world_name))?;
    let bindings = crate::wit_util::generate_bindings(&mut resolve, world)?;
    fs::write(out_dir.join("bindings.rs"), &bindings)?;
    let lib = codegen::generate(mode, resolve, world, &bindings, use_custom_allocator)?;
    fs::write(out_dir.join("lib.rs"), lib)?;
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
        false,
    )?;
    // create component from the embedded module.
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
