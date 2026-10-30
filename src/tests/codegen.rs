//! Snapshot tests of the sources generated from the WIT of a main component in
//! `tests/fixtures/proxy`, which compare them with `tests/snapshots/<mode>`.
//! `BLESS=1 cargo test` rewrites the snapshots. `make test` builds and runs generated sources.
use crate::Mode;
use crate::instrument::{InstrumentArgs, generate_sources};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "tests/fixtures/proxy";
const SNAPSHOTS: &str = "tests/snapshots";
/// The generated files to compare, relative to the generation directory.
const OUTPUTS: &[&str] = &[
    "wit/component.wit",
    "wit/deps/conversion.wit",
    "record_imports/lib.rs",
    "record_exports/lib.rs",
];

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let to = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), to).unwrap();
        }
    }
}

/// Compares `actual` with the snapshot at `path`, or rewrites it with `BLESS` set.
fn check(path: &Path, actual: &str) -> Result<(), String> {
    if std::env::var_os("BLESS").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, actual).unwrap();
        return Ok(());
    }
    let Ok(expected) = fs::read_to_string(path) else {
        return Err(format!("{} is missing", path.display()));
    };
    if expected == actual {
        return Ok(());
    }
    let line = expected
        .lines()
        .zip(actual.lines())
        .position(|(e, a)| e != a)
        .unwrap_or_else(|| expected.lines().count().min(actual.lines().count()));
    Err(format!(
        "{} differs at line {}:\n  expected: {}\n  actual:   {}",
        path.display(),
        line + 1,
        expected.lines().nth(line).unwrap_or("<end of file>"),
        actual.lines().nth(line).unwrap_or("<end of file>"),
    ))
}

/// Generates the sources for `mode` into a new directory.
fn generate(mode: &Mode) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(Path::new(FIXTURE), &dir.path().join("wit"));
    let args = InstrumentArgs {
        wasm_file: PathBuf::new(),
        mode: mode.clone(),
        use_host_recorder: false,
        one_byte_page_size: false,
    };
    generate_sources(&args, dir.path()).unwrap();
    dir
}

fn snapshot(mode: Mode) {
    let dir = generate(&mode);
    let errors: Vec<_> = OUTPUTS
        .iter()
        .filter_map(|output| {
            let actual = fs::read_to_string(dir.path().join(output)).unwrap();
            let snapshot = Path::new(SNAPSHOTS).join(mode.to_str()).join(output);
            check(&snapshot, &actual).err()
        })
        .collect();
    assert!(
        errors.is_empty(),
        "{}\nIf the change is intended, run `BLESS=1 cargo test` and review the snapshots.",
        errors.join("\n")
    );
}

#[test]
fn record() {
    snapshot(Mode::Record);
}
#[test]
fn replay() {
    snapshot(Mode::Replay);
}
#[test]
fn fuzz() {
    snapshot(Mode::Fuzz);
}
#[test]
fn dialog() {
    snapshot(Mode::Dialog);
}
