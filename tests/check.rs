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

/// Every `.trig` file under `dir` (recursive) — build must leave none.
fn stray_trig(dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "trig") {
                out.push(p.to_string_lossy().into_owned());
            }
        }
    }
    out
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
fn c1_drift_polarity() {
    // 0.4.0: C1 is the single drift rule — the committed root AGENTS.md
    // must byte-match regeneration. Good: an untouched build output.
    let good = materialize("C6/good");
    build_and_check_ok(&good);

    // Bad: a hand-edited AGENTS.md — a line appended inside a graph block,
    // so regeneration differs — fails C1 naming AGENTS.md.
    let bad = materialize("C6/good");
    build_and_check_ok(&bad);
    let agents = bad.path().join("AGENTS.md");
    let md = std::fs::read_to_string(&agents).unwrap();
    let close = md.rfind('}').expect("a named-graph closer to edit inside");
    let tampered = format!("{}# a hand-edited line\n{}", &md[..close], &md[close..]);
    assert_ne!(tampered, md);
    std::fs::write(&agents, tampered).unwrap();
    let combined = check_fails_with(&bad, "C1");
    assert!(
        combined.contains("C1 AGENTS.md"),
        "drift named as C1 on AGENTS.md: {combined}"
    );
    assert!(
        combined.contains("root AGENTS.md (the compiled graph) is out of date or hand-edited"),
        "drift message present: {combined}"
    );
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
fn decision_record_and_supersedes_chain_pass() {
    // Good (0.3.0): two record/ Decision vertices — the second carries
    // `supersedes` pointing at the first's @id, proving the new edge
    // resolves under C5. One vertex spells @type as a scalar, the other as
    // an array with a repo archetype riding alongside (C9). Neither carries
    // disposition/witnesses — C8 gates Plan and ReflectVerdict only.
    let good = materialize("C5/supersedes-chain");
    build_and_check_ok(&good);
}

#[test]
fn decision_missing_required_fields_fails_c4() {
    // Bad (0.3.0): one Decision missing `timestamp`, one missing
    // `statement` — both required by the record schema's Decision arm.
    let bad = materialize("C4/decision-missing-required");
    let combined = check_fails_with(&bad, "C4");
    assert!(combined.contains("required"), "{combined}");
    assert!(combined.contains("timestamp"), "{combined}");
    assert!(combined.contains("statement"), "{combined}");
    assert!(
        combined.contains("decision-no-timestamp.yamlld")
            && combined.contains("decision-no-statement.yamlld"),
        "both offending vertices named: {combined}"
    );
}

#[test]
fn decision_with_disposition_fails_c4() {
    // Bad (0.3.0 eval round 2): a Decision carrying `disposition` — the
    // record schema's Decision arm rejects disposition/witnesses outright
    // (a decision is not CI-judged); the prose now matches enforcement.
    let bad = materialize("C4/decision-with-disposition");
    let combined = check_fails_with(&bad, "C4");
    assert!(
        combined.contains("decision-with-disposition.yamlld"),
        "offending vertex named: {combined}"
    );
}

#[test]
fn supersedes_dangling_target_fails_c5() {
    // Bad (0.3.0): a Decision whose `supersedes` edge points at a vertex
    // @id that exists nowhere in this repo's graph — a dangling edge like
    // any other.
    let bad = materialize("C5/supersedes-dangling");
    let combined = check_fails_with(&bad, "C5");
    // The dangling-vertex violation names the unresolved target IRI.
    assert!(
        combined.contains("https://yeetz.dev/bedrock/vertex/decision-never-written"),
        "{combined}"
    );
    assert!(
        combined.contains("neither a vertex @id"),
        "dangling-edge message present: {combined}"
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
fn build_emits_only_agents_md() {
    // 0.4.0: ONE artifact. Build writes the root AGENTS.md and nothing
    // else — no situation/graph.trig, no situation/plan/*.trig — and a
    // repo migrating from <=0.3.0 (legacy artifacts still on disk) builds
    // clean: check flags them, build deletes them.
    let s = materialize("C1/good");
    // Plant the legacy artifacts exactly as <=0.3.0 left them.
    std::fs::write(s.path().join("situation/graph.trig"), "# legacy graph\n").unwrap();
    std::fs::write(
        s.path().join("situation/plan/plan-a.trig"),
        "# legacy plan\n",
    )
    .unwrap();

    // check (the CI gate) flags both as C1 legacy artifacts, by path.
    let combined = check_fails_with(&s, "C1");
    assert!(
        combined.contains("legacy generated artifact"),
        "legacy message present: {combined}"
    );
    assert!(
        combined.contains("situation/graph.trig")
            && combined.contains("situation/plan/plan-a.trig"),
        "legacy paths named: {combined}"
    );

    // build deletes them, emits only AGENTS.md, and passes.
    let (c, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 0, "build must clean legacy artifacts:\n{combined}");
    assert!(
        out.contains("bedrock build: AGENTS.md (the compiled graph) up to date"),
        "build stdout names the single artifact: {out}"
    );
    assert!(
        s.path().join("AGENTS.md").exists(),
        "the one artifact exists"
    );
    assert!(
        stray_trig(&s.path().join("situation")).is_empty(),
        "build leaves no .trig under situation/"
    );
    // The migrated repo now checks clean.
    let (c2, out2, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 0, "post-migration check passes: {out2}");
}

#[test]
fn commit_idempotence_deterministic_regenerate() {
    // Build twice → byte-identical AGENTS.md (C6 byte stability) and no
    // resurrected .trig under situation/ — one artifact stays one artifact.
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let a1 = std::fs::read(s.path().join("AGENTS.md")).unwrap();
    let (c, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "{out}\n{err}");
    let a2 = std::fs::read(s.path().join("AGENTS.md")).unwrap();
    assert_eq!(a1, a2, "AGENTS.md must be byte-stable");
    assert!(
        stray_trig(&s.path().join("situation")).is_empty(),
        "rebuild must not resurrect .trig artifacts"
    );
}

#[test]
fn agents_md_compiled_graph_content() {
    // 0.4.0: ONE artifact — the root AGENTS.md IS the compiled graph: a
    // `#`-comment preamble (so the whole file stays valid TriG) followed
    // by the complete TriG body. No prose projection sections.
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let md = std::fs::read_to_string(s.path().join("AGENTS.md")).unwrap();
    let v = env!("CARGO_PKG_VERSION");

    // Machine-owned stamp: a TriG comment, not an HTML comment.
    assert!(
        md.starts_with(&format!(
            "# generated by bedrock v{v}; do not edit; source: situation/\n"
        )),
        "AGENTS.md opens with the machine-owned TriG comment: {md}"
    );
    assert!(
        !md.contains("<!--"),
        "no HTML comments — the file is TriG: {md}"
    );

    // The preamble is pure `#` comments up to the blank line before the
    // body (legal TriG end-to-end).
    let (preamble, body) = md
        .split_once("\n\n")
        .expect("a blank line separates preamble from graph body");
    assert!(
        preamble.lines().all(|l| l.starts_with('#')),
        "every preamble line is a TriG comment: {preamble}"
    );

    // How-to-read block + title: the repo dir name (this fixture declares
    // no identity vertex).
    assert!(
        preamble.contains("This file IS the situation graph"),
        "how-to-read names the file itself: {preamble}"
    );
    let repo_name = s
        .path()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    assert!(
        preamble.contains(&format!("# {repo_name}\n")),
        "title line names the repo dir: {preamble}"
    );

    // Operating section: the verb-framing line, the five verb lines, the
    // authoring loop — and the Decisions line verbatim, between the chain
    // and the base-protocol pointer.
    assert!(
        preamble.contains("# Operating this repository:"),
        "{preamble}"
    );
    assert!(
        preamble.contains("# - Work happens on verb-prefixed branches"),
        "verb-framing line: {preamble}"
    );
    for verb in ["think", "plan", "execute", "reflect", "deploy"] {
        assert!(
            preamble.contains(&format!("# - {verb}/ ")),
            "verb line {verb}/: {preamble}"
        );
    }
    assert!(preamble.contains("# - Authoring loop:"), "{preamble}");
    assert!(
        preamble.contains("# - Decisions are records too: why a design is what it is lives in record/ Decision vertices — walk `supersedes` chains before relitigating a choice; write one when you close a fork. Semantics: situation/references/bedrock-operating.md"),
        "the decisions line rendered verbatim: {preamble}"
    );
    let chain_pos = preamble.find("# - The chain:").unwrap();
    let decisions_pos = preamble.find("# - Decisions are records too:").unwrap();
    let pointer_pos = preamble.find("# - The base protocol, in full").unwrap();
    assert!(
        chain_pos < decisions_pos && decisions_pos < pointer_pos,
        "decisions line sits between the chain and the base-protocol pointer: {preamble}"
    );

    // Digest marker: 64 hex chars pinning the body bytes.
    let marker = format!("# bedrock {v} digest ");
    let line = preamble
        .lines()
        .find(|l| l.starts_with(&marker))
        .expect("digest marker line");
    let rest = &line[marker.len()..];
    let (hex, tail) = rest.split_at(64);
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "digest is 64 hex chars: {line}"
    );
    assert!(
        tail.starts_with(" (regenerated by `bedrock build`"),
        "digest marker names regeneration: {line}"
    );

    // The body: the complete TriG — the fixed @prefix prelude, the named
    // namespace graphs, and the compiler-emitted document edge pointing
    // each vertex at its own source file.
    for p in [
        "@prefix rdf: ",
        "@prefix xsd: ",
        "@prefix bedrock: ",
        "@prefix graph: ",
    ] {
        assert!(body.contains(p), "prefix prelude carries {p}: {body}");
    }
    assert!(body.contains("graph:definition {"), "{body}");
    assert!(body.contains("graph:plan {"), "{body}");
    assert!(
        body.contains(
            "bedrock:document bedrock:path\\/situation\\/definition\\/invariant-01.yamlld"
        ),
        "document edge points at the vertex source: {body}"
    );
    assert!(
        body.contains("bedrock:document bedrock:path\\/situation\\/plan\\/plan-a.yamlld"),
        "plan vertex carries its document edge: {body}"
    );
    assert!(
        md.contains("A fixture invariant that must compile clean."),
        "the graph embeds vertex statements: {md}"
    );

    // No markdown projection sections survive anywhere.
    for stale in ["## Invariants", "## Breadcrumbs", "## Where things live"] {
        assert!(!md.contains(stale), "no `{stale}` section: {md}");
    }
    assert!(
        !md.lines().any(|l| l.starts_with("## ")),
        "no markdown ## sections at all: {md}"
    );
}

#[test]
fn check_flags_drifted_source_c1() {
    // A repo whose situation/ changed since the last build drifts: the
    // committed AGENTS.md no longer matches regeneration → C1.
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
    let combined = check_fails_with(&s, "C1");
    assert!(
        combined.contains("C1 AGENTS.md"),
        "drift named as C1 on AGENTS.md: {combined}"
    );
}

#[test]
fn build_regenerates_a_hand_edited_agents_md() {
    // build must self-heal its own output: a hand-mangled AGENTS.md is
    // regenerated, not a blocker (the strict drift gate is `check`, CI's).
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let agents = s.path().join("AGENTS.md");
    std::fs::write(&agents, b"# sabotage\n").unwrap();
    let (c, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 0, "build must regenerate the graph:\n{combined}");
    let md = std::fs::read_to_string(&agents).unwrap();
    assert!(
        md.starts_with("# generated by bedrock v"),
        "artifact restored: {md}"
    );
    assert!(
        md.contains("@prefix bedrock:"),
        "the compiled graph body is restored: {md}"
    );
    // Drift remains detectable by check alone.
    std::fs::write(&agents, b"# sabotage\n").unwrap();
    let (c2, out2, err2) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 1, "check must still flag drift:\n{out2}\n{err2}");
    assert!(
        format!("{out2}\n{err2}").contains("C1 AGENTS.md"),
        "drift named as C1 on AGENTS.md:\n{out2}\n{err2}"
    );
}
