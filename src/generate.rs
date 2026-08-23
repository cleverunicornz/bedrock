//! AGENTS.md generation (SPINE §5, 0.4.0 base protocol).
//!
//! One artifact: the root AGENTS.md IS the compiled graph — a `#`-comment
//! preamble (machine stamp, how-to-read, identity title, the operating
//! lines, digest marker) followed by the complete TriG body. No prose
//! projection sections, no separate graph.trig: the file the harness
//! injects into every agent context is the graph itself, and C7 parse-back
//! reads that same file (the preamble is legal TriG comments).
//!
//! Matching is by IRI *local name* (last path segment), so it is robust to
//! the ontology base IRI the seed context chooses.

use oxrdf::{Quad, Term};
use std::collections::BTreeMap;

/// Predicate local names this generator understands.
fn local_name(iri: &str) -> &str {
    iri.rsplit(['#', '/']).next().unwrap_or(iri)
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

/// Render the single generated artifact: the root AGENTS.md — a `#`-comment
/// preamble (stamp, how-to-read, identity, operating lines, digest marker)
/// followed by the complete compiled TriG body. The whole file is valid
/// TriG (the preamble is comments), so C7 parse-back reads the same file
/// agents read, and the harness injects the graph itself into every
/// context window (SPINE §5, 0.4.0 base protocol).
pub fn generate_agents_md(repo_root: &std::path::Path, quads: &[Quad], trig_body: &str) -> String {
    let repo_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("repo")
        .to_string();

    // Group quads by subject (deterministic BTreeMap iteration) — the
    // identity vertex sources the title line.
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

    let mut identity = None::<(String, String)>; // (label, statement)
    for (id, vqs) in &by_subject {
        let role = literal_of(vqs, &["role"]);
        if role.as_deref() != Some("identity") {
            continue;
        }
        identity = Some((
            literal_of(vqs, &["label"]).unwrap_or_else(|| id.clone()),
            literal_of(vqs, &["statement", "description", "instruction"]).unwrap_or_default(),
        ));
    }

    let mut out = String::new();
    // Machine-owned stamp: this file is generated, never hand-edited.
    out.push_str(&crate::provenance::header(
        crate::provenance::Kind::Trig,
        env!("CARGO_PKG_VERSION"),
    ));
    out.push_str("\n#\n");
    out.push_str(
        "# This file IS the situation graph: the complete TriG compiled from situation/.\n",
    );
    out.push_str("# The harness injects it into every agent context. Each vertex carries its\n");
    out.push_str("# description and its relationships; follow `references`/`path` pointers into\n");
    out.push_str("# situation/ to read the full document. Humans wanting prose read situation/.\n");
    out.push_str("#\n");
    out.push_str("# NEVER hand-edit this file: it is built by `bedrock build`, and hand edits\n");
    out.push_str("# are detected and rejected by `bedrock check`. To change the graph, edit\n");
    out.push_str("# the situation/ vertices, then run `bedrock build` and commit the result.\n");
    out.push_str("#\n");
    // Title: repo name + identity statement.
    match &identity {
        Some((_, stmt)) if !stmt.trim().is_empty() => {
            out.push_str(&format!("# {} — {}\n", repo_name, stmt.trim()));
        }
        Some((label, _)) => out.push_str(&format!("# {} — {}\n", repo_name, label)),
        None => out.push_str(&format!("# {}\n", repo_name)),
    }
    out.push_str("#\n");
    // Operating this repository (base protocol). Static preamble lines —
    // `#` comments, so the file stays valid TriG end-to-end and renders as
    // markdown text. Carried by init/adopt/update into
    // situation/references/bedrock-operating.md (C10 guards it against
    // drift).
    out.push_str("# Operating this repository:\n");
    out.push_str("# - Work happens on verb-prefixed branches, one verb per state (these are branch names, not directories):\n");
    out.push_str("# - think/ — explore and decide.\n");
    out.push_str("# - plan/ — write the plan as a graph.\n");
    out.push_str("# - execute/ — do the work.\n");
    out.push_str("# - reflect/ — review after the fact.\n");
    out.push_str("# - deploy/ — place the result where it is recorded.\n");
    out.push_str("# - Authoring loop: write a vertex, `bedrock check`, `bedrock build`, commit source AND generated output, open a PR — a human merges.\n");
    out.push_str("# - The chain: every plan is a promise; its criteria are its oracle; its witnesses prove it held; its residual declares what was not assured.\n");
    out.push_str("# - Decisions are records too: why a design is what it is lives in record/ Decision vertices — walk `supersedes` chains before relitigating a choice; write one when you close a fork. Semantics: situation/references/bedrock-operating.md\n");
    out.push_str("# - The base protocol, in full — THE CHAIN, the ontology, every rule: situation/references/bedrock-operating.md\n");
    out.push_str("#\n");

    // Digest marker (comment): pins the graph body bytes; filled by the
    // caller via `stamp_digest`.
    out.push_str(&format!(
        "# bedrock {} digest __DIGEST__ (regenerated by `bedrock build`; hand edits are detected and rejected by `bedrock check`)\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push('\n');

    // The graph body: the complete compiled TriG. This is what agents read
    // and traverse; it is the file.
    out.push_str(trig_body);
    out
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
