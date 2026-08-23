//! The §5 compile pipeline and the per-file rules it implements.
//!
//! ```text
//! YAML (serde_norway) → serde_json::Value
//!   → oxjsonld (remote loading disabled) → oxrdf Quads
//!   → deterministic sort (subject IRI, predicate IRI, object, graph)
//!   → oxttl TriGSerializer (fixed @prefix prelude)
//!   → parse-back equivalence (C7)
//! ```
//!
//! C2 (YAML + anchors/aliases/merge-keys) is enforced here; C3's LD profile
//! and graph-membership checks live here; C7 parse-back here. C1/C4/C5/C6
//! are orchestrated by `check`.

use crate::contextreg::{ContextRegistry, GRAPH_NAMESPACES, PREFIXES};
use crate::errors::Violation;
use crate::yamlsyntax::{self, YamlViolation};
use oxjsonld::{JsonLdParser, JsonLdRemoteDocument};
use oxrdf::{GraphName, NamedNode, Quad};
use serde_json::Value;

/// C2: parse `text` as YAML 1.2 and reject anchors/aliases/merge keys.
///
/// Returns the parsed document plus any C2 violations. The document is
/// returned even when violations exist so callers can decide how to proceed
/// (check reports all C2 findings; a failed parse produces none).
pub struct YamlParse {
    pub value: Option<Value>,
    pub violations: Vec<Violation>,
}

/// Parse + syntax-scan. `rel_path` is the repo-relative path for messages.
pub fn parse_yaml(text: &str, rel_path: &str) -> YamlParse {
    let mut violations = Vec::new();

    // Syntax-level anchor/alias/merge-key scan (C2). Runs on the raw bytes;
    // the tokenizer treats block-scalar content as opaque, so markdown `*`
    // emphasis in statements can never false-positive.
    for v in yamlsyntax::scan_forbidden(text.as_bytes()) {
        violations.push(match v {
            YamlViolation::Anchor { line, name } => Violation::new(
                "C2",
                rel_path,
                line,
                format!("YAML anchors are forbidden (found `&{name}`)"),
            ),
            YamlViolation::Alias { line, name } => Violation::new(
                "C2",
                rel_path,
                line,
                format!("YAML aliases are forbidden (found `*{name}`)"),
            ),
            YamlViolation::MergeKey { line } => Violation::new(
                "C2",
                rel_path,
                line,
                "YAML merge keys (`<<`) are forbidden".to_string(),
            ),
            YamlViolation::Tokens { line, message } => {
                Violation::new("C2", rel_path, line, format!("YAML scanner: {message}"))
            }
        });
    }

    if violations
        .iter()
        .any(|v| v.message.starts_with("YAML scanner"))
    {
        return YamlParse {
            value: None,
            violations,
        };
    }

    let value: Result<Value, _> = serde_norway::from_str(text);
    match value {
        Ok(v) => YamlParse {
            value: Some(v),
            violations,
        },
        Err(e) => {
            let line = error_line(&e.to_string());
            violations.push(Violation::new(
                "C2",
                rel_path,
                line,
                format!("does not parse as YAML 1.2: {e}"),
            ));
            YamlParse {
                value: None,
                violations,
            }
        }
    }
}

fn error_line(msg: &str) -> u32 {
    // serde_norway errors name the position as "at line N column M".
    for tok in msg.split_whitespace() {
        if let Some(rest) = tok.strip_prefix("line") {
            if let Ok(n) = rest.trim_end_matches(',').parse::<u32>() {
                return n;
            }
        }
    }
    1
}

/// C3: LD profile on the parsed document, before expansion.
///
/// Enforces: exactly one `@context` (inline object or a served bedrock IRI);
/// all `@id`/`@type` are absolute IRIs; no blank-node (`_:`) ids.
pub fn ld_profile(
    value: &Value,
    rel_path: &str,
    src: &str,
    registry: &ContextRegistry,
) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(root) = value.as_object() else {
        out.push(Violation::new(
            "C3",
            rel_path,
            1,
            "vertex must be a YAML mapping (object)".to_string(),
        ));
        return out;
    };

    // @context must be present and singular.
    match root.get("@context") {
        None => out.push(Violation::new(
            "C3",
            rel_path,
            line_of(src, "@context"),
            "missing @context: exactly one required (inline object or bedrock context IRI)"
                .to_string(),
        )),
        Some(Value::String(iri)) => {
            if !is_absolute_iri(iri) {
                out.push(Violation::new(
                    "C3",
                    rel_path,
                    line_of(src, iri),
                    format!("@context is not an absolute IRI: {iri}"),
                ));
            } else if !registry.serves(iri) {
                out.push(Violation::new(
                    "C3",
                    rel_path,
                    line_of(src, iri),
                    format!("remote context loading is disabled and {iri} is not a locally served bedrock context"),
                ));
            }
        }
        Some(Value::Object(_)) => {} // embedded repo-local context
        Some(_) => out.push(Violation::new(
            "C3",
            rel_path,
            line_of(src, "@context"),
            "@context must be a single inline object or the bedrock context IRI".to_string(),
        )),
    }

    // Absolute @id / @type, no blank-node ids, everywhere.
    walk_profile(value, rel_path, src, &mut out);
    out
}

fn walk_profile(node: &Value, rel_path: &str, src: &str, out: &mut Vec<Violation>) {
    match node {
        Value::Object(map) => {
            for (k, v) in map {
                match k.as_str() {
                    "@id" => {
                        if let Some(s) = v.as_str() {
                            if s.starts_with("_:") {
                                out.push(Violation::new(
                                    "C3",
                                    rel_path,
                                    line_of(src, s),
                                    "blank node ids (`_:`) are forbidden".to_string(),
                                ));
                            } else if !is_absolute_iri(s) {
                                out.push(Violation::new(
                                    "C3",
                                    rel_path,
                                    line_of(src, s),
                                    format!("@id must be an absolute IRI, got: {s}"),
                                ));
                            }
                        }
                    }
                    "@type" => {
                        // @type may be a string or array of strings.
                        let vals: Vec<Option<&str>> = match v {
                            Value::String(s) => vec![Some(s.as_str())],
                            Value::Array(a) => a.iter().map(|x| x.as_str()).collect(),
                            _ => vec![None],
                        };
                        for s in vals.into_iter().flatten() {
                            if s.starts_with("_:") {
                                out.push(Violation::new(
                                    "C3",
                                    rel_path,
                                    line_of(src, s),
                                    "blank node ids (`_:`) are forbidden in @type".to_string(),
                                ));
                            } else if !is_absolute_iri(s) {
                                out.push(Violation::new(
                                    "C3",
                                    rel_path,
                                    line_of(src, s),
                                    format!("@type must be an absolute IRI, got: {s}"),
                                ));
                            }
                        }
                    }
                    _ => walk_profile(v, rel_path, src, out),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_profile(item, rel_path, src, out);
            }
        }
        _ => {}
    }
}

/// True for `scheme:...` strings (constrained profile: no spaces/controls).
pub fn is_absolute_iri(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    let mut seen_colon_scheme = false;
    for c in chars {
        if c == ':' {
            seen_colon_scheme = true;
            break;
        }
        if !(c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
            return false;
        }
    }
    if !seen_colon_scheme {
        return false;
    }
    // Value after the scheme must not contain whitespace or control chars.
    !s.chars().any(|c| c.is_whitespace() || (c as u32) < 0x20)
}

/// Expand a parsed document into quads via oxjsonld, with remote context
/// loading disabled: the only IRIs served come from `registry`.
///
/// Returns quads with blank nodes rejected (C3). No graph remapping is done
/// here — callers apply `remap_graphs` for namespace membership.
pub fn expand(
    value: &Value,
    rel_path: &str,
    src: &str,
    registry: &ContextRegistry,
) -> Result<Vec<Quad>, Vec<Violation>> {
    let json = serde_json::to_string(value).expect("Value serializes");
    let registry = registry.clone(); // closure must be 'static
    let parser = JsonLdParser::new()
        .with_base_iri(crate::contextreg::BASE_IRI)
        .map_err(|e| {
            vec![Violation::new(
                "C3",
                rel_path,
                1,
                format!("invalid base IRI: {e}"),
            )]
        })?
        .for_slice(json.as_bytes())
        .with_load_document_callback(move |url, _opts| match registry.get(url) {
            Some(doc) => Ok(JsonLdRemoteDocument {
                document: serde_json::to_vec(doc).expect("context serializes"),
                document_url: url.to_string(),
            }),
            None => {
                Err(format!("remote context loading disabled; {url} not served locally").into())
            }
        });

    let mut quads = Vec::new();
    let mut violations = Vec::new();
    for q in parser {
        match q {
            Ok(quad) => quads.push(quad),
            Err(e) => {
                violations.push(Violation::new(
                    "C3",
                    rel_path,
                    line_of(src, "@id"),
                    format!("JSON-LD expansion failed: {e}"),
                ));
                break;
            }
        }
    }
    if !violations.is_empty() {
        return Err(violations);
    }

    // C3: no blank nodes, anywhere.
    let mut bnode = Vec::new();
    for q in &quads {
        if matches!(q.subject, oxrdf::NamedOrBlankNode::BlankNode(_))
            || matches!(q.object, oxrdf::Term::BlankNode(_))
            || matches!(q.graph_name, GraphName::BlankNode(_))
        {
            bnode.push(format!("{} → {}", q.subject, q.predicate));
        }
    }
    if !bnode.is_empty() {
        let mut msg = String::from(
            "blank nodes are forbidden (a vertex object without an absolute @id produces one): ",
        );
        msg.push_str(&bnode.join("; "));
        return Err(vec![Violation::new(
            "C3",
            rel_path,
            line_of(src, "@type"),
            msg,
        )]);
    }

    Ok(quads)
}

/// C3: named-graph membership — a file's quads may only live in its own
/// namespace graph (default-graph quads are folded in); named graphs are
/// otherwise restricted to the five namespace IRIs.
///
/// `ns` is the file's namespace (`""`/`None` for non-namespace files such as
/// `references/**` or golden fixtures, which compile into the default graph).
pub fn remap_graphs(
    quads: Vec<Quad>,
    ns: Option<&str>,
    rel_path: &str,
    src: &str,
) -> Result<Vec<Quad>, Vec<Violation>> {
    let own = ns.map(crate::contextreg::namespace_graph);
    let mut out = Vec::with_capacity(quads.len());
    let mut violation: Option<Violation> = None;

    for q in quads {
        match &q.graph_name {
            GraphName::DefaultGraph => {
                let mut q = q;
                if let Some(own) = &own {
                    q.graph_name = NamedNode::new(own.clone())
                        .expect("namespace graph IRI is valid")
                        .into();
                }
                out.push(q);
            }
            GraphName::NamedNode(g) => {
                let g_str = g.to_string();
                let in_own = own.as_deref() == Some(g_str.as_str());
                let in_any = GRAPH_NAMESPACES
                    .iter()
                    .any(|n| crate::contextreg::namespace_graph(n) == g_str);
                if !in_any {
                    if violation.is_none() {
                        violation = Some(Violation::new(
                            "C3",
                            rel_path,
                            line_of(src, g.as_str()),
                            format!(
                                "named graph membership is restricted to the namespace directory (definition|architecture|risk|plan|record); got graph {g_str}"
                            ),
                        ));
                    }
                } else if !in_own {
                    if violation.is_none() {
                        violation = Some(Violation::new(
                            "C3",
                            rel_path,
                            line_of(src, g.as_str()),
                            format!(
                                "file graphs into {g_str} but its own namespace graph is {}",
                                own.as_deref().unwrap_or("<none>")
                            ),
                        ));
                    }
                } else {
                    out.push(q);
                }
            }
            GraphName::BlankNode(_) => {
                if violation.is_none() {
                    violation = Some(Violation::new(
                        "C3",
                        rel_path,
                        line_of(src, "@type"),
                        "blank-node graph names are forbidden".to_string(),
                    ));
                }
            }
        }
    }

    if let Some(v) = violation {
        return Err(vec![v]);
    }
    Ok(out)
}

/// Deterministic sort per SPINE §5: (subject IRI, predicate IRI, object,
/// graph). Keys are canonical N-Quads-style renderings of each term.
pub fn sort_quads(mut quads: Vec<Quad>) -> Vec<Quad> {
    quads.sort_by_key(quad_key);
    quads
}

fn quad_key(q: &Quad) -> (String, String, String, String) {
    (
        q.subject.to_string(),
        q.predicate.to_string(),
        q.object.to_string(),
        q.graph_name.to_string(),
    )
}

/// Serialize sorted quads to TriG with the fixed, deterministic prefix
/// prelude (SPINE §5: oxttl TriGSerializer).
pub fn serialize_trig(quads: &[Quad]) -> Result<Vec<u8>, String> {
    let mut serializer = oxttl::TriGSerializer::new();
    for (name, iri) in PREFIXES {
        serializer = serializer
            .with_prefix(name, iri)
            .map_err(|e| format!("invalid prefix {name}: {e}"))?;
    }
    let mut writer = serializer.for_writer(Vec::new());
    for q in quads {
        writer
            .serialize_quad(q.as_ref())
            .map_err(|e| format!("TriG write failed: {e}"))?;
    }
    writer
        .finish()
        .map_err(|e| format!("TriG finish failed: {e}"))
}

/// C7: re-parse emitted TriG and return the quads it encodes.
pub fn parse_back(bytes: &[u8], rel: &str) -> Result<Vec<Quad>, Violation> {
    let mut quads = Vec::new();
    for q in oxttl::TriGParser::new().for_slice(bytes) {
        match q {
            Ok(quad) => quads.push(quad),
            Err(e) => {
                return Err(Violation::new(
                    "C7",
                    rel,
                    1,
                    format!("emitted TriG does not re-parse: {e}"),
                ));
            }
        }
    }
    Ok(quads)
}

/// C7 gate: the emitted TriG must decode back to the exact dataset compiled
/// from source (multiset equality; blank nodes are already excluded). None
/// when the gate passes, else a C7 violation.
pub fn verify_parseback(compiled: &[Quad], trig_bytes: &[u8]) -> Option<Violation> {
    match parse_back(trig_bytes, "AGENTS.md") {
        Ok(back) => {
            if sort_quads(back) != sort_quads(compiled.to_vec()) {
                Some(Violation::new(
                    "C7",
                    "AGENTS.md",
                    1,
                    "parse-back dataset differs from the compiled dataset".to_string(),
                ))
            } else {
                None
            }
        }
        Err(v) => Some(v),
    }
}

/// Find the 1-based line containing `needle`, else line 1.
fn line_of(text: &str, needle: &str) -> u32 {
    for (i, line) in text.lines().enumerate() {
        if line.contains(needle) {
            return i as u32 + 1;
        }
    }
    1
}

/// Render a human path pointer from a bedrock path IRI: strip the
/// `https://yeetz.dev/bedrock/path/` prefix (W2 contract) to recover the
/// repo-relative path; non-path IRIs pass through unchanged.
pub fn path_pointer_of(iri: &str) -> String {
    iri.strip_prefix(crate::contextreg::PATH_PREFIX)
        .map(|p| p.to_string())
        .unwrap_or_else(|| iri.to_string())
}
