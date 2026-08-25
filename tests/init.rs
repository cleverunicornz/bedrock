//! init/adopt behavior tests (SPINE §1/§6/§7): seed install, epoch record,
//! check-before-finish, instruction emission, version-gate wiring.

use std::process::Command;

mod common;
use common::{
    Scratch, bedrock_exe, fixture_seed, manifest, run, run_gate, run_no_seed, version_json,
};

#[test]
fn init_seeds_new_repo_and_passes_check() {
    let s = Scratch::new("init");
    let (c, out, err) = run(&["init", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 0, "init must succeed:\n{combined}");

    // Situation skeleton installed with the six namespaces.
    for ns in [
        "definition",
        "architecture",
        "risk",
        "plan",
        "record",
        "references",
    ] {
        assert!(s.path().join("situation").join(ns).is_dir(), "missing {ns}");
    }
    // Seed floor vertices + identity + breadcrumb copied in.
    assert!(
        s.path()
            .join("situation/definition/invariant-01.yamlld")
            .exists()
    );
    assert!(
        s.path()
            .join("situation/architecture/identity.yamlld")
            .exists()
    );
    // Epoch record written (mode: init).
    let record = s.path().join("situation/record");
    let fname = std::fs::read_dir(record)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with("epoch-"))
        .expect("an epoch record must exist");
    let text = std::fs::read_to_string(s.path().join("situation/record").join(&fname)).unwrap();
    assert!(
        text.contains("mode: init"),
        "epoch record carries mode init: {text}"
    );
    assert!(
        text.contains("version:") && text.contains("commit:"),
        "epoch record carries commit+version: {text}"
    );
    assert!(text.contains("statement:"), "{text}");
    assert!(
        text.contains("\"@context\": \"urn:bedrock:context/v1\"")
            && text.contains("\"@id\": \"urn:bedrock:vertex/")
            && text.contains("\"@type\": \"urn:bedrock:ontology/EpochRecord\""),
        "new epoch authoring is canonical URN only: {text}"
    );
    // Compiled artifact: exactly one — the root AGENTS.md; init never
    // writes the legacy situation/graph.trig.
    assert!(s.path().join("AGENTS.md").exists());
    assert!(
        !s.path().join("situation/graph.trig").exists(),
        "the compiled graph lives in AGENTS.md — no separate artifact"
    );
    // 0.2.0 base protocol: the operating reference is installed.
    assert!(
        s.path()
            .join("situation/references/bedrock-operating.md")
            .exists(),
        "operating reference installed by init"
    );
    assert!(
        s.path().join("seed/substrate-lock.json").is_file(),
        "substrate lock installed"
    );
    // Instruction set emitted (W4 placeholders are wired via include_str!).
    assert!(
        out.contains("AGENTS.md") && out.contains("think/"),
        "init prints the real instruction set: {out}"
    );

    // A subsequent check passes (idempotent, artifacts fresh).
    let (c2, out2, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 0, "check after init must pass:\n{out2}");
}

#[test]
fn init_stamps_machine_owned_provenance() {
    // 0.2.1: every installed/generated artifact announces it is machine-owned
    // with a provenance stamp naming the generating version.
    let s = Scratch::new("init-stamps");
    let (c, out, err) = run(&["init", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "init must succeed\n{out}\n{err}");
    let v = env!("CARGO_PKG_VERSION");
    let stamp = format!("Installed by bedrock v{v}");
    // # comment header classes.
    for rel in [
        "seed/context.yamlld",
        "situation/references/bedrock-operating.md",
        "situation/references/review-playbook.md",
    ] {
        let p = s.path().join(rel);
        assert!(p.is_file(), "missing {rel}");
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(
            text.starts_with(&format!("# {stamp}")),
            "{rel} carries the machine-owned header: {text}"
        );
    }
    // JSON schemas carry the "$comment" key.
    let schema = std::fs::read_to_string(s.path().join("seed/schemas/definition.json")).unwrap();
    assert!(
        schema.contains(&format!("\"$comment\": \"{stamp}")),
        "schema carries the $comment stamp: {schema}"
    );
    let lock = std::fs::read_to_string(s.path().join("seed/substrate-lock.json")).unwrap();
    assert!(
        lock.contains(&format!("\"$comment\": \"{stamp}")) && lock.contains("\"ref\": \"0.8.0\""),
        "lock carries provenance and exact checker ref: {lock}"
    );
    // Floor vertices get the light `# seeded by bedrock vX` note only (repos
    // own their situation — never a do-not-edit).
    let def_dir = s.path().join("situation/definition");
    let floor_vertex = std::fs::read_dir(&def_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().map(|x| x == "yamlld").unwrap_or(false))
        .expect("a floor vertex was installed");
    let floor = std::fs::read_to_string(&floor_vertex).unwrap();
    assert!(
        floor.starts_with(&format!("# seeded by bedrock v{v}")),
        "floor vertex carries the seeded note: {floor}"
    );
    assert!(
        !floor.contains("DO NOT EDIT"),
        "floor vertices never carry a do-not-edit: {floor}"
    );
    // Generated artifact: the root AGENTS.md opens with the machine-owned
    // TriG-comment stamp (the whole file is valid TriG — preamble and
    // graph body together).
    let agents = std::fs::read_to_string(s.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.starts_with(&format!(
            "# generated by bedrock v{v}; do not edit; source: situation/"
        )),
        "AGENTS.md opens with the machine-owned comment: {agents}"
    );
    assert!(
        !s.path().join("situation/graph.trig").exists(),
        "no separate graph artifact — AGENTS.md is the graph"
    );
}

#[test]
fn init_refuses_existing_repo() {
    let s = Scratch::new("init-existing");
    std::fs::create_dir_all(s.path().join("situation/definition")).unwrap();
    std::fs::write(s.path().join("situation/definition/old.yamlld"), "x: 1").unwrap();
    let (c, out, err) = run(&["init", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 1, "init on an existing repo must fail");
    let combined = format!("{out}\n{err}");
    assert!(
        combined.contains("adopt"),
        "must point at adopt: {combined}"
    );
}

#[test]
fn init_missing_seed_is_loud() {
    // An EXPLICIT BEDROCK_SEED pointing at nothing stays a loud exit-1
    // failure — the SPINE §1 order never silently falls through to `./seed`
    // in cwd or the embedded copy once the env var is defined.
    let s = Scratch::new("init-noseed");
    let out = Command::new(bedrock_exe())
        .args(["init", s.path().to_str().unwrap()])
        .current_dir(s.path())
        .env("BEDROCK_SEED", s.path().join("does-not-exist"))
        .env(
            "BEDROCK_VERSION_JSON",
            version_json(env!("CARGO_PKG_VERSION")),
        )
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text.contains("BEDROCK_SEED"),
        "must name the missing BEDROCK_SEED: {text}"
    );
}

#[test]
fn init_without_seed_on_disk_uses_embedded_copy_and_passes_check() {
    // A bare dir with no ./seed on disk and no BEDROCK_SEED must seed from
    // the compile-time-embedded seed/ (the 0.1.1 standalone-binary fix:
    // cargo-installed bedrock has no repository checkout to read seed/ from).
    let s = Scratch::new("embedded-seed");
    let out = Command::new(bedrock_exe())
        .args(["init", s.path().to_str().unwrap()])
        .current_dir(s.path())
        .env_remove("BEDROCK_SEED")
        .env(
            "BEDROCK_VERSION_JSON",
            version_json(env!("CARGO_PKG_VERSION")),
        )
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "embedded-seed init must succeed:\n{text}"
    );
    // The embedded copy installed the six namespaces + real floor content.
    for ns in [
        "definition",
        "architecture",
        "risk",
        "plan",
        "record",
        "references",
    ] {
        assert!(s.path().join("situation").join(ns).is_dir(), "missing {ns}");
    }
    assert!(
        s.path()
            .join("situation/definition/invariant-01-possibility-space.yamlld")
            .exists(),
        "embedded seed floor vertices must be installed"
    );
    assert!(
        !s.path().join("situation/graph.trig").exists(),
        "the compiled graph lives in AGENTS.md — no separate artifact"
    );
    assert!(s.path().join("AGENTS.md").exists());
    let lock = std::fs::read_to_string(s.path().join("seed/substrate-lock.json")).unwrap();
    assert!(
        lock.contains("\"package\": \"yeetz-bedrock\"") && lock.contains("\"ref\": \"0.8.0\""),
        "embedded seed installs exact substrate lock: {lock}"
    );
    let workflow = std::fs::read_to_string(s.path().join(".github/workflows/bedrock.yml")).unwrap();
    assert!(
        workflow.contains("cleverunicornz/bedrock/.github/workflows/graph.yml@v0.8.0")
            && workflow.contains("synchronize")
            && !workflow.contains("cargo install")
            && !workflow.contains("paths:"),
        "promoted workflow is a current-head caller stub pinned to the v0.8.0 \
         central reusable gate, carries no gate logic, and never filters by path: {workflow}"
    );

    // End-to-end: the seeded repo passes a full check with no seed env.
    let out2 = Command::new(bedrock_exe())
        .args(["check", s.path().to_str().unwrap()])
        .current_dir(s.path())
        .env_remove("BEDROCK_SEED")
        .output()
        .unwrap();
    let text2 = format!(
        "{}{}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr)
    );
    assert_eq!(
        out2.status.code(),
        Some(0),
        "embedded-seed init must pass check end-to-end:\n{text2}"
    );
}

#[test]
fn init_default_seed_path_inside_target() {
    // Without BEDROCK_SEED, init resolves seed from <target>/seed when
    // present? No — init resolves the SOURCE seed from the caller's cwd (or
    // BEDROCK_SEED), then installs a copy; check/build afterwards use the
    // installed <root>/seed. Verify the installed-seed path works standalone.
    let s = Scratch::new("init-installed");
    let (c, out, err) = run(&["init", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "init must succeed\n{out}\n{err}");
    assert!(
        s.path().join("seed/context.yamlld").exists(),
        "installed seed keeps context"
    );
    assert!(s.path().join("seed/schemas/definition.json").exists());
    // Run check WITHOUT the env override → resolves <root>/seed.
    let (c2, out2, _) = run_no_seed(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 0, "check via installed seed must pass:\n{out2}");
}

#[test]
fn adopt_writes_mode_adopt_record_and_regenerates() {
    let s = Scratch::new("adopt");
    // An "existing" repo: give it a stale AGENTS.md that differs from any
    // generated output (its pre-adopt donor content).
    std::fs::create_dir_all(s.path().join("situation/definition")).unwrap();
    std::fs::write(
        s.path().join("AGENTS.md"),
        "# legacy hand-written register\n",
    )
    .unwrap();

    let (c, out, err) = run(&["adopt", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out}\n{err}");
    assert_eq!(c, 0, "adopt must succeed:\n{combined}");

    // The old AGENTS.md was regenerated from situation/.
    let md = std::fs::read_to_string(s.path().join("AGENTS.md")).unwrap();
    assert!(
        md.starts_with("# generated by bedrock v"),
        "regenerated graph: {md}"
    );
    assert!(
        md.contains("@prefix bedrock:"),
        "the compiled graph body replaced the register: {md}"
    );
    assert!(
        !md.contains("legacy hand-written"),
        "stale register must be replaced"
    );

    let record = s.path().join("situation/record");
    let fname = std::fs::read_dir(&record)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with("epoch-"))
        .expect("epoch record");
    let text = std::fs::read_to_string(record.join(&fname)).unwrap();
    assert!(text.contains("mode: adopt"), "{text}");
    assert!(text.contains("offline: false"), "{text}");
    // Instructions printed.
    assert!(
        out.contains("epoch") && out.contains("AGENTS.md"),
        "adopt prints the real instruction set: {out}"
    );
    // check passes afterwards.
    let (c2, out2, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c2, 0, "{out2}");
}

#[test]
fn update_refreshes_skewed_installed_base_files() {
    // A repo seeded by init, then skewed: `bedrock update` must restore the
    // installed base files from this binary's embedded copies, print exactly
    // what changed, run check+build, and leave the repo passing.
    let s = Scratch::new("update");
    let (c, out, err) = run(&["init", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "init must succeed\n{out}\n{err}");

    let rewrite_append = |rel: &str, extra: &str| {
        let p = s.path().join(rel);
        let mut text = std::fs::read_to_string(&p).unwrap();
        text.push_str(extra);
        std::fs::write(&p, text).unwrap();
    };
    rewrite_append("seed/schemas/plan.json", "# stale installed schema\n");
    rewrite_append(
        "seed/context.yamlld",
        "\n  stale-term: \"https://yeetz.dev/stale/marker\"\n",
    );
    rewrite_append(
        "situation/references/bedrock-operating.md",
        "\nstale marker\n",
    );
    rewrite_append("seed/substrate-lock.json", "\n");

    // check must fail with C10 naming `bedrock update`.
    let (c2, out2, err2) = run(&["check", s.path().to_str().unwrap()], &manifest());
    let v2 = format!("{out2}\n{err2}");
    assert_eq!(c2, 1, "skewed base files must fail check:\n{v2}");
    assert!(
        v2.contains("C10") && v2.contains("bedrock update"),
        "C10 violation names the fix: {v2}"
    );

    // update refreshes from embedded copies and reports the changed files.
    let (c3, out3, err3) = run(&["update", s.path().to_str().unwrap()], &manifest());
    let combined = format!("{out3}\n{err3}");
    assert_eq!(c3, 0, "update must succeed:\n{combined}");
    assert!(
        out3.contains(
            "bedrock update: check + build pass (AGENTS.md — the resident projection — regenerated)"
        ),
        "update stdout names the regenerated artifact: {out3}"
    );
    assert!(
        out3.contains("seed/schemas/plan.json")
            && out3.contains("seed/context.yamlld")
            && out3.contains("seed/substrate-lock.json")
            && out3.contains("bedrock-operating.md"),
        "update prints exactly what changed: {out3}"
    );

    // Restored to the binary's embedded copies (byte-identical).
    let embedded_plan = std::fs::read(manifest().join("seed/schemas/plan.json")).unwrap();
    let installed_plan = std::fs::read(s.path().join("seed/schemas/plan.json")).unwrap();
    assert_eq!(embedded_plan, installed_plan, "schema restored");
    let embedded_op = std::fs::read(manifest().join("src/embedded/bedrock-operating.md")).unwrap();
    let installed_op =
        std::fs::read(s.path().join("situation/references/bedrock-operating.md")).unwrap();
    assert_eq!(embedded_op, installed_op, "operating reference restored");
    let embedded_lock = std::fs::read(manifest().join("seed/substrate-lock.json")).unwrap();
    let installed_lock = std::fs::read(s.path().join("seed/substrate-lock.json")).unwrap();
    assert_eq!(embedded_lock, installed_lock, "substrate lock restored");

    // check passes again.
    let (c4, out4, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c4, 0, "post-update check passes:\n{out4}");
}

#[test]
fn version_gate_offline_skips_lookup_and_stamps_epoch() {
    let s = Scratch::new("gate-offline");
    let (code, out, err) = run_gate(
        &["init", "--offline", s.path().to_str().unwrap()],
        &manifest(),
        "{malformed",
    );
    assert_eq!(
        code, 0,
        "--offline must skip malformed lookup:\n{out}\n{err}"
    );
    assert!(out.contains("version gate skipped"), "{out}");
    let record = s.path().join("situation/record");
    let path = std::fs::read_dir(&record)
        .unwrap()
        .flatten()
        .next()
        .unwrap()
        .path();
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains("offline: true"), "{text}");
}

#[test]
fn version_gate_current_proceeds_and_other_commands_ignore_lookup() {
    let s = Scratch::new("gate-current");
    let (code, out, err) = run(&["init", s.path().to_str().unwrap()], &manifest());
    assert_eq!(code, 0, "current version must proceed:\n{out}\n{err}");
    assert!(!out.contains("Upgrade with"), "{out}");

    for command in ["check", "build", "update", "migrate-iris"] {
        let (code, out, err) = run_gate(
            &[command, s.path().to_str().unwrap()],
            &manifest(),
            "{malformed",
        );
        assert_eq!(
            code, 0,
            "{command} must never consult the malformed gate response:\n{out}\n{err}"
        );
        assert!(!out.contains("version gate"), "{command}: {out}");
    }
}

#[test]
fn version_gate_stale_refuses_before_writing() {
    let s = Scratch::new("gate-stale");
    let response = version_json("99.0.0");
    let (code, out, err) = run_gate(
        &["init", s.path().to_str().unwrap()],
        &manifest(),
        &response,
    );
    let combined = format!("{out}\n{err}");
    assert_eq!(code, 1, "stale binary must refuse:\n{combined}");
    assert!(combined.contains("v99.0.0"), "{combined}");
    assert!(
        combined.contains("cargo install yeetz-bedrock --locked --force"),
        "{combined}"
    );
    assert!(!s.path().join("situation").exists());
}

#[test]
fn version_gate_semver_treats_stable_as_newer_than_same_prerelease() {
    let s = Scratch::new("gate-newer");
    let response = version_json(&format!("{}-alpha.1", env!("CARGO_PKG_VERSION")));
    let (code, out, err) = run_gate(
        &["init", s.path().to_str().unwrap()],
        &manifest(),
        &response,
    );
    assert_eq!(code, 0, "local stable must proceed:\n{out}\n{err}");
    assert!(out.contains("newer than crates.io"), "{out}");
}

#[test]
fn version_gate_080_clears_published_061() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.8.0");
    let s = Scratch::new("gate-080-over-061");
    let response = version_json("0.6.1");
    let (code, out, err) = run_gate(
        &["init", s.path().to_str().unwrap()],
        &manifest(),
        &response,
    );
    assert_eq!(code, 0, "0.8.0 must clear the 0.6.1 gate:\n{out}\n{err}");
    assert!(out.contains("running v0.8.0, newer than crates.io v0.6.1"));
}

#[test]
fn version_gate_lookup_failures_are_loud() {
    for (tag, response, expected) in [
        ("json", "{malformed", "not valid JSON"),
        (
            "semver",
            "{\"crate\":{\"max_version\":\"not-semver\",\"num_versions\":1}}",
            "invalid max_version",
        ),
        (
            "shape",
            "{\"crate\":{\"num_versions\":1}}",
            "no crate.max_version",
        ),
    ] {
        let s = Scratch::new(&format!("gate-fail-{tag}"));
        let (code, out, err) =
            run_gate(&["init", s.path().to_str().unwrap()], &manifest(), response);
        let combined = format!("{out}\n{err}");
        assert_eq!(code, 1, "{tag} lookup failure must refuse:\n{combined}");
        assert!(combined.contains(expected), "{combined}");
        assert!(combined.contains("--offline"), "{combined}");
        assert!(!s.path().join("situation").exists());
    }
}

#[test]
fn version_gate_zero_versions_prints_first_publication_notice() {
    let s = Scratch::new("gate-zero");
    let response = "{\"crate\":{\"num_versions\":0}}";
    let (code, out, err) = run_gate(&["init", s.path().to_str().unwrap()], &manifest(), response);
    assert_eq!(code, 0, "zero-version registry must proceed:\n{out}\n{err}");
    assert!(out.contains("reports no published"), "{out}");
}

#[test]
fn init_from_bad_seed_layout_is_loud() {
    // A seed without the situation/ skeleton → clear error.
    let bad_seed = fixture_seed().join("..").join("__nonexistent__");
    let s = Scratch::new("badseed");
    let out = Command::new(bedrock_exe())
        .args(["init", s.path().to_str().unwrap()])
        .current_dir(manifest())
        .env("BEDROCK_SEED", &bad_seed)
        .env(
            "BEDROCK_VERSION_JSON",
            version_json(env!("CARGO_PKG_VERSION")),
        )
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text.contains("BEDROCK_SEED"),
        "must name the seed resolution problem: {text}"
    );
}

#[test]
fn installed_workflow_is_promoted() {
    // seed carries a workflow template → init promotes it to .github/workflows.
    let s = Scratch::new("wf");
    // Give the fixture seed a workflow template by pointing BEDROCK_SEED at a
    // temp seed copy with one; then init must promote it.
    let seed_copy = Scratch::new("seed-wf");
    common::copy_dir(&fixture_seed(), &seed_copy.dir.join("seed"));
    std::fs::create_dir_all(seed_copy.dir.join("seed/.github/workflows")).unwrap();
    std::fs::write(
        seed_copy
            .dir
            .join("seed/.github/workflows/bedrock.sample.yml"),
        "name: bedrock\non: [push]\n",
    )
    .unwrap();
    let out = Command::new(bedrock_exe())
        .args(["init", s.path().to_str().unwrap()])
        .current_dir(manifest())
        .env("BEDROCK_SEED", seed_copy.dir.join("seed"))
        .env(
            "BEDROCK_VERSION_JSON",
            version_json(env!("CARGO_PKG_VERSION")),
        )
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    // The workflow template lands in the repo but NOT under seed/ (it was
    // promoted to .github/workflows).
    let promoted = s.path().join(".github/workflows/bedrock.sample.yml");
    assert!(
        promoted.exists(),
        "workflow template must be promoted: {}",
        promoted.display()
    );
    assert!(
        !s.path()
            .join("seed/.github/workflows/bedrock.sample.yml")
            .exists()
    );
}

#[test]
fn check_help_and_version_contracts() {
    let s = Scratch::new("meta");
    for args in [&["help"][..], &[][..]] {
        let (c, out, _) = run(args, s.path());
        assert_eq!(c, 0, "{out}");
        assert!(
            out.contains("init")
                && out.contains("adopt")
                && out.contains("check")
                && out.contains("build")
                && out.contains("update")
                && out.contains("migrate-iris")
        );
    }
    let (c, out, _) = run(&["--version"], s.path());
    assert_eq!(c, 0);
    assert!(out.starts_with("bedrock "), "{out}");
}

#[test]
fn init_accepts_w3_skeleton_layout() {
    // The installable seed includes skeleton, workflow, schemas/context, and
    // substrate lock. init must promote workflow and preserve checker inputs.
    let seed_copy = Scratch::new("seed-w3");
    for ns in [
        "definition",
        "architecture",
        "risk",
        "plan",
        "record",
        "references",
    ] {
        std::fs::create_dir_all(seed_copy.dir.join("seed/situation-skeleton").join(ns)).unwrap();
    }
    std::fs::create_dir_all(seed_copy.dir.join("seed/workflow")).unwrap();
    std::fs::copy(
        fixture_seed().join("context.yamlld"),
        seed_copy.dir.join("seed/context.yamlld"),
    )
    .unwrap();
    common::copy_dir(
        &fixture_seed().join("schemas"),
        &seed_copy.dir.join("seed/schemas"),
    );
    std::fs::copy(
        fixture_seed().join("substrate-lock.json"),
        seed_copy.dir.join("seed/substrate-lock.json"),
    )
    .unwrap();
    std::fs::write(
        seed_copy.dir.join("seed/workflow/bedrock.yml"),
        "name: bedrock\non: [push]\n",
    )
    .unwrap();
    std::fs::write(
        seed_copy.dir.join("seed/gitignore.stanza"),
        "# bedrock: commit situation/ and the generated register\n",
    )
    .unwrap();

    let s = Scratch::new("init-w3");
    let out = std::process::Command::new(bedrock_exe())
        .args(["init", s.path().to_str().unwrap()])
        .current_dir(manifest())
        .env("BEDROCK_SEED", seed_copy.dir.join("seed"))
        .env(
            "BEDROCK_VERSION_JSON",
            version_json(env!("CARGO_PKG_VERSION")),
        )
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    for ns in [
        "definition",
        "architecture",
        "risk",
        "plan",
        "record",
        "references",
    ] {
        assert!(s.path().join("situation").join(ns).is_dir(), "missing {ns}");
    }
    // Workflow promoted, not left inside seed/.
    assert!(s.path().join(".github/workflows/bedrock.yml").exists());
    assert!(!s.path().join("seed/workflow/bedrock.yml").exists());
    // gitignore stanza appended.
    let gi = std::fs::read_to_string(s.path().join(".gitignore")).unwrap();
    assert!(
        gi.contains("commit situation/"),
        "gitignore stanza appended: {gi}"
    );
    // schemas/context/lock kept for the consumer's check.
    assert!(s.path().join("seed/schemas/definition.json").exists());
    assert!(s.path().join("seed/substrate-lock.json").exists());
    // Skeleton-only repo compiles: no vertices, but check passes and
    // AGENTS.md still renders (identity/invariants empty).
    let (c, out, _) = run(&["check", s.path().to_str().unwrap()], &manifest());
    assert_eq!(c, 0, "skeleton-only situation must pass check: {out}");
}
