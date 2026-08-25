//! Executable 0.3 bridge proof: a formed repository with legacy authored IRIs
//! updates without source mutation, checks green, explicitly migrates, and
//! checks/builds green with canonical URNs.

mod common;

use common::{Scratch, manifest, run_no_seed};
use std::path::{Path, PathBuf};

fn yamlld_files(root: &Path) -> Vec<PathBuf> {
    fn visit(directory: &Path, out: &mut Vec<PathBuf>) {
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
                visit(&path, out);
            } else if path.extension().and_then(|value| value.to_str()) == Some("yamlld") {
                out.push(path);
            }
        }
    }

    let mut files = Vec::new();
    visit(&root.join("situation"), &mut files);
    files
}

fn command_ok(args: &[&str], cwd: &Path) -> String {
    let (code, stdout, stderr) = run_no_seed(args, cwd);
    assert_eq!(code, 0, "command failed: {args:?}\n{stdout}\n{stderr}");
    stdout
}

#[test]
fn legacy_formed_repository_updates_and_explicitly_migrates() {
    let scratch = Scratch::new("legacy-bridge");
    let root = scratch.path().to_str().unwrap();
    command_ok(&["init", root], &manifest());

    // Materialize an actual legacy-authored repository, then regenerate its
    // committed projections with the dual-read checker.
    for path in yamlld_files(scratch.path()) {
        let text = std::fs::read_to_string(&path).unwrap();
        let legacy = text
            .replace("urn:bedrock:graph/", "https://yeetz.dev/graph/")
            .replace("urn:bedrock:", "https://yeetz.dev/bedrock/");
        std::fs::write(path, legacy).unwrap();
    }
    command_ok(&["build", root], &manifest());

    let authored_before: Vec<(PathBuf, Vec<u8>)> = yamlld_files(scratch.path())
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();

    // Simulate stale machine-owned v0.6.1 base material. The updater owns
    // these files, but not authored YAML-LD or an existing consumer workflow.
    let context = scratch.path().join("seed/context.yamlld");
    let context_text = std::fs::read_to_string(&context)
        .unwrap()
        .replace("urn:bedrock:", "https://yeetz.dev/bedrock/");
    std::fs::write(&context, context_text).unwrap();
    let lock = scratch.path().join("seed/substrate-lock.json");
    let lock_text = std::fs::read_to_string(&lock)
        .unwrap()
        .replace("\"ref\": \"0.8.0\"", "\"ref\": \"0.6.1\"");
    std::fs::write(&lock, lock_text).unwrap();
    let operating = scratch
        .path()
        .join("situation/references/bedrock-operating.md");
    let mut operating_text = std::fs::read_to_string(&operating).unwrap();
    operating_text.push_str("\nstale bridge fixture\n");
    std::fs::write(&operating, operating_text).unwrap();
    let workflow = scratch.path().join(".github/workflows/bedrock.yml");
    let consumer_workflow = "# consumer-owned workflow; migrate once by hand\nname: custom\n";
    std::fs::write(&workflow, consumer_workflow).unwrap();

    let update_output = command_ok(&["update", root], &manifest());
    assert!(update_output.contains("seed/substrate-lock.json"));
    assert_eq!(
        std::fs::read_to_string(&workflow).unwrap(),
        consumer_workflow,
        "bedrock update must never rewrite an existing consumer workflow"
    );
    for (path, bytes) in &authored_before {
        assert_eq!(
            std::fs::read(path).unwrap(),
            *bytes,
            "update mutated authored source {}",
            path.display()
        );
    }

    // Required pre-migration polarity: current base files + legacy source are
    // fully green before the optional source rewrite.
    command_ok(&["check", root], &manifest());
    let migration_output = command_ok(&["migrate-iris", root], &manifest());
    assert!(migration_output.contains("rewrote"));

    for path in yamlld_files(scratch.path()) {
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("urn:bedrock:"), "{}: {text}", path.display());
        assert!(
            !text.contains("https://yeetz.dev/bedrock/"),
            "legacy coordinate survived in {}: {text}",
            path.display()
        );
    }
    command_ok(&["check", root], &manifest());
    command_ok(&["build", root], &manifest());
}
