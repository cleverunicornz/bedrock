//! The compile-time-embedded seed/ tree (SPINE §1 fix: a cargo-installed
//! binary must be able to run `init`/`adopt` without any seed on disk).
//!
//! `seed/` remains the single source of embedded content: `include_dir!`
//! bakes whatever is on disk at compile time into the binary, so no
//! hand-maintained duplicate list can drift. At runtime the embedded tree is
//! materialized once into a temp directory and the installer treats it like
//! any on-disk seed.

use include_dir::{Dir, include_dir};
use std::path::Path;

/// The full `seed/` tree from this crate's repository root.
pub const SEED: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/seed");

/// Write the embedded seed tree under `dst`. `dst` is created if absent;
/// `include_dir`'s `Dir::extract` fails as soon as it meets an existing
/// file, so callers must materialize into a fresh (or emptied) directory.
pub fn materialize(dst: &Path) -> std::io::Result<()> {
    SEED.extract(dst)
}
