//! `bedrock check` — the CI entrypoint enforcing C1–C7 (SPINE §4).
//!
//! Rules evaluated in order, aggregated, `RULE path:line message` each, exit
//! 1 on any violation.
//!
//! - C1 placement (situation/ shape, AGENTS.md confinement + drift)
//! - C2 YAML 1.2 + no anchors/aliases/merge-keys
//! - C3 LD profile + graph membership + no blank nodes
//! - C4 namespace schema validation
//! - C5 edge resolution (vertex @id set or repo path)
//! - C6 byte determinism vs committed graph.trig/plan projections
//! - C7 parse-back equivalence

use crate::compile;
use crate::contextreg::{ContextRegistry, NAMESPACES};
use crate::errors::{Fatal, Violation};
use crate::generate;
use crate::schema::SchemaRegistry;
use oxrdf::{NamedOrBlankNode, Quad, Term};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Everything `check` computed; `build` cheap-writes these artifacts.
pub struct Compiled {
    pub quads: Vec<Quad>,
    pub trig_bytes: Vec<u8>,
    /// (`situation/plan/<name>.trig` relative path, bytes)
    pub plan_trigs: Vec<(PathBuf, Vec<u8>)>,
    pub agents_md: String,
}

/// The set of source yamlld files under situation/ with their namespace.
struct SourceFile {
    rel: PathBuf,       // repo-relative, e.g. situation/definition/a.yamlld
    ns: Option<String>, // Some("definition") or None (references/)
    text: String,
}

/// Run all rules on `root`. Returns violations and the compiled graph.
pub fn run(root: &Path) -> Result<(Vec<Violation>, Option<Compiled>), Fatal> {
    let (mut out, compiled) = collect(root)?;
    if let Some(c) = &compiled {
        drift_checks(root, c, &mut out);
    }
    Ok((out, compiled))
}

/// Everything except the two drift checks (C6 committed graph.trig vs
/// regenerated, C1 AGENTS.md vs regenerated). `init`/`adopt` write
/// artifacts first, then run the full `run()`.
pub fn collect(root: &Path) -> Result<(Vec<Violation>, Option<Compiled>), Fatal> {
    let mut out: Vec<Violation> = Vec::new();
    let seed = resolve_seed(root)?;
    let registry = ContextRegistry::load(Some(&seed))?;
    let schemas = SchemaRegistry::load(Some(&seed))?;

    // ---------- C1: file placement ----------
    let sources = scan_situation(root, &mut out);
    scan_agents_md(root, &mut out);

    // ---------- per-file C2/C3/C4 ----------
    let mut all_quads: Vec<Quad> = Vec::new();
    let mut plan_quads: Vec<(PathBuf, Vec<Quad>)> = Vec::new();

    for f in &sources {
        // C2
        let parsed = compile::parse_yaml(&f.text, &rel(&f.rel));
        out.extend(parsed.violations.iter().cloned());
        let Some(value) = &parsed.value else {
            continue;
        };

        // C3 profile
        out.extend(compile::ld_profile(value, &rel(&f.rel), &f.text, &registry));

        // C3 expansion + graph membership
        let ns_name: Option<&str> = f.ns.as_deref();
        match compile::expand(value, &rel(&f.rel), &f.text, &registry) {
            Ok(quads) => match compile::remap_graphs(quads, ns_name, &rel(&f.rel), &f.text) {
                Ok(quads) => {
                    // C4
                    out.extend(schemas.validate(ns_name, value, &rel(&f.rel), &f.text));
                    if f.ns.as_deref() == Some("plan") {
                        plan_quads.push((f.rel.clone(), quads.clone()));
                    }
                    all_quads.extend(quads);
                }
                Err(vs) => out.extend(vs),
            },
            Err(vs) => out.extend(vs),
        }
    }

    // ---------- compile + C7 ----------
    let sorted = compile::sort_quads(all_quads.clone());
    let trig = compile::serialize_trig(&sorted).map_err(Fatal)?;
    // C7: parse-back equivalence — emitted TriG must decode to the dataset
    // we compiled.
    if let Some(v) = compile::verify_parseback(&sorted, &trig) {
        out.push(v);
    }

    // ---------- C5: edge resolution ----------
    let vertex_ids: BTreeSet<String> = all_quads
        .iter()
        .filter_map(|q| match &q.subject {
            NamedOrBlankNode::NamedNode(n) => Some(n.as_str().to_string()),
            NamedOrBlankNode::BlankNode(_) => None,
        })
        .collect();
    check_edges(&all_quads, &vertex_ids, root, &mut out);

    // ---------- register ----------
    let digest = generate::digest_hex(&trig);
    let agents = generate::generate_agents_md(root, &sorted);
    let agents = generate::stamp_digest(&agents, &digest);

    let plan_trigs: Vec<(PathBuf, Vec<u8>)> = plan_quads
        .into_iter()
        .map(|(rel, quads)| {
            let bytes =
                compile::serialize_trig(&compile::sort_quads(quads)).expect("plan serializes");
            (rel.clone().with_extension("trig"), bytes)
        })
        .collect();

    let compiled = Compiled {
        quads: sorted,
        trig_bytes: trig,
        plan_trigs,
        agents_md: agents,
    };
    Ok((out, Some(compiled)))
}

/// C6 committed-projection drift + C1 AGENTS.md drift.
pub fn drift_checks(root: &Path, compiled: &Compiled, out: &mut Vec<Violation>) {
    // C6: committed graph.trig.
    let graph_path = root.join("situation").join("graph.trig");
    if graph_path.exists() {
        match std::fs::read(&graph_path) {
            Ok(existing) if existing != compiled.trig_bytes => out.push(Violation::new(
                "C6",
                "situation/graph.trig",
                1,
                "committed graph.trig differs from deterministic regenerated output (run bedrock build)"
                    .to_string(),
            )),
            Ok(_) => {}
            Err(e) => out.push(Violation::new(
                "C6",
                "situation/graph.trig",
                0,
                format!("cannot read {}: {e}", graph_path.display()),
            )),
        }
    }
    // C6: committed plan projections.
    for (rel, bytes) in &compiled.plan_trigs {
        let committed = root.join(rel);
        if committed.exists()
            && let Ok(existing) = std::fs::read(&committed)
            && existing != *bytes
        {
            out.push(Violation::new(
                "C6",
                rel.to_string_lossy().into_owned(),
                1,
                "committed plan projection differs from deterministic regenerated output (run bedrock build)"
                    .to_string(),
            ));
        }
    }

    // C1: root AGENTS.md hand-edit drift.
    let agents_path = root.join("AGENTS.md");
    match std::fs::read(&agents_path) {
        Ok(existing) if existing != compiled.agents_md.as_bytes() => out.push(Violation::new(
            "C1",
            "AGENTS.md",
            1,
            "root AGENTS.md is out of date or hand-edited; run bedrock build (it is generated — never hand-edited, SPINE §5)"
                .to_string(),
        )),
        Ok(_) => {}
        Err(_) => {} // absent root AGENTS.md is not drift (init/adopt generate it)
    }
}

/// Resolve the seed directory for `check`/`build`: `BEDROCK_SEED` override,
/// else `<root>/seed` (the copy `init`/`adopt` install into the consumer).
///
/// Deliberately NO embedded fallback here: a repo under `check`/`build` must
/// carry its own installed seed (SPINE §2/§8) — the embedded copy is the
/// `init`/`adopt` bootstrap, resolved separately in `install.rs`.
pub fn resolve_seed(root: &Path) -> Result<PathBuf, Fatal> {
    if let Ok(v) = std::env::var("BEDROCK_SEED") {
        if !v.is_empty() {
            let p = PathBuf::from(v);
            if !p.is_dir() {
                return Err(Fatal(format!(
                    "BEDROCK_SEED points at a missing directory: {}",
                    p.display()
                )));
            }
            return Ok(p);
        }
    }
    let p = root.join("seed");
    if !p.is_dir() {
        return Err(Fatal(format!(
            "{}: no seed/ directory (SPINE §2/§8). Run `bedrock init` to install it, or set BEDROCK_SEED",
            p.display()
        )));
    }
    Ok(p)
}

/// C1 placement: walk situation/ per §3.
fn scan_situation(root: &Path, out: &mut Vec<Violation>) -> Vec<SourceFile> {
    let sit = root.join("situation");
    let mut sources = Vec::new();
    let Ok(entries) = std::fs::read_dir(&sit) else {
        out.push(Violation::new(
            "C1",
            "situation",
            1,
            "situation/ missing or unreadable",
        ));
        return sources;
    };
    let mut root_entries: Vec<PathBuf> = Vec::new();
    for e in entries.flatten() {
        root_entries.push(e.path());
    }
    root_entries.sort();
    for p in root_entries {
        let name = p.file_name().unwrap_or_default();
        let name = name.to_str().unwrap_or_default();
        let is_hidden = name.starts_with('.');
        if p.is_dir() {
            if NAMESPACES.contains(&name) {
                scan_namespace(root, &p, out, &mut sources);
            } else {
                out.push(Violation::new(
                    "C1",
                    format!("situation/{name}"),
                    1,
                    format!("only the six Situation namespaces are allowed (definition|architecture|risk|plan|record|references), got directory `{name}`"),
                ));
            }
        } else if is_hidden || name == "graph.trig" {
            // generated graph.trig or dot files (e.g. .gitkeep) are legal
        } else {
            out.push(Violation::new(
                "C1",
                format!("situation/{name}"),
                1,
                "unexpected file directly under situation/; only the six namespace directories (and the generated graph.trig) live here"
                    .to_string(),
            ));
        }
    }
    sources
}

fn scan_namespace(
    root: &Path,
    ns_dir: &Path,
    out: &mut Vec<Violation>,
    sources: &mut Vec<SourceFile>,
) {
    let ns = ns_dir
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(ns_dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    entries.sort();
    for p in entries {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let rel = rel(p.strip_prefix(root).unwrap_or(&p));
        if ns == "references" {
            // references/ may nest freely and hold non-YAML-LD files (§3).
            push_files_recursive(root, &p, sources);
            continue;
        }
        if p.is_dir() {
            out.push(Violation::new(
                "C1",
                &rel,
                1,
                "namespace directories (definition|architecture|risk|plan|record) hold flat vertex files only — no subdirectories"
                    .to_string(),
            ));
            continue;
        }
        if name.starts_with('.') {
            continue; // .gitkeep etc.
        }
        let compiles = name.ends_with(".yamlld") || name.ends_with(".trig");
        if !compiles {
            out.push(Violation::new(
                "C1",
                &rel,
                1,
                format!("file `{name}` is not a vertex; namespace directories hold flat `<local-name>.yamlld` vertices (generated `.trig` projections excepted)"),
            ));
            continue;
        }
        if name.ends_with(".yamlld") {
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            sources.push(SourceFile {
                rel: strip_situation_rel(root, &p),
                ns: Some(ns.clone()),
                text,
            });
        }
        // .trig files are generated artifacts; C6 verifies them.
    }
}

fn push_files_recursive(root: &Path, dir: &Path, sources: &mut Vec<SourceFile>) {
    if dir.is_dir() {
        let mut es: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|rd| rd.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        es.sort();
        for p in es {
            push_files_recursive(root, &p, sources);
        }
    } else if dir.extension().map(|e| e == "yamlld").unwrap_or(false) {
        let text = std::fs::read_to_string(dir).unwrap_or_default();
        sources.push(SourceFile {
            rel: strip_situation_rel(root, dir),
            ns: None,
            text,
        });
    }
}

fn strip_situation_rel(root: &Path, p: &Path) -> PathBuf {
    // situation/<ns>/<name>.yamlld → situation/<ns>/<name>.yamlld (repo-relative)

    p.strip_prefix(root).unwrap_or(p).to_path_buf()
}

fn rel(p: impl AsRef<Path>) -> String {
    p.as_ref().to_string_lossy().into_owned()
}

/// C1: any AGENTS.md not at the repo root → FAIL; root AGENTS.md must not
/// drift (drift is checked later, after regeneration, by the caller wiring).
fn scan_agents_md(root: &Path, out: &mut Vec<Violation>) {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name().to_str().unwrap_or_default().to_string();
            if p.is_dir() {
                // do not descend into target/, .git, or quarantined test rigs:
                // fixtures are rig material (they plant deliberate violations
                // to test the rules) and never carry repo law.
                if name == ".git" || name == "target" {
                    continue;
                }
                if name == "fixtures" && dir.file_name().and_then(|n| n.to_str()) == Some("tests") {
                    continue;
                }
                stack.push(p);
            } else if name == "AGENTS.md" && p != root.join("AGENTS.md") {
                let rel = p.strip_prefix(root).unwrap_or(&p);
                out.push(Violation::new(
                    "C1",
                    rel.to_string_lossy().into_owned(),
                    1,
                    "AGENTS.md outside the repo root — exactly ONE generated AGENTS.md per repo, at root (SPINE §3, §5)"
                        .to_string(),
                ));
            }
        }
    }
}

/// C5: every non-type IRI object must resolve to a vertex @id or a repo path.
fn check_edges(
    all_quads: &[Quad],
    vertex_ids: &BTreeSet<String>,
    root: &Path,
    out: &mut Vec<Violation>,
) {
    for q in all_quads {
        if q.predicate == oxrdf::vocab::rdf::TYPE {
            continue; // class IRIs are vocabulary, not edges
        }
        let Term::NamedNode(obj) = &q.object else {
            continue;
        };
        let target = obj.as_str();
        if vertex_ids.contains(target) {
            continue;
        }
        if let Some(path) = target.strip_prefix(crate::contextreg::PATH_PREFIX) {
            if root.join(path).exists() {
                continue;
            }
            out.push(Violation::new(
                "C5",
                "situation",
                1,
                format!("edge target {target} → repo path `{path}` does not exist"),
            ));
            continue;
        }
        out.push(Violation::new(
            "C5",
            "situation",
            1,
            format!("edge target {target} is neither a vertex @id in this repo's graph nor an existing repo path"),
        ));
    }
}
