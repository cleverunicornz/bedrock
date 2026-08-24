//! Golden pipeline tests: the spec donor fixtures and the donor execution
//! record must compile through the §5 pipeline — YAML → JSON-LD expansion
//! (remote loading disabled, embedded bedrock context) → quads → sort →
//! TriG → parse-back equivalence.

use bedrock::compile;
use bedrock::contextreg::ContextRegistry;

mod common;
use common::{donor_execution_record, donor_fixtures_dir, manifest};

/// Run the full pipeline on one YAML-LD text; panics with details on failure.
fn pipeline_golden(rel: &str, text: &str) {
    let parsed = compile::parse_yaml(text, rel);
    let value = match &parsed.value {
        Some(v) => v,
        None => panic!(
            "golden {rel}: C2 parse failed:\n{}",
            parsed
                .violations
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        ),
    };
    let registry = ContextRegistry::embedded();
    let profile = compile::ld_profile(value, rel, text, &registry);
    assert!(
        profile.is_empty(),
        "golden {rel}: C3 profile violations:\n{}",
        profile
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let quads = compile::expand(value, rel, text, &registry)
        .unwrap_or_else(|vs| panic!("golden {rel}: expansion failed:\n{vs:?}"));
    assert!(!quads.is_empty(), "golden {rel}: produced no quads");
    let sorted = compile::sort_quads(quads);
    let trig = compile::serialize_trig(&sorted).expect("TriG serializes");
    let mismatch = compile::verify_parseback(&sorted, &trig);
    assert!(
        mismatch.is_none(),
        "golden {rel}: C7 parse-back mismatch: {mismatch:?}"
    );
    // Determinism: serialize twice → same bytes.
    let again = compile::serialize_trig(&sorted).expect("TriG serializes");
    assert_eq!(trig, again, "golden {rel}: non-deterministic output");
    eprintln!("golden {rel}: {} quads, {} bytes", sorted.len(), trig.len());
}

#[test]
fn donor_valid_fixtures_compile() {
    let valid = donor_fixtures_dir().join("valid");
    let mut files: Vec<_> = std::fs::read_dir(&valid)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "yamlld").unwrap_or(false))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    for p in files {
        let text = std::fs::read_to_string(&p).unwrap();
        let rel = p
            .strip_prefix(manifest())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        pipeline_golden(&rel, &text);
    }
}

#[test]
fn donor_execution_record_compiles() {
    let text = std::fs::read_to_string(donor_execution_record()).unwrap();
    pipeline_golden("spec/donor-execution-record.yamlld", &text);
}

#[test]
fn donor_control_files_are_not_yamlld() {
    // recovery.yaml / invalid/controls.yaml are control specs, not YAML-LD:
    // they must be rejected by the LD profile (no @context, root not a
    // vertex) rather than silently "compile" to an empty graph.
    for rel in [
        "spec/donor-fixtures/recovery.yaml",
        "spec/donor-fixtures/invalid/controls.yaml",
    ] {
        let p = manifest().join(rel);
        let text = std::fs::read_to_string(&p).unwrap();
        let parsed = compile::parse_yaml(&text, rel);
        assert!(
            parsed.value.is_some(),
            "{rel}: should parse as YAML (it is a control file), got {:?}",
            parsed.violations
        );
        let registry = ContextRegistry::embedded();
        let v = parsed.value.expect("parsed");
        let profile = compile::ld_profile(&v, rel, &text, &registry);
        // The root is not a mapping with @context → at least one C3 finding.
        assert!(
            profile.iter().any(|x| x.rule == "C3"),
            "{rel}: control file must fail the YAML-LD profile, got no C3 violations"
        );
    }
}

#[test]
fn empty_document_fails_profile() {
    // A purely empty file must be a loud C2/C3 failure, never a silent pass.
    let parsed = compile::parse_yaml("", "situation/definition/empty.yamlld");
    let mut vs = parsed.violations.clone();
    if let Some(v) = &parsed.value {
        // Empty YAML may parse to `null`; the LD profile must reject it.
        let registry = ContextRegistry::embedded();
        vs.extend(compile::ld_profile(
            v,
            "situation/definition/empty.yamlld",
            "",
            &registry,
        ));
    }
    assert!(!vs.is_empty(), "empty file must produce a violation");
}

#[test]
fn trig_prefix_iris_are_canonical_and_pairwise_length_distinct() {
    let prefixes = bedrock::contextreg::PREFIXES;
    let lengths: std::collections::BTreeSet<usize> =
        prefixes.iter().map(|(_, iri)| iri.len()).collect();
    assert_eq!(
        lengths.len(),
        prefixes.len(),
        "oxttl's equal-length prefix tie break would make bytes unstable"
    );
    assert!(prefixes.contains(&("bedrock", "urn:bedrock:")));
    assert!(prefixes.contains(&("graph", "urn:bedrock:graph/")));
}
