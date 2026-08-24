//! Synthetic-only Mount Contract v1 coverage. No expansion-specific schemas or
//! concepts enter Bedrock; `example-expansion` is opaque rig material.

mod common;

use common::{Scratch, build_and_check_ok, manifest, run};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
enum Coordinates {
    Urn,
    Legacy,
}

impl Coordinates {
    fn context(self) -> &'static str {
        match self {
            Self::Urn => "urn:bedrock:context/v1",
            Self::Legacy => "https://yeetz.dev/bedrock/context/v1",
        }
    }

    fn vertex_prefix(self) -> &'static str {
        match self {
            Self::Urn => "urn:bedrock:vertex/",
            Self::Legacy => "https://yeetz.dev/bedrock/vertex/",
        }
    }

    fn path_prefix(self) -> &'static str {
        match self {
            Self::Urn => "urn:bedrock:path/",
            Self::Legacy => "https://yeetz.dev/bedrock/path/",
        }
    }

    fn ontology_prefix(self) -> &'static str {
        match self {
            Self::Urn => "urn:bedrock:ontology/",
            Self::Legacy => "https://yeetz.dev/bedrock/ontology/",
        }
    }

    fn predicate_prefix(self) -> &'static str {
        match self {
            Self::Urn => "urn:bedrock:",
            Self::Legacy => "https://yeetz.dev/bedrock/",
        }
    }

    fn graph_prefix(self) -> &'static str {
        match self {
            Self::Urn => "urn:bedrock:graph/",
            Self::Legacy => "https://yeetz.dev/graph/",
        }
    }
}

#[derive(Debug)]
struct MountFiles {
    registration: PathBuf,
    mount_root: PathBuf,
    graph: PathBuf,
    manifest: PathBuf,
    manifest_sha256: String,
}

fn base_repo(tag: &str, coordinates: Coordinates) -> Scratch {
    let scratch = Scratch::new(tag);
    for namespace in [
        "definition",
        "architecture",
        "risk",
        "plan",
        "record",
        "references",
    ] {
        std::fs::create_dir_all(scratch.path().join("situation").join(namespace)).unwrap();
    }
    std::fs::write(
        scratch.path().join("situation/definition/invariant.yamlld"),
        format!(
            "\"@context\": \"{}\"\n\"@id\": \"{}fixture-invariant\"\n\"@type\": \"{}Invariant\"\nlabel: \"Fixture invariant\"\nlayer: floor\nstatement: \"Synthetic mount fixtures remain generic.\"\n",
            coordinates.context(),
            coordinates.vertex_prefix(),
            coordinates.ontology_prefix(),
        ),
    )
    .unwrap();
    scratch
}

fn add_mount(
    scratch: &Scratch,
    name: &str,
    coordinates: Coordinates,
    graph_body: &str,
) -> MountFiles {
    let mount_root = scratch.path().join("situation").join(name);
    let run_dir = mount_root.join("runs/run-1");
    std::fs::create_dir_all(&run_dir).unwrap();

    let init = mount_root.join("example-init.yaml");
    let init_bytes = b"contract: example-expansion/v1\n";
    std::fs::write(&init, init_bytes).unwrap();
    let init_sha256 = bedrock::generate::digest_hex(init_bytes);

    let graph = run_dir.join("graph.trig");
    std::fs::write(&graph, graph_body.as_bytes()).unwrap();
    let graph_sha256 = bedrock::generate::digest_hex(graph_body.as_bytes());

    let graph_rel = format!("situation/{name}/runs/run-1/graph.trig");
    let manifest_text = format!("artifacts:\n  - path: {graph_rel}\n    sha256: {graph_sha256}\n");
    let manifest = mount_root.join("graph-manifest.yaml");
    std::fs::write(&manifest, manifest_text.as_bytes()).unwrap();
    let manifest_sha256 = bedrock::generate::digest_hex(manifest_text.as_bytes());

    let registration = scratch
        .path()
        .join("situation/architecture")
        .join(format!("mount-{name}.yamlld"));
    let registration_text = format!(
        "\"@context\": \"{}\"\n\"@id\": \"{}mount-{name}\"\n\"@type\":\n  - \"urn:example:ontology/ExampleMount\"\n  - \"{}ExpansionMount\"\nlabel: \"{name}\"\nmount_contract_version: 1\nmount_name: {name}\nmount_path: \"{}situation/{name}\"\nchecker_identity: \"urn:example:checker/v1\"\nchecker_arguments:\n  - check\ninit_path: \"{}situation/{name}/example-init.yaml\"\ninit_sha256: \"{init_sha256}\"\ngraph_manifest_path: \"{}situation/{name}/graph-manifest.yaml\"\ngraph_manifest_sha256: \"{manifest_sha256}\"\n",
        coordinates.context(),
        coordinates.vertex_prefix(),
        coordinates.ontology_prefix(),
        coordinates.path_prefix(),
        coordinates.path_prefix(),
        coordinates.path_prefix(),
    );
    std::fs::write(&registration, registration_text).unwrap();

    MountFiles {
        registration,
        mount_root,
        graph,
        manifest,
        manifest_sha256,
    }
}

fn clean_graph(name: &str) -> String {
    format!("@prefix ex: <urn:example:> .\n<urn:example:run/{name}> ex:result \"ok\" .\n")
}

fn check_failure(scratch: &Scratch, rule: &str) -> String {
    let (code, stdout, stderr) = run(&["check", scratch.path().to_str().unwrap()], &manifest());
    let combined = format!("{stdout}\n{stderr}");
    assert_eq!(code, 1, "check must fail:\n{combined}");
    assert!(combined.contains(rule), "expected {rule}:\n{combined}");
    combined
}

fn update_manifest_digest(files: &mut MountFiles, manifest_text: &str) {
    std::fs::write(&files.manifest, manifest_text.as_bytes()).unwrap();
    let digest = bedrock::generate::digest_hex(manifest_text.as_bytes());
    let registration = std::fs::read_to_string(&files.registration).unwrap();
    let registration = registration.replace(&files.manifest_sha256, &digest);
    assert_ne!(
        registration,
        std::fs::read_to_string(&files.registration).unwrap()
    );
    std::fs::write(&files.registration, registration).unwrap();
    files.manifest_sha256 = digest;
}

fn snapshot_files(root: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(
        base: &std::path::Path,
        directory: &std::path::Path,
        out: &mut Vec<(PathBuf, Vec<u8>)>,
    ) {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .collect();
        entries.sort();
        for path in entries {
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                visit(base, &path, out);
            } else if metadata.is_file() {
                out.push((
                    path.strip_prefix(base).unwrap().to_path_buf(),
                    std::fs::read(&path).unwrap(),
                ));
            }
        }
    }

    let mut files = Vec::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn valid_mount_manifest_linkage_and_discovery_are_deterministic() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let scratch = base_repo("mount-valid", coordinates);
        let files = add_mount(
            &scratch,
            "example-expansion",
            coordinates,
            &clean_graph("example-expansion"),
        );
        build_and_check_ok(&scratch);

        let graph = std::fs::read_to_string(scratch.path().join("AGENTS.md")).unwrap();
        assert!(graph.contains("bedrock:references"), "{graph}");
        assert!(graph.contains("bedrock:produces"), "{graph}");
        assert!(graph.contains(&files.manifest_sha256), "{graph}");
        assert!(
            !graph.contains("urn:example:run/example-expansion"),
            "expansion quads must never be included: {graph}"
        );
        assert!(
            !scratch.path().join("situation/graph.trig").exists(),
            "Bedrock keeps one generated artifact; mount-owned run graphs remain inside the mount"
        );

        let agents = std::fs::read_to_string(scratch.path().join("AGENTS.md")).unwrap();
        assert!(agents.contains("# Mounted expansions:"), "{agents}");
        assert!(
            agents.contains("# - example-expansion — path: situation/example-expansion; checker: urn:example:checker/v1"),
            "one registration line required: {agents}"
        );
    }
}

#[test]
fn unregistered_mount_and_registration_without_mount_fail_closed() {
    let unregistered = base_repo("mount-unregistered", Coordinates::Urn);
    std::fs::create_dir_all(unregistered.path().join("situation/example-expansion")).unwrap();
    let combined = check_failure(&unregistered, "C1");
    assert!(combined.contains("unregistered directory `example-expansion`"));

    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let missing = base_repo("mount-missing-root", coordinates);
        let files = add_mount(
            &missing,
            "example-expansion",
            coordinates,
            &clean_graph("missing"),
        );
        std::fs::remove_dir_all(files.mount_root).unwrap();
        let combined = check_failure(&missing, "C12");
        assert!(combined.contains("does not exist"), "{combined}");
    }
}

#[test]
fn duplicate_and_overlapping_mounts_fail_closed() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let duplicate = base_repo("mount-duplicate", coordinates);
        let first = add_mount(
            &duplicate,
            "example-expansion",
            coordinates,
            &clean_graph("duplicate"),
        );
        let duplicate_text = std::fs::read_to_string(&first.registration)
            .unwrap()
            .replace("mount-example-expansion", "mount-example-expansion-copy");
        std::fs::write(
            duplicate
                .path()
                .join("situation/architecture/mount-example-copy.yamlld"),
            duplicate_text,
        )
        .unwrap();
        let combined = check_failure(&duplicate, "C12");
        assert!(
            combined.contains("duplicate mount name") && combined.contains("duplicate mount path"),
            "{combined}"
        );

        let overlap = base_repo("mount-overlap", coordinates);
        add_mount(
            &overlap,
            "example-expansion",
            coordinates,
            &clean_graph("outer"),
        );
        let nested = add_mount(
            &overlap,
            "second-expansion",
            coordinates,
            &clean_graph("inner"),
        );
        let text = std::fs::read_to_string(&nested.registration).unwrap();
        let text = text.replace(
            &format!("{}situation/second-expansion\"", coordinates.path_prefix()),
            &format!(
                "{}situation/example-expansion/runs\"",
                coordinates.path_prefix()
            ),
        );
        std::fs::write(&nested.registration, text).unwrap();
        let combined = check_failure(&overlap, "C12");
        assert!(combined.contains("overlapping mount paths"), "{combined}");
    }
}

#[cfg(unix)]
#[test]
fn symlinked_mount_roots_fail_closed() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let scratch = base_repo("mount-symlink", coordinates);
        let files = add_mount(
            &scratch,
            "example-expansion",
            coordinates,
            &clean_graph("symlink"),
        );
        let outside = scratch.path().join("outside-example-expansion");
        std::fs::rename(&files.mount_root, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, &files.mount_root).unwrap();
        let combined = check_failure(&scratch, "C12");
        assert!(
            combined.contains("symlink") && combined.contains("mount roots must be real"),
            "{combined}"
        );
    }
}

#[test]
fn nested_agents_and_bedrock_ld_claims_are_rejected() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let agents = base_repo("mount-agents", coordinates);
        let files = add_mount(
            &agents,
            "example-expansion",
            coordinates,
            &clean_graph("agents"),
        );
        std::fs::create_dir_all(files.mount_root.join("nested")).unwrap();
        std::fs::write(files.mount_root.join("nested/AGENTS.md"), "# forbidden\n").unwrap();
        let combined = check_failure(&agents, "C12");
        assert!(combined.contains("AGENTS.md is forbidden"), "{combined}");

        let context_claim = base_repo("mount-context-claim", coordinates);
        let files = add_mount(
            &context_claim,
            "example-expansion",
            coordinates,
            &clean_graph("context-claim"),
        );
        std::fs::write(
            files.mount_root.join("context-claim.yamlld"),
            format!(
                "\"@context\": \"{}\"\n\"@id\": \"urn:example:claim\"\n\"@type\": \"urn:example:Claim\"\n",
                coordinates.context()
            ),
        )
        .unwrap();
        let combined = check_failure(&context_claim, "C12");
        assert!(
            combined.contains("claims Bedrock context/base type"),
            "{combined}"
        );

        let type_claim = base_repo("mount-type-claim", coordinates);
        let files = add_mount(
            &type_claim,
            "example-expansion",
            coordinates,
            &clean_graph("type-claim"),
        );
        std::fs::write(
            files.mount_root.join("type-claim.jsonld"),
            format!(
                "{{\"@context\": {{\"ex\": \"urn:example:\"}}, \"@id\": \"urn:example:claim\", \"@type\": \"{}Risk\"}}\n",
                coordinates.ontology_prefix()
            ),
        )
        .unwrap();
        let combined = check_failure(&type_claim, "C12");
        assert!(
            combined.contains("claims Bedrock context/base type"),
            "{combined}"
        );
    }
}

#[test]
fn every_rdf_iri_position_rejects_bedrock_ownership() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let cases = [
            format!(
                "<{}leak> <urn:example:p> <urn:example:o> .\n",
                coordinates.vertex_prefix()
            ),
            format!(
                "<urn:example:s> <{}leak> <urn:example:o> .\n",
                coordinates.predicate_prefix()
            ),
            format!(
                "<urn:example:s> <urn:example:p> <{}leak> .\n",
                coordinates.vertex_prefix()
            ),
            format!(
                "GRAPH <{}risk> {{ <urn:example:s> <urn:example:p> <urn:example:o> . }}\n",
                coordinates.graph_prefix()
            ),
        ];
        for (index, graph) in cases.iter().enumerate() {
            let scratch = base_repo("mount-iri-leak", coordinates);
            add_mount(&scratch, "example-expansion", coordinates, graph);
            let combined = check_failure(&scratch, "C12");
            assert!(
                combined.contains("Bedrock-owned RDF IRI"),
                "position {index}: {combined}"
            );
        }
    }
}

#[test]
fn missing_and_tampered_registered_artifacts_fail_digest_checks() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let artifact_tamper = base_repo("mount-artifact-tamper", coordinates);
        let files = add_mount(
            &artifact_tamper,
            "example-expansion",
            coordinates,
            &clean_graph("artifact-tamper"),
        );
        std::fs::write(&files.graph, clean_graph("changed-after-manifest")).unwrap();
        let combined = check_failure(&artifact_tamper, "C12");
        assert!(combined.contains("SHA-256 mismatch"), "{combined}");

        let manifest_tamper = base_repo("mount-manifest-tamper", coordinates);
        let files = add_mount(
            &manifest_tamper,
            "example-expansion",
            coordinates,
            &clean_graph("manifest-tamper"),
        );
        let mut manifest = std::fs::read_to_string(&files.manifest).unwrap();
        manifest.push_str("# tampered\n");
        std::fs::write(&files.manifest, manifest).unwrap();
        let combined = check_failure(&manifest_tamper, "C12");
        assert!(combined.contains("graph manifest") && combined.contains("SHA-256 mismatch"));

        let missing = base_repo("mount-artifact-missing", coordinates);
        let files = add_mount(
            &missing,
            "example-expansion",
            coordinates,
            &clean_graph("missing-artifact"),
        );
        std::fs::remove_file(&files.graph).unwrap();
        let combined = check_failure(&missing, "C12");
        assert!(
            combined.contains("registered graph artifact") && combined.contains("does not exist")
        );
    }
}

#[test]
fn manifest_paths_are_sorted_and_multiple_mounts_remain_one_line_each() {
    let sorted = base_repo("mount-manifest-sort", Coordinates::Urn);
    let mut files = add_mount(
        &sorted,
        "example-expansion",
        Coordinates::Urn,
        &clean_graph("sort-a"),
    );
    let second_graph = files.mount_root.join("runs/run-2/graph.trig");
    std::fs::create_dir_all(second_graph.parent().unwrap()).unwrap();
    let second_body = clean_graph("sort-b");
    std::fs::write(&second_graph, second_body.as_bytes()).unwrap();
    let first_sha = bedrock::generate::digest_hex(&std::fs::read(&files.graph).unwrap());
    let second_sha = bedrock::generate::digest_hex(second_body.as_bytes());
    let unsorted = format!(
        "artifacts:\n  - path: situation/example-expansion/runs/run-2/graph.trig\n    sha256: {second_sha}\n  - path: situation/example-expansion/runs/run-1/graph.trig\n    sha256: {first_sha}\n"
    );
    update_manifest_digest(&mut files, &unsorted);
    let combined = check_failure(&sorted, "C12");
    assert!(combined.contains("strictly sorted"), "{combined}");

    let multiple = base_repo("mount-multiple", Coordinates::Urn);
    add_mount(
        &multiple,
        "alpha-expansion",
        Coordinates::Urn,
        &clean_graph("alpha"),
    );
    add_mount(
        &multiple,
        "beta-expansion",
        Coordinates::Urn,
        &clean_graph("beta"),
    );
    build_and_check_ok(&multiple);
    let agents = std::fs::read_to_string(multiple.path().join("AGENTS.md")).unwrap();
    assert_eq!(agents.matches("— path: situation/").count(), 2, "{agents}");
}

#[test]
fn unsupported_older_contract_version_refuses_with_migration() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let scratch = base_repo("mount-old-contract", coordinates);
        let files = add_mount(
            &scratch,
            "example-expansion",
            coordinates,
            &clean_graph("old-contract"),
        );
        let registration = std::fs::read_to_string(&files.registration)
            .unwrap()
            .replace("mount_contract_version: 1", "mount_contract_version: 0");
        std::fs::write(&files.registration, registration).unwrap();
        let combined = check_failure(&scratch, "C12");
        assert!(
            combined.contains("unsupported")
                && combined.contains("explicitly migrate")
                && combined.contains("never silently rewrites"),
            "{combined}"
        );
    }
}

fn write_verdict(scratch: &Scratch, coordinates: Coordinates, subject_path: &str, name: &str) {
    std::fs::write(
        scratch
            .path()
            .join("situation/record")
            .join(format!("{name}.yamlld")),
        format!(
            "\"@context\": \"{}\"\n\"@id\": \"{}{name}\"\n\"@type\": \"{}ReflectVerdict\"\nlabel: \"Mounted subject verdict\"\nsubject: \"{}{subject_path}\"\ncriteria:\n  - \"The registered mount path remained contained.\"\n",
            coordinates.context(),
            coordinates.vertex_prefix(),
            coordinates.ontology_prefix(),
            coordinates.path_prefix(),
        ),
    )
    .unwrap();
}

#[test]
fn reflect_verdict_subject_accepts_only_registered_mount_paths() {
    for coordinates in [Coordinates::Urn, Coordinates::Legacy] {
        let valid = base_repo("mount-verdict-valid", coordinates);
        add_mount(
            &valid,
            "example-expansion",
            coordinates,
            &clean_graph("verdict"),
        );
        write_verdict(
            &valid,
            coordinates,
            "situation/example-expansion/runs/run-1",
            "mounted-verdict",
        );
        build_and_check_ok(&valid);

        let invalid = base_repo("mount-verdict-invalid", coordinates);
        add_mount(
            &invalid,
            "example-expansion",
            coordinates,
            &clean_graph("verdict-invalid"),
        );
        write_verdict(
            &invalid,
            coordinates,
            "situation/references",
            "outside-verdict",
        );
        let combined = check_failure(&invalid, "C5");
        assert!(
            combined.contains("not contained by a registered mount"),
            "{combined}"
        );
    }
}

#[test]
fn update_leaves_mount_and_existing_workflow_bytes_untouched() {
    let scratch = base_repo("mount-update-lanes", Coordinates::Urn);
    let files = add_mount(
        &scratch,
        "example-expansion",
        Coordinates::Urn,
        &clean_graph("update-lanes"),
    );
    build_and_check_ok(&scratch);
    let mount_before = snapshot_files(&files.mount_root);

    let workflow = scratch.path().join(".github/workflows/bedrock.yml");
    std::fs::create_dir_all(workflow.parent().unwrap()).unwrap();
    let consumer_workflow = b"name: consumer-owned-combined-witness\n";
    std::fs::write(&workflow, consumer_workflow).unwrap();

    let (code, stdout, stderr) = run(&["update", scratch.path().to_str().unwrap()], &manifest());
    assert_eq!(code, 0, "update failed:\n{stdout}\n{stderr}");
    assert_eq!(snapshot_files(&files.mount_root), mount_before);
    assert_eq!(std::fs::read(&workflow).unwrap(), consumer_workflow);
    let (code, stdout, stderr) = run(&["check", scratch.path().to_str().unwrap()], &manifest());
    assert_eq!(code, 0, "post-update check failed:\n{stdout}\n{stderr}");
}

#[test]
fn explicit_iri_migration_never_enters_mount_contents() {
    let scratch = base_repo("mount-migrate", Coordinates::Legacy);
    let files = add_mount(
        &scratch,
        "example-expansion",
        Coordinates::Legacy,
        &clean_graph("migrate"),
    );
    let notes = files.mount_root.join("notes.txt");
    let legacy_note = "opaque note: https://yeetz.dev/bedrock/vertex/do-not-touch\n";
    std::fs::write(&notes, legacy_note).unwrap();
    build_and_check_ok(&scratch);

    let (code, stdout, stderr) = run(
        &["migrate-iris", scratch.path().to_str().unwrap()],
        &manifest(),
    );
    assert_eq!(code, 0, "migration failed:\n{stdout}\n{stderr}");
    assert_eq!(std::fs::read_to_string(notes).unwrap(), legacy_note);
    let registration = std::fs::read_to_string(files.registration).unwrap();
    assert!(registration.contains("urn:bedrock:ontology/ExpansionMount"));
    assert!(!registration.contains("https://yeetz.dev/bedrock/"));
}
