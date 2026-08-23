//! Both-polarity rule tests for C1–C7 (SPINE §4), each proving the failing
//! and passing fixtures for the rule, plus init/adopt behavior tests.

use bedrock::compile;

mod common;
use common::{Scratch, build_and_check_ok, manifest, materialize, run};

fn check_fails_with(s: &Scratch, rule: &str) -> String {
    let (c, out, err) = run(&["check", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 1, "check must fail with exit 1 for {rule}\n{combined}");
    assert!(
        combined.contains(rule),
        "expected a `{rule}` violation line, got:\n{combined}"
    );
    combined
}

#[test]
fn c1_placement_polarity() {
    let good = materialize("C1/good");
    build_and_check_ok(&good);

    let bad = materialize("C1/bad");
    let combined = check_fails_with(&bad, "C1");
    // Seventh situation namespace dir.
    assert!(
        combined.contains("situation/extra-dir"),
        "extra-dir must be named"
    );
    // Nested AGENTS.md outside root.
    assert!(
        combined.contains("AGENTS.md") && combined.contains("outside the repo root"),
        "nested AGENTS.md must be named: {combined}"
    );
}

#[test]
fn c2_anchor_alias_merge_polarity() {
    // Good: markdown emphasis in block scalars is legal.
    let good = materialize("C2/good");
    build_and_check_ok(&good);

    // Bad: anchor + alias + merge key.
    let bad = materialize("C2/bad");
    let combined = check_fails_with(&bad, "C2");
    assert!(combined.contains("anchor"), "{combined}");
    assert!(combined.contains("alias"), "{combined}");
    assert!(combined.contains("merge key"), "{combined}");
}

#[test]
fn c3_profile_polarity() {
    let good = materialize("C3/good");
    build_and_check_ok(&good);

    // Bad fixture carries both a `_:` blank @id (definition) and a named
    // graph outside the five namespace graphs (record).
    let bad = materialize("C3/bad");
    let combined = check_fails_with(&bad, "C3");
    assert!(combined.contains("blank node"), "{combined}");
    assert!(combined.contains("named graph membership"), "{combined}");
}

#[test]
fn c4_schema_polarity() {
    let good = materialize("C4/good");
    build_and_check_ok(&good);

    let bad = materialize("C4/bad");
    let combined = check_fails_with(&bad, "C4");
    assert!(combined.contains("required"), "{combined}");
}

#[test]
fn c4_vertex_id_base_polarity() {
    // Defect-2 regression: the namespace schemas accept a CONSUMER repo base
    // (https://yeetz.dev/<repo>/vertex/<slug>) for situated vertices, while
    // floor content stays bedrock-namespaced by law.
    // Pass pole: situated vertices under a repo IRI base (architecture and
    // definition, layered situated) validate — the schemas' vertexId pattern
    // accepts either base, with an identical slug pattern.
    let good = materialize("C4/good");
    build_and_check_ok(&good);

    // Fail pole: a `layer: floor` vertex under a repo base must FAIL C4 —
    // floor is bedrock-namespaced (the floor gate binds layer:floor @ids to
    // the bedrock base).
    let bad = materialize("C4/bad");
    let combined = check_fails_with(&bad, "C4");
    assert!(
        combined.contains("floor-repo-base.yamlld"),
        "floor-out-of-base fixture named: {combined}"
    );
    assert!(
        combined.contains("https://yeetz.dev/myrepo/vertex/invariant-floor-renamed"),
        "offending @id named: {combined}"
    );
}

#[test]
fn c5_edge_resolution_polarity() {
    let good = materialize("C5/good");
    build_and_check_ok(&good);

    let bad = materialize("C5/bad");
    let combined = check_fails_with(&bad, "C5");
    assert!(
        combined.contains("https://yeetz.dev/bedrock/vertex/ghost"),
        "{combined}"
    );
    assert!(combined.contains("does-not-exist.md"), "{combined}");
}

#[test]
fn c6_determinism_polarity() {
    let good = materialize("C6/good");
    build_and_check_ok(&good);

    // Tamper the committed graph.trig → C6 drift.
    let bad = materialize("C6/bad");
    build_and_check_ok(&bad);
    let trig = bad.path().join("situation").join("graph.trig");
    let mut bytes = std::fs::read(&trig).unwrap();
    // Flip a byte late in the file (prelude is fixed; flip a literal digit).
    *bytes.last_mut().unwrap() ^= 0x01;
    let _ = std::fs::write(&trig, bytes);
    let combined = check_fails_with(&bad, "C6");
    assert!(combined.contains("graph.trig"), "{combined}");
}

#[test]
fn c7_parseback_gate_polarity() {
    // Good: build compiles and the pipeline parse-back checks pass.
    let good = materialize("C7/good");
    build_and_check_ok(&good);

    // Bad: the parse-back gate, driven on real emitted bytes with a
    // corrupted payload, must produce a C7 violation. (No valid source input
    // can produce a divergent TriG — C7 is a pipeline-integrity gate.)
    let s = materialize("C7/good");
    build_and_check_ok(&s);
    let src =
        std::fs::read_to_string(s.path().join("situation/definition/invariant.yamlld")).unwrap();
    let parsed = compile::parse_yaml(&src, "situation/definition/invariant.yamlld");
    let value = parsed.value.expect("parses");
    let registry = bedrock::contextreg::ContextRegistry::embedded();
    let quads = compile::expand(&value, "x", &src, &registry).expect("expands");
    let sorted = compile::sort_quads(quads);
    let trig = compile::serialize_trig(&sorted).expect("serializes");
    assert!(
        compile::verify_parseback(&sorted, &trig).is_none(),
        "clean bytes pass"
    );

    // Corrupt a literal byte and pad: gate must now produce a C7 violation.
    let mut corrupt = trig;
    let n = corrupt.len();
    corrupt[n - 2] ^= 0x40; // flip a hex digit
    assert!(
        compile::verify_parseback(&sorted, &corrupt).is_some(),
        "corrupted TriG must trip C7"
    );
}

#[test]
fn c8_witness_gate_polarity() {
    // Good: a Plan and a ReflectVerdict, both disposition done WITH witnesses.
    let good = materialize("C8/good");
    build_and_check_ok(&good);

    // Bad: disposition done with zero witnesses -> C8 on both namespaces.
    let bad = materialize("C8/bad");
    let combined = check_fails_with(&bad, "C8");
    assert!(
        combined.contains("no witness, no done"),
        "gate message present: {combined}"
    );
    assert!(
        combined.contains("situation/plan/plan-done.yamlld"),
        "plan named: {combined}"
    );
    assert!(combined.contains("C8"), "{combined}");
}

#[test]
fn c9_base_type_polarity() {
    // Good: a repo archetype riding alongside a base @type.
    let good = materialize("C9/good");
    build_and_check_ok(&good);

    // Bad: a repo archetype alone, no base @type.
    let bad = materialize("C9/bad");
    let combined = check_fails_with(&bad, "C9");
    assert!(
        combined.contains("no base @type"),
        "base-type message present: {combined}"
    );
    assert!(
        combined.contains("https://yeetz.dev/myrepo/ADR"),
        "offending archetype named: {combined}"
    );
}

#[test]
fn c10_digest_skew_polarity() {
    // Good: installed base files byte-match this binary's canonical stamped
    // form — the embedded template + provenance stamp for the current
    // version. The fixture ships the real stamped form.
    let good = materialize("C10/good");
    build_and_check_ok(&good);
    // The stamped form names the generating version in each class.
    let ctx = std::fs::read_to_string(good.path().join("seed/context.yamlld")).unwrap();
    assert!(
        ctx.starts_with("# Installed by bedrock v"),
        "context carries the machine-owned header: {ctx}"
    );
    let schema = std::fs::read_to_string(good.path().join("seed/schemas/plan.json")).unwrap();
    assert!(
        schema.contains("\"$comment\": \"Installed by bedrock v"),
        "schema carries the $comment stamp: {schema}"
    );

    // Bad: a stale-version stamp (v0.2.0 against this v{} binary) plus
    // appended tamper on a schema, the context, and the operating reference.
    let bad = materialize("C10/bad");
    let combined = check_fails_with(&bad, "C10");
    assert!(
        combined.contains("bedrock update"),
        "violation names `bedrock update`: {combined}"
    );
    assert!(
        combined.contains("file an issue")
            && combined.contains("https://github.com/cleverunicornz/bedrock"),
        "violation routes base defects to the owning repo: {combined}"
    );
    assert!(
        combined.contains("seed/schemas/plan.json")
            && combined.contains("seed/context.yamlld")
            && combined.contains("bedrock-operating.md"),
        "skewed files named: {combined}"
    );
}

#[test]
fn plan_trig_projection_is_written_and_deterministic() {
    // C1/good carries a plan vertex → build emits situation/plan/plan-a.trig.
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let p = s.path().join("situation/plan/plan-a.trig");
    assert!(
        p.exists(),
        "per-plan .trig must be emitted next to the plan source"
    );
    let first = std::fs::read(&p).unwrap();
    // Rebuild → byte-stable.
    let (c, _, _) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0);
    let second = std::fs::read(&p).unwrap();
    assert_eq!(first, second, "plan projection must be byte-stable (C6)");
    let text = String::from_utf8_lossy(&first);
    assert!(
        text.contains("plan-a"),
        "plan projection names the plan: {text}"
    );
}

#[test]
fn commit_idempotence_deterministic_regenerate() {
    // Build twice → identical graph.trig + AGENTS.md (C6 byte stability).
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let g1 = std::fs::read(s.path().join("situation/graph.trig")).unwrap();
    let a1 = std::fs::read(s.path().join("AGENTS.md")).unwrap();
    let (c, _, _) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0);
    let g2 = std::fs::read(s.path().join("situation/graph.trig")).unwrap();
    let a2 = std::fs::read(s.path().join("AGENTS.md")).unwrap();
    assert_eq!(g1, g2, "graph.trig must be byte-stable");
    assert_eq!(a1, a2, "AGENTS.md must be byte-stable");
}

#[test]
fn agents_md_register_content() {
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let md = std::fs::read_to_string(s.path().join("AGENTS.md")).unwrap();
    // Invariants heading + at least the one fixture invariant's statement.
    assert!(
        md.contains("## Invariants"),
        "register has invariants heading"
    );
    assert!(
        md.contains("A fixture invariant that must compile clean."),
        "register renders vertex statements: {md}"
    );
    // Breadcrumbs heading (none declared in this fixture → placeholder).
    assert!(md.contains("## Breadcrumbs"));
    assert!(md.contains("## Where things live"));
    // 0.2.0 base protocol: operating section with THE CHAIN in one line.
    assert!(
        md.contains("## Operating this repository"),
        "operating section present: {md}"
    );
    assert!(
        md.contains("every plan is a promise; its criteria are its oracle; its witnesses prove it held; its residual declares what was not assured"),
        "the chain rendered in one line: {md}"
    );
    assert!(
        md.contains("situation/references/bedrock-operating.md"),
        "pointer to the operating reference: {md}"
    );
    // Terminal generator+digest marker.
    assert!(
        md.contains("bedrock ") && md.contains("digest "),
        "marker line present: {md}"
    );
    // 0.2.1 provenance: the register's first line announces it is generated.
    assert!(
        md.starts_with("<!-- generated by bedrock v"),
        "register opens with the machine-owned comment: {md}"
    );
    // No TriG embedded (register is a projection).
    assert!(!md.contains("@prefix"), "AGENTS.md must not embed TriG");
}

#[test]
fn check_unbuilt_repo_flags_c6() {
    // A repo whose situation/ changed since the last build must drift C6.
    let s = materialize("C3/good");
    build_and_check_ok(&s);
    // Change actual data: a comment would not alter quads — the statement
    // text must change for the compiled graph to differ.
    let v = s.path().join("situation/definition/clean.yamlld");
    let text = std::fs::read_to_string(&v).unwrap();
    let changed = text.replace(
        "No blank nodes, absolute IRIs, single served context.",
        "No blank nodes, absolute IRIs, single served context — revised.",
    );
    assert_ne!(changed, text);
    std::fs::write(&v, changed).unwrap();
    let combined = check_fails_with(&s, "C6");
    assert!(combined.contains("graph.trig"), "{combined}");
}

#[test]
fn build_regenerates_a_stale_register() {
    // build must self-heal its own outputs: a hand-mangled AGENTS.md is
    // regenerated, not a blocker (the strict drift gate is `check`, CI's).
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let agents = s.path().join("AGENTS.md");
    std::fs::write(&agents, b"# sabotage\n").unwrap();
    let (c, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 0, "build must regenerate the register:\n{combined}");
    let md = std::fs::read_to_string(&agents).unwrap();
    assert!(md.contains("## Invariants"), "register restored: {md}");
    // Drift remains detectable by check alone.
    std::fs::write(&agents, b"# sabotage\n").unwrap();
    let (c2, out2, err2) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 1, "check must still flag drift:\n{out2}\n{err2}");
    assert!(
        format!("{out2}\n{err2}").contains("C1 AGENTS.md"),
        "drift named as C1 on AGENTS.md:\n{out2}\n{err2}"
    );
}
