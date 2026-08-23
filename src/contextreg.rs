//! Context registry and shared compiler constants.
//!
//! C3 requires each vertex's `@context` be either the embedded repo-local
//! context (an inline object) or the exact IRI of the bedrock context
//! (`https://yeetz.dev/bedrock/context/v1`, shipped in `seed/context.yamlld`)
//! — remote context loading stays disabled (SPINE §4 C3, §5). The registry
//! defines exactly which IRIs are served locally; every other URL is refused
//! by the JSON-LD load callback.
//!
//! Served contexts:
//!   1. `https://yeetz.dev/bedrock/context/v1` — the bedrock context, from
//!      `src/embedded/bedrock-context.json` (W2's `seed/context.yamlld` map
//!      plus the four epoch-record terms `commit`/`version`/`mode`/`offline`
//!      that W2's map omits but `seed/schemas/record.json` requires — merged
//!      compiler-side; reported to W2). At runtime the repo's
//!      `seed/context.yamlld` overlays this map per term (seed authoritative).
//!   2. `https://yeetz.dev/context/execution/v1` — the donor-era execution
//!      context (spec/donor-*), served only so the golden donor fixtures
//!      compile; a real consumer repo never references it.

use crate::errors::Fatal;
use serde_json::Value;
use std::collections::BTreeMap;

/// The bedrock context IRI (W2 contract, shipped in `seed/context.yamlld`).
pub const BEDROCK_CONTEXT_IRI: &str = "https://yeetz.dev/bedrock/context/v1";

/// The donor-era execution context IRI, served only for goldens.
pub const EXECUTION_CONTEXT_IRI: &str = "https://yeetz.dev/context/execution/v1";

/// Base IRI used when expanding YAML-LD. All `@id`/`@type`/edge values are
/// absolute IRIs (enforced by C3/C5), so this only resolves *relative*
/// values of `@type: @id` context terms — which C5 then treats as
/// repo-relative paths. Keeping the base fixed (never a filesystem path)
/// preserves byte determinism (C6).
pub const BASE_IRI: &str = "https://yeetz.dev/bedrock/path/";

/// `https://yeetz.dev/bedrock/path/` — the namespace under which repo-path
/// pointers are expressed as absolute IRIs (W2 contract). C5 strips this
/// prefix to recover the repo-relative path; AGENTS.md "where things live"
/// reads the same predicate-objects.
pub const PATH_PREFIX: &str = "https://yeetz.dev/bedrock/path/";

/// `https://yeetz.dev/bedrock/vertex/` — the namespace of vertex @ids
/// (W2 contract, enforced by `seed/schemas/*.json` vertexId patterns).
pub const VERTEX_PREFIX: &str = "https://yeetz.dev/bedrock/vertex/";

/// The six situation namespaces (SPINE §3).
pub const NAMESPACES: [&str; 6] = [
    "definition",
    "architecture",
    "risk",
    "plan",
    "record",
    "references",
];

/// The five namespaces that own named graphs (all but `references`, whose
/// files point at content and do not own a graph). C3 named-graph membership
/// is restricted to `GRAPH(<ns>)` for these five.
pub const GRAPH_NAMESPACES: [&str; 5] = ["definition", "architecture", "risk", "plan", "record"];

/// The graph IRI a namespace's files land in: `https://yeetz.dev/graph/<ns>`.
pub fn namespace_graph(ns: &str) -> String {
    format!("https://yeetz.dev/graph/{ns}")
}

/// The bedrock ontology base: `https://yeetz.dev/bedrock/ontology/`.
pub const ONTOLOGY_PREFIX: &str = "https://yeetz.dev/bedrock/ontology/";

/// The bedrock predicate base: `https://yeetz.dev/bedrock/`.
pub const PREDICATE_PREFIX: &str = "https://yeetz.dev/bedrock/";

pub fn ontology_type(local: &str) -> String {
    format!("{ONTOLOGY_PREFIX}{local}")
}

pub fn predicate(local: &str) -> String {
    format!("{PREDICATE_PREFIX}{local}")
}

/// The closed set of base @types (0.2.0 base protocol, operating reference
/// §"Base ontology"): every vertex's `@type` must intersect this set — rule
/// C9. Repo-specific archetypes ride alongside one of these in the same
/// `@type` array; they never stand alone and never redefine a base term.
pub const BASE_TYPES: [&str; 11] = [
    "https://yeetz.dev/bedrock/ontology/Invariant",
    "https://yeetz.dev/bedrock/ontology/Breadcrumb",
    "https://yeetz.dev/bedrock/ontology/Term",
    "https://yeetz.dev/bedrock/ontology/Identity",
    "https://yeetz.dev/bedrock/ontology/SituationStructure",
    "https://yeetz.dev/bedrock/ontology/Risk",
    "https://yeetz.dev/bedrock/ontology/Plan",
    "https://yeetz.dev/bedrock/ontology/EpochRecord",
    "https://yeetz.dev/bedrock/ontology/DeployRecord",
    "https://yeetz.dev/bedrock/ontology/ReflectVerdict",
    "https://yeetz.dev/bedrock/ontology/Decision",
];

/// Base IRIs used by check rules C8 (witness gate) and C10 (digest skew).
pub const WITNESSES_IRI: &str = "https://yeetz.dev/bedrock/witnesses";
pub const DISPOSITION_IRI: &str = "https://yeetz.dev/bedrock/disposition";
pub const RDF_JSON_DATATYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON";
pub const ORACLE_IRI: &str = "https://yeetz.dev/bedrock/oracle";

/// The installed base files `bedrock update` refreshes from the binary's
/// embedded copies, checked by C10: `seed/schemas/*.json`, `seed/context.yamlld`,
/// and the operating reference at `situation/references/bedrock-operating.md`.
pub const OPERATING_REF_PATH: &str = "situation/references/bedrock-operating.md";

/// The fixed, deterministic `@prefix` prelude for TriG output (SPINE §5:
/// "sorted @prefix prelude"). The oxttl serializer orders prefixes by
/// *descending IRI length*; all four lengths below are distinct so the
/// unstable tie-break can never affect output bytes (C6).
pub const PREFIXES: [(&str, &str); 4] = [
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
    ("bedrock", "https://yeetz.dev/bedrock/"),
    ("graph", "https://yeetz.dev/graph/"),
];

/// A mapping of context IRI → full context document (`{"@context": {...}}`).
#[derive(Default, Clone)]
pub struct ContextRegistry {
    served: BTreeMap<String, Value>,
}

/// The embedded bedrock context document (W2's map + epoch overlays).
fn bedrock_context() -> Value {
    serde_json::from_str(include_str!("embedded/bedrock-context.json"))
        .expect("static embedded bedrock context")
}

/// The donor-derived execution context: defines every term appearing in
/// spec/donor-fixtures and spec/donor-execution-record.yamlld so goldens
/// compile into meaningful quads rather than silently dropping terms.
fn donor_context() -> Value {
    serde_json::from_str(
        r#"{
  "@context": {
    "sequence": "https://yeetz.dev/ontology/sequence",
    "executionId": "https://yeetz.dev/ontology/execution-id",
    "repository": "https://yeetz.dev/ontology/repository",
    "sourceRevision": "https://yeetz.dev/ontology/source-revision",
    "schemaVersion": "https://yeetz.dev/ontology/schema-version",
    "artifactKind": "https://yeetz.dev/ontology/artifact-kind",
    "revisionScope": "https://yeetz.dev/ontology/revision-scope",
    "gitCommit": "https://yeetz.dev/ontology/git-commit",
    "sha256": "https://yeetz.dev/ontology/sha256",
    "path": "https://yeetz.dev/ontology/path",
    "intent": "https://yeetz.dev/ontology/intent",
    "outcomeSummary": "https://yeetz.dev/ontology/outcome-summary",
    "acceptanceCriteria": { "@id": "https://yeetz.dev/ontology/acceptance-criteria", "@container": "@set" },
    "actor": { "@id": "https://yeetz.dev/ontology/actor", "@type": "@id" },
    "causalClass": { "@id": "https://yeetz.dev/ontology/causal-class", "@type": "@id" },
    "targetClass": { "@id": "https://yeetz.dev/ontology/target-class", "@type": "@id" },
    "lane": { "@id": "https://yeetz.dev/ontology/lane", "@type": "@id" },
    "previous": { "@id": "https://yeetz.dev/ontology/previous", "@type": "@id" },
    "instruction": { "@id": "https://yeetz.dev/ontology/instruction", "@type": "@id" },
    "opening": { "@id": "https://yeetz.dev/ontology/opening", "@type": "@id" },
    "candidate": { "@id": "https://yeetz.dev/ontology/candidate", "@type": "@id" },
    "validationReport": { "@id": "https://yeetz.dev/ontology/validation-report", "@type": "@id" },
    "disposition": { "@id": "https://yeetz.dev/ontology/disposition", "@type": "@id" },
    "outcome": { "@id": "https://yeetz.dev/ontology/outcome", "@type": "@id" },
    "admittedActs": { "@id": "https://yeetz.dev/ontology/admitted-acts", "@type": "@id", "@container": "@set" },
    "consumes": { "@id": "https://yeetz.dev/ontology/consumes", "@type": "@id", "@container": "@set" },
    "requiredTests": { "@id": "https://yeetz.dev/ontology/required-tests", "@type": "@id", "@container": "@set" },
    "references": { "@id": "https://yeetz.dev/ontology/references", "@type": "@id", "@container": "@set" },
    "evidence": { "@id": "https://yeetz.dev/ontology/evidence", "@type": "@id", "@container": "@set" },
    "subjectCandidate": { "@id": "https://yeetz.dev/ontology/subject-candidate", "@type": "@id" }
  }
}"#,
    )
    .expect("static embedded context")
}

impl ContextRegistry {
    /// Embedded-only registry (no seed checkout): serves the bedrock context
    /// and the donor execution context.
    pub fn embedded() -> Self {
        let mut r = ContextRegistry::default();
        r.served
            .insert(BEDROCK_CONTEXT_IRI.to_string(), bedrock_context());
        r.served
            .insert(EXECUTION_CONTEXT_IRI.to_string(), donor_context());
        r
    }

    /// Build the registry, overlaying any context present in
    /// `<repo>/seed/context.yamlld` onto the embedded bedrock map (seed terms
    /// are authoritative per key; embedded-only epoch terms survive).
    ///
    /// Accepted authoring shapes (W2 ships the bare `@context` map form):
    ///   - a bare `@context` term map `{term: def, ...}`
    ///   - `{"@id": "<iri>", "@context": {...}}` — served under `<iri>` too
    ///
    /// A malformed file is a hard error (loud, never silently skipped).
    pub fn load(seed_dir: Option<&std::path::Path>) -> Result<Self, Fatal> {
        let mut r = Self::embedded();
        let Some(seed) = seed_dir else {
            return Ok(r);
        };
        let path = seed.join("context.yamlld");
        if !path.exists() {
            return Ok(r);
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Fatal(format!("failed to read {}: {e}", path.display())))?;
        // Strip comment lines (the seed context is YAML, not a vertex).
        let yaml_text: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        let doc: Value = serde_norway::from_str(&yaml_text).map_err(|e| {
            Fatal(format!(
                "{}: seed context is not valid YAML: {e}",
                path.display()
            ))
        })?;

        let (extra_iri, map) = match doc.get("@context") {
            Some(Value::Object(_)) => {
                let iri = doc
                    .get("@id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (iri, doc)
            }
            Some(_) => {
                return Err(Fatal(format!(
                    "{}: seed context `@context` must be an object",
                    path.display()
                )));
            }
            None => (None, doc),
        };

        let map_obj = map.get("@context").cloned().unwrap_or_else(|| map.clone());
        overlay(&mut r, BEDROCK_CONTEXT_IRI, &map_obj)?;
        if let Some(iri) = extra_iri {
            r.served.entry(iri).or_insert_with(|| map.clone());
        }
        Ok(r)
    }

    /// Look up the full context document served for `iri`.
    pub fn get(&self, iri: &str) -> Option<&Value> {
        self.served.get(iri)
    }

    /// True if `iri` is served locally by bedrock (seed or embedded).
    pub fn serves(&self, iri: &str) -> bool {
        self.served.contains_key(iri)
    }
}

/// Overlay a context term map onto the served document for `iri`.
fn overlay(reg: &mut ContextRegistry, iri: &str, map: &Value) -> Result<(), Fatal> {
    let Some(map_obj) = map.as_object() else {
        return Err(Fatal(format!(
            "seed context must resolve to a term map (object), got: {map}"
        )));
    };
    let doc = reg
        .served
        .get_mut(iri)
        .ok_or_else(|| Fatal(format!("internal: context {iri} not registered")))?;
    let base = doc
        .get_mut("@context")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| Fatal(format!("internal: context {iri} is not a map")))?;
    for (k, v) in map_obj {
        base.insert(k.clone(), v.clone());
    }
    Ok(())
}
