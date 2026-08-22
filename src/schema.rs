//! C4: namespace schema validation.
//!
//! Each namespace (definition|architecture|risk|plan|record) has a JSON
//! Schema in `seed/schemas/<ns>.json` (authored by W2; installed into
//! consumers by init/adopt). Every vertex must validate against its
//! namespace schema. A missing schema is a hard, loud error — never a
//! silently skipped rule.

use crate::errors::{Fatal, Violation};
use jsonschema::Validator;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub struct SchemaRegistry {
    validators: BTreeMap<String, Validator>,
}

impl SchemaRegistry {
    /// Load + compile the five namespace schemas from `seed/schemas/`.
    /// `seed_dir` absent → error (a fixable missing prerequisite, reported
    /// loudly per SPINE §4's "brittleness with intent").
    pub fn load(seed_dir: Option<&Path>) -> Result<Self, Fatal> {
        let mut validators = BTreeMap::new();
        let Some(seed) = seed_dir else {
            return Err(Fatal(
                "no seed/ directory found; C4 namespace schemas live in seed/schemas/ (SPINE §4). \
                 Install seed/ (bedrock init) or point BEDROCK_SEED at it"
                    .to_string(),
            ));
        };
        let schemas = seed.join("schemas");
        if !schemas.is_dir() {
            return Err(Fatal(format!(
                "{}: expected namespace schemas (SPINE §4 C4); create or install seed/",
                schemas.display()
            )));
        }
        for ns in crate::contextreg::GRAPH_NAMESPACES {
            let path = schemas.join(format!("{ns}.json"));
            let text = std::fs::read_to_string(&path).map_err(|e| {
                Fatal(format!(
                    "{}: cannot load namespace schema for `{ns}` (SPINE §4 C4): {e}",
                    path.display()
                ))
            })?;
            let schema: Value = serde_json::from_str(&text).map_err(|e| {
                Fatal(format!(
                    "{}: namespace schema for `{ns}` is not valid JSON: {e}",
                    path.display()
                ))
            })?;
            let validator = jsonschema::validator_for(&schema).map_err(|e| {
                Fatal(format!(
                    "{}: namespace schema for `{ns}` failed to compile: {e}",
                    path.display()
                ))
            })?;
            validators.insert(ns.to_string(), validator);
        }
        Ok(SchemaRegistry { validators })
    }

    /// Validate one vertex (relative path `rel_path`, source text `src`)
    /// against its namespace schema. `None` namespace (references/) is not
    /// schema-governed.
    pub fn validate(
        &self,
        ns: Option<&str>,
        instance: &Value,
        rel_path: &str,
        src: &str,
    ) -> Vec<Violation> {
        let Some(ns) = ns else {
            return Vec::new();
        };
        let Some(validator) = self.validators.get(ns) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for err in validator.iter_errors(instance) {
            let path = err.instance_path().to_string();
            let msg = err.to_string();
            // Locate the failing line if the value appears in the source
            // (attribute greedily: the deepest path segment often appears).
            let needle = path
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("@id");
            out.push(Violation::new(
                "C4",
                rel_path,
                crate::errors::line_of(src, needle),
                format!("schema violation at {path}: {msg}"),
            ));
        }
        out
    }
}
