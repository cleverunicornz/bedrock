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
//! - C6 byte determinism: the committed AGENTS.md (the compiled graph) vs regenerated
//! - C7 parse-back equivalence
//! - C8 witness gate (base protocol): disposition `done` requires ≥1 witness
//! - C9 base-type: every vertex @type intersects the base @type set
//! - C10 digest-skew: installed base files vs this binary's canonical
//!   stamped form (embedded template + provenance stamp for the current
//!   version)

use crate::compile;
use crate::contextreg::{ContextRegistry, GRAPH_NAMESPACES, NAMESPACES, OPERATING_REF_PATH};
use crate::errors::{Fatal, Violation};
use crate::generate;
use crate::schema::SchemaRegistry;
use oxrdf::{NamedOrBlankNode, Quad, Term};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub struct Compiled {
    pub quads: Vec<Quad>,
    /// The single generated artifact: root AGENTS.md = comment preamble +
    /// complete TriG body (0.4.0 base protocol — the file the harness
    /// injects is the graph itself).
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
    // C10: installed base files (schemas, context, operating reference)
    // must match this binary's embedded copies.
    digest_skew_checks(root, &mut out);
    Ok((out, compiled))
}

/// Everything except the drift check (C1: committed AGENTS.md — the compiled
/// graph — vs regenerated). `init`/`adopt` write artifacts first, then run
/// the full `run()`.
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
                Ok(mut quads) => {
                    // C4
                    out.extend(schemas.validate(ns_name, value, &rel(&f.rel), &f.text));
                    // C8 witness gate (plan + record namespace)
                    out.extend(c8_witness_gate(ns_name, value, &rel(&f.rel), &f.text));
                    // C9 base-type intersection
                    out.extend(c9_base_type(value, &rel(&f.rel), &f.text));
                    // 0.4.0: the pointer is the document itself — every
                    // vertex carries an automatic `document` edge to its
                    // source file, so an agent reading the injected graph
                    // can ingest the full node context on demand.
                    if let Some(id) = value.get("@id").and_then(|v| v.as_str())
                        && let Some(graph) = quads.first().map(|q| q.graph_name.clone())
                    {
                        quads.push(Quad {
                            subject: NamedOrBlankNode::NamedNode(
                                oxrdf::NamedNode::new(id).expect("vertex @id is absolute"),
                            ),
                            predicate: oxrdf::NamedNode::new("https://yeetz.dev/bedrock/document")
                                .expect("static IRI"),
                            object: Term::NamedNode(
                                oxrdf::NamedNode::new(format!(
                                    "https://yeetz.dev/bedrock/path/{}",
                                    f.rel.display()
                                ))
                                .expect("path IRI is absolute"),
                            ),
                            graph_name: graph,
                        });
                    }
                    all_quads.extend(quads);
                }
                Err(vs) => out.extend(vs),
            },
            Err(vs) => out.extend(vs),
        }
    }

    // ---------- compile ----------
    let sorted = compile::sort_quads(all_quads.clone());
    let raw_trig = compile::serialize_trig(&sorted).map_err(Fatal)?;

    // ---------- the single artifact: AGENTS.md IS the graph ----------
    // Preamble (# comments) + the complete TriG body; the digest pins the
    // body bytes. C7 parse-back reads the emitted AGENTS.md itself — the
    // same file agents read (comments are legal TriG).
    let digest = generate::digest_hex(&raw_trig);
    let agents = generate::generate_agents_md(root, &sorted, &String::from_utf8_lossy(&raw_trig));
    let agents = generate::stamp_digest(&agents, &digest);
    if let Some(v) = compile::verify_parseback(&sorted, agents.as_bytes()) {
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

    let compiled = Compiled {
        quads: sorted,
        agents_md: agents,
    };
    Ok((out, Some(compiled)))
}

/// Drift check: the committed AGENTS.md (the compiled graph) must
/// byte-match regeneration. One artifact, one drift rule (0.4.0).
pub fn drift_checks(root: &Path, compiled: &Compiled, out: &mut Vec<Violation>) {
    // C1: root AGENTS.md — the compiled graph — drift.

    // C1: root AGENTS.md hand-edit drift.
    let agents_path = root.join("AGENTS.md");
    match std::fs::read(&agents_path) {
        Ok(existing) if existing != compiled.agents_md.as_bytes() => out.push(Violation::new(
            "C1",
            "AGENTS.md",
            1,
            "root AGENTS.md (the compiled graph) is out of date or hand-edited; run bedrock build (it is generated — never hand-edited, SPINE §5)"
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
        } else if name == "graph.trig" {
            out.push(Violation::new(
                "C1",
                "situation/graph.trig",
                1,
                "legacy generated artifact (0.4.0 compiles the graph into the root AGENTS.md — one artifact); `bedrock build` deletes it"
                    .to_string(),
            ));
        } else if is_hidden {
            // dot files (e.g. .gitkeep) are legal
        } else {
            out.push(Violation::new(
                "C1",
                format!("situation/{name}"),
                1,
                "unexpected file directly under situation/; only the six namespace directories live here"
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
        if name.ends_with(".trig") {
            out.push(Violation::new(
                "C1",
                &rel,
                1,
                format!("legacy generated artifact `{name}` (0.4.0 compiles the graph into the root AGENTS.md — one artifact); `bedrock build` deletes it"),
            ));
            continue;
        }
        if !name.ends_with(".yamlld") {
            out.push(Violation::new(
                "C1",
                &rel,
                1,
                format!("file `{name}` is not a vertex; namespace directories hold flat `<local-name>.yamlld` vertices"),
            ));
            continue;
        }
        {
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

/// C8 (witness gate, base protocol): a Plan or ReflectVerdict whose
/// `disposition.state` is `done` must carry at least one `witnesses` entry —
/// no witness, no done. Judged on the parsed vertex so the violation is
/// line-cited at the `done` (the source schema has already validated shape).
fn c8_witness_gate(ns: Option<&str>, value: &Value, rel: &str, src: &str) -> Vec<Violation> {
    let gate_type = match ns {
        Some("plan") => crate::contextreg::ontology_type("Plan"),
        Some("record") => crate::contextreg::ontology_type("ReflectVerdict"),
        _ => return Vec::new(),
    };
    let types = type_iris(value);
    if !types.contains(&gate_type) {
        return Vec::new();
    }
    let Some(state) = value
        .get("disposition")
        .and_then(|d| d.get("state"))
        .and_then(|s| s.as_str())
    else {
        return Vec::new(); // no disposition declared → no gate
    };
    if state != "done" {
        return Vec::new();
    }
    let witnesses = value
        .get("witnesses")
        .and_then(|w| w.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if witnesses == 0 {
        vec![Violation::new(
            "C8",
            rel,
            crate::errors::line_of(src, "done"),
            "disposition is `done` with zero witnesses — no witness, no done (base protocol chain rule; a witness is a retained CI-run-URL observation, never a local attestation)",
        )]
    } else {
        Vec::new()
    }
}

/// C9 (base-type, base protocol): every vertex's `@type` set must intersect
/// the closed base @type set. Repo archetypes may ride alongside a base type
/// in the same array, never alone; they never redefine a base term.
fn c9_base_type(value: &Value, rel: &str, src: &str) -> Vec<Violation> {
    let types = type_iris(value);
    if types.is_empty() {
        return Vec::new(); // missing/malformed @type is a C4/schema concern
    }
    if types
        .iter()
        .any(|t| crate::contextreg::BASE_TYPES.contains(&t.as_str()))
    {
        return Vec::new();
    }
    let bad = types.first().cloned().unwrap_or_default();
    vec![Violation::new(
        "C9",
        rel,
        crate::errors::line_of(src, &bad),
        format!(
            "vertex {bad} carries no base @type — every vertex must carry at least one of (Invariant|Breadcrumb|Term|Identity|SituationStructure|Risk|Plan|EpochRecord|DeployRecord|ReflectVerdict|Decision); repo archetypes ride alongside, never alone"
        ),
    )]
}

/// Extract the `@type` IRIs (string or array) of a parsed vertex.
fn type_iris(value: &Value) -> Vec<String> {
    match value.get("@type") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

/// C10 (digest-skew): installed seed/schemas/*.json, seed/context.yamlld, and
/// the operating reference must byte-match this binary's canonical stamped
/// form — the embedded template with the provenance stamp rendered for the
/// CURRENT binary version — else the repo runs against a different base
/// protocol than the installed binary and must be refreshed with `bedrock
/// update`. A version-skewed stamp is exactly this violation. Absence is not
/// skew — a repo that never installed a base file fails earlier and
/// differently.
fn digest_skew_checks(root: &Path, out: &mut Vec<Violation>) {
    let seed = &crate::embedded::SEED;
    use crate::provenance::Kind;
    for ns in GRAPH_NAMESPACES {
        let rel = format!("seed/schemas/{ns}.json");
        // SEED is rooted at `seed/`, so the in-tree path has no `seed/` prefix.
        let embedded = seed
            .get_file(format!("schemas/{ns}.json").as_str())
            .map(|f| f.contents());
        check_base_blob(root, out, &rel, embedded, Kind::Json);
    }
    let embedded_ctx = seed.get_file("context.yamlld").map(|f| f.contents());
    check_base_blob(root, out, "seed/context.yamlld", embedded_ctx, Kind::Hash);
    let embedded_op = Some(include_str!("embedded/bedrock-operating.md").as_bytes());
    check_base_blob(root, out, OPERATING_REF_PATH, embedded_op, Kind::Hash);
}

fn check_base_blob(
    root: &Path,
    out: &mut Vec<Violation>,
    rel: &str,
    embedded: Option<&[u8]>,
    kind: crate::provenance::Kind,
) {
    let Some(embedded) = embedded else {
        return; // the embedded copy is compile-time; absence here is a build defect
    };
    let p = root.join(rel);
    if !p.exists() {
        return;
    }
    // C10 coherence (0.2.1): the canonical expected form is the embedded
    // template + the provenance stamp rendered for the current binary
    // version — install, update, and this comparison use the same render, so
    // raw embedded bytes ≠ installed bytes and a stale stamp flags here.
    let version = crate::provenance::current_version();
    let expected = crate::provenance::render(kind, version, embedded);
    match std::fs::read(&p) {
        Ok(installed) if installed != expected => out.push(Violation::new(
            "C10",
            rel,
            0,
            format!(
                "installed {rel} differs from this binary's canonical stamped form (embedded template + provenance stamp v{version}) — run `bedrock update` to refresh the installed base files; if the base itself is wrong, file an issue: {} (base protocol C10; a version-skewed stamp is exactly this violation)",
                crate::provenance::REPO_URL,
            ),
        )),
        _ => {}
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
