//! # bedrock
//!
//! A single Rust binary that seeds and validates a repo's complete YAML-LD
//! `situation/`, then compiles its current resident working set into ONE
//! generated artifact: root `AGENTS.md`, the TriG graph every agent receives.
//!
//! Six commands:
//!
//! - `bedrock init [DIR]` — seed a NEW repo: install the situation skeleton +
//!   seed floor vertices + workflow, write the first epoch record, run `check`,
//!   print the init instruction set.
//! - `bedrock adopt [DIR]` — epoch-change an EXISTING repo: same install,
//!   plus an epoch record vertex declaring the cut line.
//! - `bedrock check [DIR]` — validate complete source plus resident projection
//!   (SPINE rules C1–C11); CI entrypoint.
//! - `bedrock build [DIR]` — `check`, then compile root `AGENTS.md`, the
//!   resident TriG working-set projection.
//! - `bedrock update [DIR]` — refresh the installed base files (schemas,
//!   context, operating reference) from this binary's embedded copies, then
//!   `check` + `build`.
//! - `bedrock help` — the contract, short.
//!
//! The seed/ instructions ship compiled into the binary (embedded at build
//! time), so a cargo-installed binary is standalone: `init`/`adopt` resolve
//! the seed from `BEDROCK_SEED` → `./seed` in the cwd → the embedded copy.
//!
//! # Example
//!
//! ```text
//! $ cargo install yeetz-bedrock
//! $ cd my-new-repo
//! $ bedrock init
//! bedrock init: wrote situation/record/epoch-20260822-40xxx.yamlld
//! $ git add situation seed AGENTS.md
//! $ git commit -m "adopt bedrock"
//! ```
//!
//! [TriG]: https://www.w3.org/TR/trig/

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match bedrock::cli::parse(&args) {
        Ok(cmd) => match bedrock::cli::run(cmd) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        Err(e) => {
            eprintln!("{e}");
            1
        }
    };
    std::process::exit(code);
}
