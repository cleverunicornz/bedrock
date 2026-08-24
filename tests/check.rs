//! Both-polarity contract tests for source validation, resident projection,
//! generated artifact behavior, and init/adopt/update.

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
fn plan_lifecycle_state_is_required() {
    let s = materialize("C1/good");
    let path = s.path().join("situation/plan/plan-a.yamlld");
    let text = std::fs::read_to_string(&path).unwrap();
    let changed = text.replace("disposition:\n  state: active\n", "");
    assert_ne!(changed, text);
    std::fs::write(path, changed).unwrap();
    let combined = check_fails_with(&s, "C4");
    assert!(
        combined.contains("disposition") && combined.contains("required"),
        "Plan lifecycle failure is explicit: {combined}"
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
        combined.contains("root AGENTS.md (the resident projection) is out of date or hand-edited"),
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
    // Decisions are the resident record class: both vertices and their
    // supersedes edge must survive projection.
    let good = materialize("C5/supersedes-chain");
    build_and_check_ok(&good);
    let md = std::fs::read_to_string(good.path().join("AGENTS.md")).unwrap();
    let body = md.split_once("\n\n").expect("artifact has TriG body").1;
    assert!(
        body.contains("decision-posix-only")
            && body.contains("decision-windows-revisited")
            && (body.contains("bedrock:supersedes")
                || body.contains("<https://yeetz.dev/bedrock/supersedes>")),
        "Decision chain stays resident across the identity bridge: {md}"
    );
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
    let lock = std::fs::read_to_string(good.path().join("seed/substrate-lock.json")).unwrap();
    assert!(
        lock.contains("\"checker\"") && lock.contains("\"ref\": \"0.7.0\""),
        "substrate lock carries the exact checker ref: {lock}"
    );

    // Bad: stale-version stamps and lock ref across every C10-owned class.
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
            && combined.contains("seed/substrate-lock.json")
            && combined.contains("bedrock-operating.md"),
        "skewed files named: {combined}"
    );
}

#[test]
fn c11_resident_projection_closure_polarity() {
    let good = materialize("C11/good");
    build_and_check_ok(&good);
    let (_, out, _) = run(&["check", good.path().to_str().unwrap()], &manifest());
    assert!(
        out.contains("plans 1 active resident; 1 draft"),
        "projection report exposes active/cold lifecycle: {out}"
    );

    let bad = materialize("C11/bad");
    let combined = check_fails_with(&bad, "C11");
    assert!(
        combined.contains("resident vertex")
            && combined.contains("cold vertex")
            && combined.contains("repo-path pointer"),
        "C11 names the boundary and fix: {combined}"
    );
}

#[test]
fn closed_plan_and_reflect_record_are_cold() {
    let s = materialize("C8/good");
    build_and_check_ok(&s);
    let md = std::fs::read_to_string(s.path().join("AGENTS.md")).unwrap();
    assert!(
        !md.contains("plan-done") && !md.contains("verdict-witnessed"),
        "done Plan and ReflectVerdict must not occupy resident context: {md}"
    );
    let (_, out, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert!(
        out.contains("plans 0 active resident; 0 draft / 1 done")
            && out.contains("records 0 decisions resident / 1 episodic cold"),
        "report makes cold history visible without injecting it: {out}"
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
        out.contains("bedrock build: AGENTS.md (the resident projection) up to date"),
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
    // ONE artifact: root AGENTS.md IS the resident situation graph. The
    // preamble stays valid TriG comments; the body is the deterministic
    // current working-set projection, never the execution archive.
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
        preamble.contains("This file IS the resident situation graph"),
        "how-to-read names the resident projection: {preamble}"
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

    // The body: resident TriG — fixed prefixes, resident named graphs, and
    // compiler-emitted document edges.
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
        body.contains("bedrock:state \"active\""),
        "active Plan exposes its lifecycle state: {body}"
    );
    for cold in [
        "bedrock:acceptance-criteria",
        "bedrock:tasks",
        "bedrock:witnesses",
        "bedrock:reflect-depth",
        "bedrock:disposition",
    ] {
        assert!(
            !body.contains(cold),
            "active Plan payload `{cold}` stays cold: {body}"
        );
    }
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

#[test]
fn body_never_compiles_into_the_graph() {
    // 0.5.0 face/body law: `body` is uncompiled depth prose. Adding one
    // changes ZERO bytes of the artifact, and neither the prose nor a
    // `bedrock:body` predicate ever reaches the graph.
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let agents = s.path().join("AGENTS.md");
    let before = std::fs::read(&agents).unwrap();

    let vertex = s.path().join("situation/definition/invariant-01.yamlld");
    let mut text = std::fs::read_to_string(&vertex).unwrap();
    text.push_str(
        "body: |\n  ## Depth\n  UNCOMPILED-MARKER-9314 rides only in the source document.\n",
    );
    std::fs::write(&vertex, text).unwrap();

    let (c, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "build with a body must pass:\n{out}\n{err}");
    let after = std::fs::read(&agents).unwrap();
    assert_eq!(before, after, "a body-only edit changes no artifact bytes");
    let rendered = String::from_utf8_lossy(&after);
    assert!(
        !rendered.contains("UNCOMPILED-MARKER-9314"),
        "body prose must never reach the graph"
    );
    assert!(
        !rendered.contains("bedrock:body"),
        "no body predicate may reach the graph"
    );
    let (c2, out2, err2) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 0, "check stays clean with a body:\n{out2}\n{err2}");
}

#[test]
fn active_plan_payload_edit_changes_no_artifact_bytes() {
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let agents = s.path().join("AGENTS.md");
    let before = std::fs::read(&agents).unwrap();
    let plan = s.path().join("situation/plan/plan-a.yamlld");
    let text = std::fs::read_to_string(&plan).unwrap();
    let changed = text.replace(
        "Run bedrock build.",
        "Run bedrock build, then retain the execution witness.",
    );
    assert_ne!(changed, text);
    std::fs::write(plan, changed).unwrap();

    let (code, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(
        code, 0,
        "cold active-Plan payload remains valid:\n{out}\n{err}"
    );
    let after = std::fs::read(&agents).unwrap();
    assert_eq!(
        before, after,
        "changing only active-Plan tasks changes no resident bytes"
    );
}

#[test]
fn yamlld_reference_is_validated_but_cold() {
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let agents = s.path().join("AGENTS.md");
    let before = std::fs::read(&agents).unwrap();
    std::fs::write(
        s.path().join("situation/references/cold-term.yamlld"),
        "\"@context\": \"https://yeetz.dev/bedrock/context/v1\"\n\
         \"@id\": \"https://yeetz.dev/bedrock/vertex/cold-reference-term\"\n\
         \"@type\": \"https://yeetz.dev/bedrock/ontology/Term\"\n\
         label: \"Cold reference term\"\n\
         statement: \"Valid source that is read only through the disclosed references path.\"\n",
    )
    .unwrap();
    let (code, out, err) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(code, 0, "cold YAML-LD reference validates:\n{out}\n{err}");
    assert!(
        out.contains("references 1 YAML-LD cold"),
        "report counts cold references: {out}"
    );
    let after = std::fs::read(&agents).unwrap();
    assert_eq!(
        before, after,
        "adding a cold YAML-LD reference changes no resident bytes"
    );
}

#[test]
fn body_must_be_a_string_fails_c4() {
    // Bad polarity: a non-string body is a schema violation (C4), loudly.
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let vertex = s.path().join("situation/definition/invariant-01.yamlld");
    let mut text = std::fs::read_to_string(&vertex).unwrap();
    text.push_str("body:\n  - not\n  - prose\n");
    std::fs::write(&vertex, text).unwrap();
    let (c, out, err) = run(&["check", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 1, "non-string body must fail check:\n{combined}");
    assert!(
        combined.contains("C4") && combined.contains("body"),
        "violation is C4 and names body:\n{combined}"
    );
}

#[test]
fn size_report_soft_budget_polarity() {
    // The size report is advisory, never failing: a compiled face over the
    // 4096-char soft budget earns a SOFT line while exit stays 0.
    let s = materialize("C1/good");
    build_and_check_ok(&s);
    let (c, out, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0);
    assert!(
        out.contains("bedrock report: AGENTS.md"),
        "check prints the size report: {out}"
    );
    assert!(
        out.contains("0 face(s) over the 4096-char soft budget"),
        "small resident faces stay under budget: {out}"
    );
    assert!(!out.contains("SOFT "), "no SOFT lines under budget: {out}");

    // A vertex whose face exceeds the budget (statement within its own
    // 4096-char field bound; the whole face crosses the line).
    let big = "x".repeat(4000);
    std::fs::write(
        s.path().join("situation/definition/big-face.yamlld"),
        format!(
            "\"@context\": \"https://yeetz.dev/bedrock/context/v1\"\n\"@id\": \"https://yeetz.dev/bedrock/vertex/big-face\"\n\"@type\": \"https://yeetz.dev/bedrock/ontology/Term\"\nlabel: \"Big face\"\nstatement: \"{big}\"\n"
        ),
    )
    .unwrap();
    let (c2, out2, err2) = run(&["build", s.path().to_str().unwrap()], &manifest());
    assert_eq!(
        c2, 0,
        "an over-budget face NEVER fails build:\n{out2}\n{err2}"
    );
    assert!(
        out2.contains("1 face(s) over the 4096-char soft budget"),
        "report counts the oversized resident face: {out2}"
    );
    assert!(
        out2.contains("SOFT situation/definition/big-face.yamlld:1 resident face"),
        "SOFT line names the file: {out2}"
    );
    let (c3, out3, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c3, 0, "an over-budget face NEVER fails check:\n{out3}");
    assert!(
        out3.contains("SOFT situation/definition/big-face.yamlld:1 resident face"),
        "check repeats the SOFT line: {out3}"
    );
}
