use anyhow::Context;
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::Builder;
use wit_bindgen_core::{Files, wit_parser};
use wit_bindgen_rust::Opts;
use wit_parser::{Resolve, WorldId};

mod ast;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the wasm component file.
    #[arg()]
    wasm_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 1. Create a tmp directory and initialize a new Rust project in it.
    let tmp_dir = Builder::new()
        .prefix("proxy-component-")
        .disable_cleanup(true)
        .tempdir_in("/tmp")?;
    let mut cargo = File::create(tmp_dir.path().join("Cargo.toml"))?;
    writeln!(
        cargo,
        r#"
[package]
name = "proxy-component"
version = "0.1.0"
edition = "2024"
[lib]
crate-type = ["cdylib"]
[dependencies]
wit-bindgen-rt = "0.43.0"
"#
    )?;
    drop(cargo);

    let wit_dir = tmp_dir.path().join("wit");
    let src_dir = tmp_dir.path().join("src");
    std::fs::create_dir_all(&wit_dir)?;
    std::fs::create_dir_all(&src_dir)?;

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
        anyhow::bail!("wasm-tools component wit failed with exit code: {}", status);
    }

    // 3. Parse the main wit file from tmp_dir/wit and feed into opts.generate_component
    let (resolve, world) = parse_wit(&wit_dir, None)?;
    let opts = ast::Opt::new();
    let mut files = Files::default();
    opts.generate_component(&resolve, world, &mut files)?;

    // 4. Write generated files to the temp directory.
    for (name, content) in files.iter() {
        let path = wit_dir.as_path().join(name);
        eprintln!("Generating: {}", path.display());
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write generated file to {}", path.display()))?;
    }

    // 5. Generate Rust binding
    // We could use `generate!` to make code simpler, but it increases build time.
    let mut opts = Opts::default();
    opts.proxy_component = true;
    opts.stubs = true;
    opts.runtime_path = Some("wit_bindgen_rt".to_owned());
    opts.generate_all = true;
    opts.format = true;
    let mut generator = opts.build();
    let (resolve, world) = parse_wit(&wit_dir, Some("imports"))?;
    let mut files = Files::default();
    generator.generate(&resolve, world, &mut files)?;
    for (_name, content) in files.iter() {
        let path = src_dir.as_path().join("lib.rs");
        eprintln!("Generating: {}", path.display());
        std::fs::write(&path, content)?;
    }

    // 6. cargo build
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--target=wasm32-wasip2")
        .current_dir(tmp_dir.path())
        .output()
        .context("Failed to execute cargo build. Is rustup target `wasm32-wasip2` installed?")?;

    if !build_output.status.success() {
        eprintln!("--- cargo build stdout ---");
        eprintln!("{}", String::from_utf8_lossy(&build_output.stdout));
        eprintln!("--- cargo build stderr ---");
        eprintln!("{}", String::from_utf8_lossy(&build_output.stderr));
        anyhow::bail!("cargo build failed with exit code: {}", build_output.status);
    }

    let generated_wasm_path = tmp_dir
        .path()
        .join("target/wasm32-wasip2/debug/proxy_component.wasm");

    let output_file_name = args
        .wasm_file
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}.proxy.wasm", s))
        .unwrap_or_else(|| "proxy.wasm".to_string());

    let output_path = PathBuf::from(&output_file_name);

    std::fs::copy(&generated_wasm_path, &output_path).with_context(|| {
        format!(
            "Failed to copy generated wasm from {} to {}",
            generated_wasm_path.display(),
            output_path.display()
        )
    })?;

    eprintln!(
        "Successfully generated proxy component at {}",
        output_path.display()
    );

    Ok(())
}

fn parse_wit(dir: &Path, world: Option<&str>) -> anyhow::Result<(Resolve, WorldId)> {
    let mut resolve = Resolve::default();
    let (pkg, _files) = resolve
        .push_dir(dir)
        .with_context(|| format!("Failed to parse wit files in {}", dir.display()))?;

    let world = resolve
        .select_world(pkg, world)
        .context("Failed to select a world from the parsed wit files")?;
    Ok((resolve, world))
}
