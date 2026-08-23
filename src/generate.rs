//! AGENTS.md register generation (SPINE §5).
//!
//! The register is a projection of the compiled graph, never embedded TriG:
//!   1. Title: repo name + one-sentence identity (architecture vertex marked
//!      `role: identity`).
//!   2. `## Invariants …` numbered; `layer: floor` first, then
//!      `layer: situated`, each rendered from the vertex's `statement`.
//!   3. `## Breadcrumbs`: one line per vertex typed Breadcrumb
//!      (`gate` + `pointer`).
//!   4. `## Where things live`: vertices carrying `path`/`source` edges
//!      under the `https://yeetz.dev/bedrock/path/` namespace (W2 contract).
//!   5. A terminal marker naming the generator version and the compiled-graph
//!      digest, so `bedrock check` can detect hand edits (C1).
//!
//! Matching is by IRI *local name* (last path segment), so it is robust to
//! the ontology base IRI W2's `seed/context.yamlld` chooses.

use crate::contextreg::PATH_PREFIX;
use oxrdf::{Quad, Term};
use std::collections::BTreeMap;

/// Predicate local names this generator understands.
fn local_name(iri: &str) -> &str {
    iri.rsplit(['#', '/']).next().unwrap_or(iri)
}

/// Rendered register for a repo root + compiled quads.
pub fn generate_agents_md(repo_root: &std::path::Path, quads: &[Quad]) -> String {
    let repo_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("repo")
        .to_string();

    // Group quads by subject (deterministic BTreeMap iteration).
    let mut by_subject: BTreeMap<String, Vec<&Quad>> = BTreeMap::new();
    for q in quads {
        match &q.subject {
            oxrdf::NamedOrBlankNode::NamedNode(n) => {
                by_subject
                    .entry(n.as_str().to_string())
                    .or_default()
                    .push(q);
            }
            oxrdf::NamedOrBlankNode::BlankNode(_) => {} // rejected by C3
        }
    }

    // Structure vertices.
    let mut invariants = Vec::new(); // (layer, sort_id, statement)
    let mut breadcrumbs = Vec::new(); // (gate, pointer)
    let mut paths = Vec::new(); // (label_or_id, path)
    let mut identity = None::<(String, String)>; // (label, statement)

    for (id, vqs) in &by_subject {
        let types: Vec<String> = vqs
            .iter()
            .filter(|q| q.predicate == oxrdf::vocab::rdf::TYPE)
            .filter_map(|q| match &q.object {
                Term::NamedNode(n) => Some(local_name(n.as_str()).to_string()),
                _ => None,
            })
            .collect();

        let stmt = literal_of(vqs, &["statement", "description", "instruction"]);
        let label = literal_of(vqs, &["label"]);
        let layer = literal_of(vqs, &["layer"]);
        let role = literal_of(vqs, &["role"]);

        if types.iter().any(|t| t == "Invariant") {
            let order = integer_of(vqs, &["sequence", "number", "order"]);
            invariants.push(Inv {
                layer: layer.clone().unwrap_or_else(|| "situated".to_string()),
                order,
                id: id.clone(),
                statement: stmt.clone().unwrap_or_else(|| id.clone()),
            });
        }
        if types.iter().any(|t| t == "Breadcrumb") {
            let gate = literal_of(vqs, &["gate"]).unwrap_or_default();
            let pointer = breadcrumb_pointer(vqs);
            breadcrumbs.push((gate, pointer));
        }
        if role.as_deref() == Some("identity") {
            identity = Some((
                label.clone().unwrap_or_else(|| id.clone()),
                stmt.clone().unwrap_or_default(),
            ));
        }
        // Where things live: path/source edges into the repo path namespace.
        for q in vqs {
            let pname = local_name(q.predicate.as_str());
            if (pname == "path" || pname == "source")
                && let Term::NamedNode(o) = &q.object
                && let Some(p) = o.as_str().strip_prefix(PATH_PREFIX)
            {
                paths.push((label_or(id, label.as_deref()), p.to_string()));
            }
        }
    }

    // Sort invariants: floor first, then situated; within a layer by
    // (optional sequence/number/order, else @id) for determinism.
    let layer_rank = |l: &str| if l == "floor" { 0 } else { 1 };
    invariants.sort_by(|a, b| {
        layer_rank(&a.layer)
            .cmp(&layer_rank(&b.layer))
            .then_with(|| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)))
    });
    breadcrumbs.sort();
    breadcrumbs.dedup();
    paths.sort();
    paths.dedup();

    let mut out = String::new();
    // 0.2.1 provenance: the register announces it is machine-owned on its
    // first line; the generator-version terminal marker stays at the foot.
    out.push_str(&crate::provenance::header(
        crate::provenance::Kind::Html,
        env!("CARGO_PKG_VERSION"),
    ));
    out.push('\n');

    // 1. Title.
    match &identity {
        Some((label, stmt)) if !stmt.trim().is_empty() => {
            out.push_str(&format!("# {} — {}\n\n", repo_name, stmt.trim()));
        }
        Some((label, _)) => out.push_str(&format!("# {} — {}\n\n", repo_name, label)),
        None => out.push_str(&format!("# {}\n\n", repo_name)),
    }

    // 2. Invariants.
    out.push_str("## Invariants — breaking any of these is wrong, whatever else is right\n\n");
    for (n, inv) in (1..).zip(&invariants) {
        out.push_str(&format!("{n}. {}\n", inv.statement.trim()));
    }
    if invariants.is_empty() {
        out.push_str("_None declared yet._\n");
    }
    out.push('\n');

    // 3. Breadcrumbs.
    out.push_str("## Breadcrumbs\n\n");
    if breadcrumbs.is_empty() {
        out.push_str("_None declared yet._\n");
    } else {
        for (gate, pointer) in &breadcrumbs {
            let gate = gate.trim();
            if pointer.is_empty() {
                out.push_str(&format!("- {gate}\n"));
            } else {
                out.push_str(&format!("- {gate}: {pointer}\n"));
            }
        }
    }
    out.push('\n');

    // 4. Where things live.
    out.push_str("## Where things live\n\n");
    if paths.is_empty() {
        out.push_str("_None declared yet._\n");
    } else {
        for (label, path) in &paths {
            out.push_str(&format!("- {}: {}\n", label, path));
        }
    }
    out.push('\n');

    // 5. Operating this repository (0.2.0 base protocol). Static register
    //    section: the five work verbs, the authoring loop, THE CHAIN in one
    //    line, and the pointer to the installed operating reference. Carried
    //    by init/adopt/update into situation/references/bedrock-operating.md
    //    (C10 guards it against drift).
    out.push_str("## Operating this repository\n\n");
    out.push_str("- Work happens on verb-prefixed branches, one verb per state (these are branch names, not directories):\n");
    out.push_str("- think/ — explore and decide.\n");
    out.push_str("- plan/ — write the plan as a graph.\n");
    out.push_str("- execute/ — do the work.\n");
    out.push_str("- reflect/ — review after the fact.\n");
    out.push_str("- deploy/ — place the result where it is recorded.\n");
    out.push_str("- Authoring loop: write a vertex, `bedrock check`, `bedrock build`, commit source AND generated output, open a PR — a human merges.\n");
    out.push_str("- The chain: every plan is a promise; its criteria are its oracle; its witnesses prove it held; its residual declares what was not assured.\n");
    out.push_str("- Decisions are records too: why a design is what it is lives in record/ Decision vertices — walk `supersedes` chains before relitigating a choice; write one when you close a fork. Semantics: situation/references/bedrock-operating.md\n");
    out.push_str("- The base protocol, in full — THE CHAIN, the ontology, every rule: situation/references/bedrock-operating.md\n");
    out.push('\n');

    // 6. Terminal marker (also written to the very end after everything).
    // The digest is filled in by the caller (it depends on graph.trig bytes).
    out.push_str(&format!(
        "<!-- bedrock {} digest __DIGEST__ (regenerated by `bedrock build`; hand edits are detected and rejected by `bedrock check`) -->\n",
        env!("CARGO_PKG_VERSION")
    ));
    out
}

struct Inv {
    layer: String,
    order: Option<i64>,
    id: String,
    statement: String,
}

/// First literal value of any of `names` (by predicate local name).
fn literal_of(vqs: &[&Quad], names: &[&str]) -> Option<String> {
    for q in vqs {
        if names.contains(&local_name(q.predicate.as_str()))
            && let Term::Literal(l) = &q.object
        {
            return Some(l.value().to_string());
        }
    }
    None
}

/// First small non-negative integer literal of any of `names`.
fn integer_of(vqs: &[&Quad], names: &[&str]) -> Option<i64> {
    for q in vqs {
        if names.contains(&local_name(q.predicate.as_str()))
            && let Term::Literal(l) = &q.object
        {
            if let Ok(n) = l.value().parse::<i64>() {
                return Some(n);
            }
        }
    }
    None
}

fn label_or(id: &str, label: Option<&str>) -> String {
    label
        .map(|s| s.to_string())
        .unwrap_or_else(|| local_name(id).to_string())
}

/// Breadcrumb pointer: the `pointer` term if present, else the `source`
/// (W2's breadcrumbs point at depth docs via `source`, a bedrock/path IRI).
fn breadcrumb_pointer(vqs: &[&Quad]) -> String {
    for q in vqs {
        let p = local_name(q.predicate.as_str());
        if (p == "pointer" || p == "source")
            && let Term::NamedNode(o) = &q.object
        {
            return crate::compile::path_pointer_of(o.as_str());
        }
    }
    literal_of(vqs, &["pointer", "source"]).unwrap_or_default()
}

/// Compute the sha256 digest over `bytes` (hex).
pub fn digest_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(64);
    for b in out {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Substitute the digest into a rendered register.
pub fn stamp_digest(text: &str, digest: &str) -> String {
    text.replace("__DIGEST__", digest)
}
