//! Context registry and shared compiler constants.
//!
//! C3 requires each vertex's `@context` be either an inline object or one of
//! the two locally served bedrock context coordinates. `urn:bedrock:context/v1`
//! is canonical; `https://yeetz.dev/bedrock/context/v1` remains readable for
//! the 0.3 bridge. Remote context loading stays disabled (SPINE §4 C3, §5).
//!
//! The donor execution context is served only for the historical golden
//! fixtures. Consumer repositories cannot add remote contexts through this
//! registry.

use crate::errors::Fatal;
use serde_json::Value;
use std::collections::BTreeMap;

/// Canonical public Bedrock identity root (Mount Contract v1 §1).
pub const BEDROCK_IRI_BASE: &str = "urn:bedrock:";

/// Legacy private identity root accepted only by the 0.3+ read bridge.
pub const LEGACY_BEDROCK_IRI_BASE: &str = "https://yeetz.dev/bedrock/";

/// Canonical and legacy locally served context IRIs.
pub const BEDROCK_CONTEXT_IRI: &str = "urn:bedrock:context/v1";
pub const LEGACY_BEDROCK_CONTEXT_IRI: &str = "https://yeetz.dev/bedrock/context/v1";

/// The donor-era execution context IRI, served only for goldens.
pub const EXECUTION_CONTEXT_IRI: &str = "https://yeetz.dev/context/execution/v1";

/// Fixed base IRI used only to resolve relative `@id`-typed values.
pub const BASE_IRI: &str = "urn:bedrock:path/";

/// Canonical and legacy path-pointer namespaces.
pub const PATH_PREFIX: &str = "urn:bedrock:path/";
pub const LEGACY_PATH_PREFIX: &str = "https://yeetz.dev/bedrock/path/";

/// Canonical and legacy vertex namespaces.
pub const VERTEX_PREFIX: &str = "urn:bedrock:vertex/";
pub const LEGACY_VERTEX_PREFIX: &str = "https://yeetz.dev/bedrock/vertex/";

/// Decode either bridge-era path pointer into its repo-relative payload.
pub fn path_from_iri(iri: &str) -> Option<&str> {
    iri.strip_prefix(PATH_PREFIX)
        .or_else(|| iri.strip_prefix(LEGACY_PATH_PREFIX))
}

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

/// Canonical and legacy named-graph roots.
pub const GRAPH_PREFIX: &str = "urn:bedrock:graph/";
pub const LEGACY_GRAPH_PREFIX: &str = "https://yeetz.dev/graph/";

/// The canonical graph IRI a namespace's files land in.
pub fn namespace_graph(ns: &str) -> String {
    format!("{GRAPH_PREFIX}{ns}")
}

pub fn legacy_namespace_graph(ns: &str) -> String {
    format!("{LEGACY_GRAPH_PREFIX}{ns}")
}

pub fn is_namespace_graph(iri: &str, ns: &str) -> bool {
    iri == namespace_graph(ns) || iri == legacy_namespace_graph(ns)
}

pub fn is_any_namespace_graph(iri: &str) -> bool {
    GRAPH_NAMESPACES
        .iter()
        .any(|ns| is_namespace_graph(iri, ns))
}

/// Canonical and legacy ontology roots.
pub const ONTOLOGY_PREFIX: &str = "urn:bedrock:ontology/";
pub const LEGACY_ONTOLOGY_PREFIX: &str = "https://yeetz.dev/bedrock/ontology/";

/// Canonical and legacy predicate roots.
pub const PREDICATE_PREFIX: &str = BEDROCK_IRI_BASE;
pub const LEGACY_PREDICATE_PREFIX: &str = LEGACY_BEDROCK_IRI_BASE;

pub fn ontology_type(local: &str) -> String {
    format!("{ONTOLOGY_PREFIX}{local}")
}

pub fn legacy_ontology_type(local: &str) -> String {
    format!("{LEGACY_ONTOLOGY_PREFIX}{local}")
}

pub fn is_ontology_type(iri: &str, local: &str) -> bool {
    iri == ontology_type(local) || iri == legacy_ontology_type(local)
}

pub fn predicate(local: &str) -> String {
    format!("{PREDICATE_PREFIX}{local}")
}

pub fn legacy_predicate(local: &str) -> String {
    format!("{LEGACY_PREDICATE_PREFIX}{local}")
}

pub fn is_predicate(iri: &str, local: &str) -> bool {
    iri == predicate(local) || iri == legacy_predicate(local)
}

/// True for every Bedrock-owned IRI on either side of the read bridge.
pub fn is_bedrock_owned_iri(iri: &str) -> bool {
    iri.starts_with(BEDROCK_IRI_BASE)
        || iri.starts_with(LEGACY_BEDROCK_IRI_BASE)
        || iri.starts_with(LEGACY_GRAPH_PREFIX)
}

/// The twelve canonical base @types in the 0.7 mount protocol. Repository
/// archetypes ride alongside one; legacy spellings remain read-compatible.
pub const BASE_TYPES: [&str; 12] = [
    "urn:bedrock:ontology/Invariant",
    "urn:bedrock:ontology/Breadcrumb",
    "urn:bedrock:ontology/Term",
    "urn:bedrock:ontology/Identity",
    "urn:bedrock:ontology/SituationStructure",
    "urn:bedrock:ontology/Risk",
    "urn:bedrock:ontology/Plan",
    "urn:bedrock:ontology/EpochRecord",
    "urn:bedrock:ontology/DeployRecord",
    "urn:bedrock:ontology/ReflectVerdict",
    "urn:bedrock:ontology/Decision",
    "urn:bedrock:ontology/ExpansionMount",
];

pub const LEGACY_BASE_TYPES: [&str; 12] = [
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
    "https://yeetz.dev/bedrock/ontology/ExpansionMount",
];

pub fn is_base_type(iri: &str) -> bool {
    BASE_TYPES.contains(&iri) || LEGACY_BASE_TYPES.contains(&iri)
}

/// Base IRIs used by semantic checks and generated linkage.
pub const WITNESSES_IRI: &str = "urn:bedrock:witnesses";
pub const DISPOSITION_IRI: &str = "urn:bedrock:disposition";
pub const RDF_JSON_DATATYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON";
pub const ORACLE_IRI: &str = "urn:bedrock:oracle";

/// Machine-owned installed base files guarded by C10.
pub const OPERATING_REF_PATH: &str = "situation/references/bedrock-operating.md";
pub const SUBSTRATE_LOCK_PATH: &str = "seed/substrate-lock.json";

/// Fixed deterministic TriG prelude. oxttl orders by descending IRI length;
/// these four lengths are pairwise distinct, avoiding its unstable tie break.
pub const PREFIXES: [(&str, &str); 4] = [
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
    ("graph", "urn:bedrock:graph/"),
    ("bedrock", "urn:bedrock:"),
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

/// Derive the frozen legacy term map from the canonical map. The translation
/// is context-local: graph IRIs do not occur in this document.
fn legacy_bedrock_context() -> Value {
    fn translate(value: &mut Value) {
        match value {
            Value::String(s) => {
                if let Some(local) = s.strip_prefix(BEDROCK_IRI_BASE) {
                    *s = format!("{LEGACY_BEDROCK_IRI_BASE}{local}");
                }
            }
            Value::Array(values) => {
                for value in values {
                    translate(value);
                }
            }
            Value::Object(values) => {
                for value in values.values_mut() {
                    translate(value);
                }
            }
            _ => {}
        }
    }

    let mut context = bedrock_context();
    translate(&mut context);
    context
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
    /// Embedded-only registry: canonical and legacy Bedrock contexts plus the
    /// donor execution context used by historical goldens.
    pub fn embedded() -> Self {
        let mut r = ContextRegistry::default();
        r.served
            .insert(BEDROCK_CONTEXT_IRI.to_string(), bedrock_context());
        r.served.insert(
            LEGACY_BEDROCK_CONTEXT_IRI.to_string(),
            legacy_bedrock_context(),
        );
        r.served
            .insert(EXECUTION_CONTEXT_IRI.to_string(), donor_context());
        r
    }

    /// Build the registry, overlaying `<repo>/seed/context.yamlld` onto the
    /// matching bridge-era embedded map. A legacy installed context therefore
    /// keeps legacy compact-term expansion, while a refreshed context expands
    /// to canonical URNs. Extension-only maps overlay both families.
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
                let iri = doc.get("@id").and_then(Value::as_str).map(str::to_string);
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
        let has_urn = value_contains_prefix(&map_obj, BEDROCK_IRI_BASE);
        let has_legacy = value_contains_prefix(&map_obj, LEGACY_BEDROCK_IRI_BASE);
        if has_urn && has_legacy {
            return Err(Fatal(format!(
                "{}: seed context mixes canonical and legacy Bedrock identity bases",
                path.display()
            )));
        }
        match (has_urn, has_legacy) {
            (true, false) => overlay(&mut r, BEDROCK_CONTEXT_IRI, &map_obj)?,
            (false, true) => overlay(&mut r, LEGACY_BEDROCK_CONTEXT_IRI, &map_obj)?,
            (false, false) => {
                overlay(&mut r, BEDROCK_CONTEXT_IRI, &map_obj)?;
                overlay(&mut r, LEGACY_BEDROCK_CONTEXT_IRI, &map_obj)?;
            }
            (true, true) => unreachable!(),
        }
        if let Some(iri) = extra_iri
            && iri != BEDROCK_CONTEXT_IRI
            && iri != LEGACY_BEDROCK_CONTEXT_IRI
        {
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

fn value_contains_prefix(value: &Value, prefix: &str) -> bool {
    match value {
        Value::String(value) => value.starts_with(prefix),
        Value::Array(values) => values
            .iter()
            .any(|value| value_contains_prefix(value, prefix)),
        Value::Object(values) => values
            .values()
            .any(|value| value_contains_prefix(value, prefix)),
        _ => false,
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
