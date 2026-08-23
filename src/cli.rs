//! Hand-rolled CLI over `bedrock`'s five verbs. Deps stay minimal (no clap).

use crate::errors::Fatal;
use std::path::PathBuf;

pub const HELP: &str = "\
bedrock — compile an org's situation/ vertices into ONE artifact: the root AGENTS.md, the complete TriG graph the harness injects into every agent.

Usage:
  bedrock init   [--offline] [DIR]   seed a NEW repo: install situation/ skeleton + seed floor
                                     vertices + workflow; write the first epoch record; run check;
                                     print the init instruction set.
  bedrock adopt  [--offline] [DIR]   epoch-change an EXISTING repo: same install, plus an epoch
                                     record vertex declaring the cut line.
  bedrock check  [DIR]               validate situation/ (§4 C1–C10); CI entrypoint. Prints the size
                                     report: total AGENTS.md bytes + compiled faces over the 4096-char
                                     soft budget (SOFT lines — advisory, never failing). Exit 1 with
                                     one `RULE path:line message` per violation.
  bedrock build  [DIR]               check, then compile the root AGENTS.md — the complete TriG
                                     graph — and print the size report. Fails if check fails.
  bedrock update [DIR]               refresh the installed base files (schemas, context, operating
                                     reference, missing workflow template) from this binary's embedded
                                     copies; print exactly what changed; then run check + build.
  bedrock help                       this contract, short.
  bedrock --version / -V             print version.

Flags:
  --offline     skip the (post-publication) version gate; the version is still stamped into the
                epoch record. Only meaningful for init/adopt.
  DIR           repo root; defaults to the current directory.

Until the crate's first publication the version gate is a no-op with a notice (SPINE §1).
";

pub enum Command {
    Init { dir: PathBuf, offline: bool },
    Adopt { dir: PathBuf, offline: bool },
    Check { dir: PathBuf },
    Build { dir: PathBuf },
    Update { dir: PathBuf },
    Help,
    Version,
}

/// Parse argv (excluding `argv[0]`).
pub fn parse(args: &[String]) -> Result<Command, Fatal> {
    if args.is_empty() {
        return Ok(Command::Help);
    }
    // First non-flag token is the command; --offline may appear anywhere.
    let mut offline = false;
    let mut cmd: Option<String> = None;
    let mut dirs: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--offline" => offline = true,
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            s if s.starts_with('-') => {
                return Err(Fatal(format!("unknown flag `{a}`; try `bedrock help`")));
            }
            s if cmd.is_none() => cmd = Some(s.to_string()),
            s => dirs.push(s.to_string()),
        }
        i += 1;
    }
    let dir = dirs
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if dirs.len() > 1 {
        return Err(Fatal(
            "expected at most one DIR argument; try `bedrock help`".to_string(),
        ));
    }
    match cmd.as_deref() {
        Some("init") => Ok(Command::Init { dir, offline }),
        Some("adopt") => Ok(Command::Adopt { dir, offline }),
        Some("check") => Ok(Command::Check { dir }),
        Some("build") => Ok(Command::Build { dir }),
        Some("update") => Ok(Command::Update { dir }),
        Some("help") => Ok(Command::Help),
        Some(other) => Err(Fatal(format!(
            "unknown command `{other}`; try `bedrock help`"
        ))),
        None => Ok(Command::Help),
    }
}

pub fn run(cmd: Command) -> Result<i32, Fatal> {
    match cmd {
        Command::Help => {
            print!("{HELP}");
            Ok(0)
        }
        Command::Version => {
            println!("bedrock {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Command::Init { dir, offline } => {
            crate::install::init(&dir, offline)?;
            Ok(0)
        }
        Command::Adopt { dir, offline } => {
            crate::install::adopt(&dir, offline)?;
            Ok(0)
        }
        Command::Check { dir } => {
            let (violations, compiled) = crate::check::run(&dir)?;
            if let Some(c) = &compiled {
                crate::install::print_report(c);
            }
            if violations.is_empty() {
                println!("bedrock check: {} violations", 0);
                Ok(0)
            } else {
                crate::install::print_violations(&violations);
                println!("bedrock check: {} violation(s)", violations.len());
                Ok(1)
            }
        }
        Command::Build { dir } => {
            crate::install::build(&dir)?;
            Ok(0)
        }
        Command::Update { dir } => {
            crate::install::update(&dir)?;
            Ok(0)
        }
    }
}
