#![allow(dead_code)] // each test binary uses a subset of these helpers
//! Shared helpers for bedrock integration tests.
//!
//! Fixtures live under `tests/fixtures/`; tests materialize them into
//! scratch dirs and drive the compiled `bedrock` binary, pointing it at the
//! shared fixture seed via `BEDROCK_SEED` (check/build/init resolve the seed
//! from that env var before `<root>/seed`).

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn bedrock_exe() -> &'static str {
    env!("CARGO_BIN_EXE_bedrock")
}

pub fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixture_seed() -> PathBuf {
    manifest().join("tests/fixtures/seed")
}

pub fn donor_fixtures_dir() -> PathBuf {
    manifest().join("spec/donor-fixtures")
}

pub fn donor_execution_record() -> PathBuf {
    manifest().join("spec/donor-execution-record.yamlld")
}

pub fn version_json(version: &str) -> String {
    format!("{{\"crate\":{{\"max_version\":\"{version}\",\"num_versions\":1}}}}")
}

pub fn use_current_gate(cmd: &mut Command) {
    cmd.env(
        "BEDROCK_VERSION_JSON",
        version_json(env!("CARGO_PKG_VERSION")),
    );
}

pub fn run_gate(args: &[&str], cwd: &Path, response: &str) -> (i32, String, String) {
    let out = Command::new(bedrock_exe())
        .args(args)
        .current_dir(cwd)
        .env("BEDROCK_SEED", fixture_seed())
        .env("BEDROCK_VERSION_JSON", response)
        .output()
        .expect("spawn bedrock");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Run the binary with installed seed fixtures and a current-version registry
/// response. Integration tests are network-free by construction.
pub fn run(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let mut command = Command::new(bedrock_exe());
    command
        .args(args)
        .current_dir(cwd)
        .env("BEDROCK_SEED", fixture_seed());
    use_current_gate(&mut command);
    let out = command.output().expect("spawn bedrock");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Run without forcing BEDROCK_SEED; the gate still uses a deterministic
/// current-version response.
pub fn run_no_seed(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let mut command = Command::new(bedrock_exe());
    command.args(args).current_dir(cwd);
    use_current_gate(&mut command);
    let out = command.output().expect("spawn bedrock");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A fresh scratch directory, removed on drop.
pub struct Scratch {
    pub dir: PathBuf,
}

impl Scratch {
    pub fn new(tag: &str) -> Self {
        // pid + clock can collide across parallel test threads in the same
        // process; a global monotonic counter guarantees uniqueness.
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "bedrock-test-{tag}-{}-{seq}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        Scratch { dir: base }
    }
    pub fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Copy `src` (directory) recursively into `dst`.
pub fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), to).unwrap();
        }
    }
}

/// Materialize a fixture dir (which is a repo root without seed/) into a
/// scratch dir. `fixture` is relative to `tests/fixtures/rules`.
pub fn materialize(fixture: &str) -> Scratch {
    let src = manifest().join("tests/fixtures/rules").join(fixture);
    let s = Scratch::new(&fixture.replace('/', "-"));
    for e in std::fs::read_dir(&src).unwrap() {
        let e = e.unwrap();
        let to = s.dir.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), to).unwrap();
        }
    }
    s
}

/// Build + check a materialized fixture; assert both exit 0.
pub fn build_and_check_ok(s: &Scratch) {
    let (c, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "build should pass\nstdout:\n{out}\nstderr:\n{err}");
    let (c, out, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "check should pass\n{out}");
}
