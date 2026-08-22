//! `init`/`adopt`/`build` — installs, epoch records, artifact writing, and
//! the §1 version gate.

use crate::check::{self, Compiled};
use crate::errors::{Fatal, Violation};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Version gate (SPINE §1): until the crate's first publication the gate is
/// a no-op that prints a notice. Once published, flip this to `true` and the
/// network check (against crates.io's `yeetz-bedrock`) becomes active — it
/// must never run for `check`/`build`, only `init`/`adopt`.
pub const VERSION_GATE_ENABLED: bool = false;

/// `bedrock build` — compile TriG + regenerate AGENTS.md.
///
/// Validates the *source* (check::collect: C1 placement, C2–C5, C7) and
/// aborts on any source violation; then regenerates its own outputs
/// (graph.trig, plan projections, AGENTS.md) — a stale register is exactly
/// what build fixes, so the drift checks (C1-AGENTS, C6-committed) are only
/// enforced by `check`, the CI gate, and verified against the fresh state
/// after writing.
pub fn build(root: &Path) -> Result<(), Fatal> {
    let (violations, compiled) = check::collect(root)?;
    if !violations.is_empty() {
        print_violations(&violations);
        return Err(Fatal("build aborted: source check failed".to_string()));
    }
    let c = compiled.expect("collect always yields a compiled graph");
    write_artifacts(root, &c)?;
    let (post, _) = check::run(root)?;
    if !post.is_empty() {
        print_violations(&post);
        return Err(Fatal(
            "build aborted: regenerated state does not pass the full check (non-deterministic output?)"
                .to_string(),
        ));
    }
    println!(
        "bedrock build: situation/graph.trig + {} plan projection(s) + AGENTS.md up to date",
        c.plan_trigs.len()
    );
    Ok(())
}

/// Write graph.trig, per-plan .trig, and AGENTS.md (SPINE §5).
pub fn write_artifacts(root: &Path, c: &Compiled) -> Result<(), Fatal> {
    let graph_path = root.join("situation").join("graph.trig");
    std::fs::write(&graph_path, &c.trig_bytes)
        .map_err(|e| Fatal(format!("cannot write {}: {e}", graph_path.display())))?;
    for (rel, bytes) in &c.plan_trigs {
        let p = root.join(rel);
        std::fs::write(&p, bytes)
            .map_err(|e| Fatal(format!("cannot write {}: {e}", p.display())))?;
    }
    let agents = root.join("AGENTS.md");
    std::fs::write(&agents, c.agents_md.as_bytes())
        .map_err(|e| Fatal(format!("cannot write {}: {e}", agents.display())))?;
    Ok(())
}

/// `bedrock init` — seed a NEW repo (SPINE §1/§6).
pub fn init(target: &Path, offline: bool) -> Result<(), Fatal> {
    version_gate("init", offline)?;
    let seed = resolve_install_seed()?;
    tradition_check(target, false)?;
    install_seed(&seed, target)?;
    let record_short = write_epoch_record(target, "init", offline)?;
    finalize_and_verify(target, "init", record_short)
}

/// `bedrock adopt` — epoch-change an EXISTING repo (SPINE §6).
pub fn adopt(target: &Path, offline: bool) -> Result<(), Fatal> {
    version_gate("adopt", offline)?;
    let seed = resolve_install_seed()?;
    install_seed(&seed, target)?;
    let record_short = write_epoch_record(target, "adopt", offline)?;
    finalize_and_verify(target, "adopt", record_short)
}

fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// The compile-time-embedded seed tree, materialized once per process into a
/// unique temp dir (thread-safe: multi-threaded test binaries run init
/// concurrently; `LazyLock` guarantees a single materialization). The stored
/// `Result` keeps a failure loud without panicking the installer.
static EMBEDDED_SEED: std::sync::LazyLock<Result<PathBuf, String>> =
    std::sync::LazyLock::new(|| {
        let dir =
            std::env::temp_dir().join(format!("bedrock-embedded-seed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return Err(format!(
                "cannot create embedded-seed temp dir {}: {e}",
                dir.display()
            ));
        }
        crate::embedded::materialize(&dir).map_err(|e| {
            format!(
                "cannot materialize embedded seed into {}: {e}",
                dir.display()
            )
        })?;
        Ok(dir)
    });

fn embedded_seed() -> Result<&'static PathBuf, Fatal> {
    match EMBEDDED_SEED.as_ref() {
        Ok(p) => Ok(p),
        Err(e) => Err(Fatal(e.clone())),
    }
}

/// Resolve the seed for `init`/`adopt` — the three-tier SPINE §1 order:
///   1. explicit `BEDROCK_SEED` env var — a defined-but-missing directory is
///      a loud exit-1 failure (never silently falls through to the default);
///   2. `./seed` in the caller's cwd, if present;
///   3. the compile-time-embedded seed/ tree (what makes a cargo-installed
///      binary standalone).
fn resolve_install_seed() -> Result<PathBuf, Fatal> {
    if let Ok(v) = std::env::var("BEDROCK_SEED") {
        if !v.is_empty() {
            let p = PathBuf::from(&v);
            if !p.is_dir() {
                return Err(Fatal(format!(
                    "BEDROCK_SEED points at a missing directory: {}",
                    p.display()
                )));
            }
            return Ok(p);
        }
    }
    let cwd_seed = current_dir().join("seed");
    if cwd_seed.is_dir() {
        return Ok(cwd_seed);
    }
    Ok(embedded_seed()?.clone())
}

/// Recursive directory copy (dirs + files). Overwrites existing files;
/// preserves nothing else (no symlinks expected in the seed tree).
fn copy_dir(src: &Path, dst: &Path) -> Result<(), Fatal> {
    std::fs::create_dir_all(dst)
        .map_err(|e| Fatal(format!("cannot create {}: {e}", dst.display())))?;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(src)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .map_err(|e| Fatal(format!("cannot read {}: {e}", src.display())))?;
    entries.sort();
    for p in entries {
        let name = p.file_name().unwrap_or_default();
        let to = dst.join(name);
        if p.is_dir() {
            copy_dir(&p, &to)?;
        } else {
            // Ignore the seed's own .gitkeep/dotfiles when they'd shadow a
            // real file; plain overwrite otherwise.
            std::fs::copy(&p, &to).map_err(|e| {
                Fatal(format!(
                    "cannot copy {} → {}: {e}",
                    p.display(),
                    to.display()
                ))
            })?;
        }
    }
    Ok(())
}

/// init: the target must not already carry a situation/ (it is a NEW repo).
fn tradition_check(target: &Path, _is_init: bool) -> Result<(), Fatal> {
    let sit = target.join("situation");
    if sit.exists()
        && std::fs::read_dir(&sit)
            .map(|rd| rd.flatten().count())
            .unwrap_or(0)
            > 0
    {
        return Err(Fatal(format!(
            "{} already contains situation/ — `bedrock init` seeds a NEW repo; use `bedrock adopt` to epoch-change an existing one",
            target.display()
        )));
    }
    Ok(())
}

/// Copy the seed tree into the target.
///
/// SPINE §1: install situation/ skeleton + seed floor vertices + workflow;
/// schemas/context.yamlld keep the consumer's own `check` runnable.
///
/// Seed layout contract (W1↔W3): the skeleton may live at
/// `seed/situation-skeleton/` (W3's name) or `seed/situation/`; each of the
/// six namespaces is created even if the seed ships only `.gitkeep`. Any
/// YAML-LD floor vertices W2/W3 place under seed/ propagate verbatim (today
/// the portable floor lives in the repo's situation/, not seed/ — see the
/// summary's open seam). Workflow templates under seed/workflow/ are promoted
/// to `.github/workflows/`; `seed/gitignore.stanza` is appended to the
/// consumer's `.gitignore`.
fn install_seed(seed: &Path, target: &Path) -> Result<(), Fatal> {
    let skeleton = if seed.join("situation-skeleton").is_dir() {
        seed.join("situation-skeleton")
    } else {
        seed.join("situation")
    };
    if !skeleton.is_dir() {
        return Err(Fatal(format!(
            "{}: expected seed/situation-skeleton/ or seed/situation/ (SPINE §8 W3 skeleton). The seed tarball may be incomplete",
            skeleton.display()
        )));
    }
    for ns in crate::contextreg::NAMESPACES {
        let src = skeleton.join(ns);
        let dst = target.join("situation").join(ns);
        if src.is_dir() {
            copy_dir(&src, &dst)?;
        } else {
            std::fs::create_dir_all(&dst)
                .map_err(|e| Fatal(format!("cannot create {}: {e}", dst.display())))?;
            let _ = std::fs::write(dst.join(".gitkeep"), "");
        }
    }

    // Copy the rest of seed/ → target/seed/ verbatim: schemas/, the context,
    // any floor vertices, and the workflow template directory.
    let seed_dst = target.join("seed");
    copy_dir(seed, &seed_dst)?;
    install_gitignore(seed, target)?;
    promote_workflow(target, &seed_dst)?;

    // Generate the consumer's first graph.trig/AGENTS.md is deferred to
    // `finalize_and_verify`, which runs after the epoch record exists.
    Ok(())
}

/// Append `seed/gitignore.stanza` (W3) to the consumer's .gitignore.
fn install_gitignore(seed: &Path, target: &Path) -> Result<(), Fatal> {
    let stanza = seed.join("gitignore.stanza");
    if !stanza.is_file() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&stanza)
        .map_err(|e| Fatal(format!("cannot read {}: {e}", stanza.display())))?;
    let gi = target.join(".gitignore");
    let existing = if gi.exists() {
        std::fs::read_to_string(&gi)
            .map_err(|e| Fatal(format!("cannot read {}: {e}", gi.display())))?
    } else {
        String::new()
    };
    if existing.contains(&content) {
        return Ok(());
    }
    let mut merged = existing.trim_end().to_string();
    if !merged.is_empty() {
        merged.push_str("\n\n");
    }
    merged.push_str(content.trim_end());
    merged.push('\n');
    std::fs::write(&gi, merged)
        .map_err(|e| Fatal(format!("cannot write {}: {e}", gi.display())))?;
    Ok(())
}

/// Promote a workflow template found in the installed seed/ into
/// `.github/workflows/` (W3's template name is not fixed by SPINE, so the
/// installer looks for files under a `workflows/` directory or whose name
/// contains "workflow").
fn promote_workflow(target: &Path, seed_dst: &Path) -> Result<(), Fatal> {
    let mut candidates = Vec::new();
    collect_workflow_files(seed_dst, &mut candidates);
    if candidates.is_empty() {
        return Ok(());
    }
    let wf_dir = target.join(".github").join("workflows");
    std::fs::create_dir_all(&wf_dir)
        .map_err(|e| Fatal(format!("cannot create {}: {e}", wf_dir.display())))?;
    for src in candidates {
        let name = src.file_name().unwrap_or_default();
        let dst = wf_dir.join(name);
        std::fs::copy(&src, &dst)
            .map_err(|e| Fatal(format!("cannot install workflow {}: {e}", dst.display())))?;
        // The installed seed/ copy is a template, not a live workflow: drop
        // it once promoted so the consumer repo has exactly one instance.
        let _ = std::fs::remove_file(&src);
    }
    Ok(())
}

fn collect_workflow_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_str().unwrap_or_default().to_string();
        if p.is_dir() {
            collect_workflow_files(&p, out);
        } else {
            // Match a workflow by its basename or by a `workflows`/`workflow`
            // ancestry component (e.g. .github/workflows/bedrock.sample.yml).
            let has_wf_dir = {
                let mut up = p.parent();
                let mut hit = false;
                while let Some(d) = up {
                    let c = d
                        .file_name()
                        .map(|s| s.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if c.contains("workflow") {
                        hit = true;
                        break;
                    }
                    if c.is_empty() || d == dir {
                        break;
                    }
                    up = d.parent();
                }
                hit
            };
            if name.to_lowercase().contains("workflow") || has_wf_dir {
                out.push(p);
            }
        }
    }
}

/// Write the epoch record vertex (SPINE §6), conforming to the record
/// namespace schema shipped in seed/schemas/record.json (W2 contract):
/// vertex @id, EpochRecord type, and `commit`/`version`/`mode`/`offline`/
/// `statement`.
fn write_epoch_record(target: &Path, mode: &str, offline: bool) -> Result<String, Fatal> {
    let sha = current_sha(target);
    let short = if sha.len() >= 12 { &sha[..12] } else { &sha };
    let date = utc_date_fragment();
    let fname = format!("epoch-{date}-{short}.yamlld");
    let id = format!("https://yeetz.dev/bedrock/vertex/epoch-{date}-{short}");
    let statement = "Everything after this commit operates under bedrock; prior history is reference, never law.";

    let body = format!(
        "\"@context\": \"https://yeetz.dev/bedrock/context/v1\"\n\
         \"@id\": \"{id}\"\n\
         \"@type\": \"https://yeetz.dev/bedrock/ontology/EpochRecord\"\n\
         commit: \"{sha}\"\n\
         version: \"{}\"\n\
         mode: {mode}\n\
         offline: {offline}\n\
         statement: \"{statement}\"\n",
        env!("CARGO_PKG_VERSION"),
    );

    let dir = target.join("situation").join("record");
    std::fs::create_dir_all(&dir)
        .map_err(|e| Fatal(format!("cannot create {}: {e}", dir.display())))?;
    let path = dir.join(&fname);
    std::fs::write(&path, body.as_bytes())
        .map_err(|e| Fatal(format!("cannot write epoch record {}: {e}", path.display())))?;
    println!("bedrock {mode}: wrote {}", path.display());
    Ok(fname)
}

/// `git rev-parse HEAD` if the target is a git repo, else the "unborn branch"
/// null-sha (no prior cut exists yet — init's first record).
fn current_sha(target: &Path) -> String {
    match Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(target)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "0".repeat(40),
    }
}

/// UTC date fragment `YYYYMMDD` (for the epoch filename).
fn utc_date_fragment() -> String {
    // Deterministic, dependency-free UTC date via shell `date -u` is avoided;
    // use std::time with a tiny civil-date conversion.
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let (y, m, d) = civil_from_days(now / 86400);
    format!("{y:04}{m:02}{d:02}")
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The version gate: currently a no-op with a notice until first publication.
/// `--offline` is stamped into the epoch record regardless.
fn version_gate(cmd: &str, offline: bool) -> Result<(), Fatal> {
    if VERSION_GATE_ENABLED {
        // TODO(publication): query crates.io for the latest `yeetz-bedrock`; if
        // the running binary is stale, refuse and print the update command;
        // `--offline` skips the check but still stamps the version into the
        // record (SPINE §1). Network MUST never be touched by check/build.
        let _ = offline;
    } else {
        println!(
            "bedrock {}: version gate inactive until the crate's first publication (SPINE §1)",
            cmd
        );
    }
    Ok(())
}

/// Finalize: compile + write artifacts + verify, then emit the instruction
/// set. Runs installations, then a full check; loud exit on failure.
fn finalize_and_verify(target: &Path, mode: &str, _record: String) -> Result<(), Fatal> {
    // 1. Compile + write artifacts (graph.trig, plans, AGENTS.md).
    //    The pre-write compile catches vertex errors without tripping on the
    //    (adopt-carryover) stale AGENTS.md.
    let (pre, compiled) = check::collect(target)?;
    if !pre.is_empty() {
        print_violations(&pre);
        return Err(Fatal(format!(
            "bedrock {mode}: installed situation/ fails compile validation"
        )));
    }
    let c = compiled.expect("collect always yields a graph");
    write_artifacts(target, &c)?;

    // 2. Full check on the fresh state (validates generated artifacts +
    //    new epoch record + register projection).
    let (violations, _) = check::run(target)?;
    if !violations.is_empty() {
        print_violations(&violations);
        return Err(Fatal(format!(
            "bedrock {mode}: installed situation/ does not pass `bedrock check`"
        )));
    }
    println!("bedrock {mode}: situation/ installed, epoch record written, check passes\n");
    print_instructions(mode);
    Ok(())
}

fn print_instructions(mode: &str) {
    let text = if mode == "init" {
        include_str!("instructions/init.md")
    } else {
        include_str!("instructions/adopt.md")
    };
    print!("{text}");
}

pub fn print_violations(v: &[Violation]) {
    for x in v {
        eprintln!("{x}");
    }
}
