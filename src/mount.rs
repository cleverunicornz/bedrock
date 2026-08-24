//! Mount Contract v1 boundary support.
//!
//! Registrations are Bedrock architecture vertices. Mount contents remain
//! opaque except for the five boundary checks frozen in the contract: root
//! identity, overlap/AGENTS confinement, LD claims, graph IRI sovereignty,
//! and contained-file SHA-256 verification.

use crate::contextreg;
use crate::errors::{Violation, line_of};
use oxrdf::{GraphName, Literal, NamedNode, NamedOrBlankNode, Quad, Term, Triple};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

pub(crate) const SUPPORTED_MOUNT_CONTRACT_VERSIONS: &[u64] = &[1];

#[derive(Clone, Debug)]
pub(crate) struct Candidate {
    rel: PathBuf,
    text: String,
    value: Value,
}

#[derive(Clone, Debug)]
pub(crate) struct Registration {
    pub(crate) source_rel: PathBuf,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) mount_rel: PathBuf,
    pub(crate) mount_path_iri: String,
    pub(crate) manifest_rel: PathBuf,
    pub(crate) manifest_path_iri: String,
    pub(crate) manifest_sha256: String,
}

/// Phase one: discover ExpansionMount base claims before C1 examines unknown
/// directories under `situation/`. Invalid registrations remain candidates so
/// they cannot turn a mount into an uninspected namespace; later phases fail
/// them through C4/C12.
pub(crate) fn discover(root: &Path) -> Vec<Candidate> {
    let architecture = root.join("situation/architecture");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&architecture)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    paths.sort();

    let mut candidates = Vec::new();
    for path in paths {
        if path.extension().and_then(|value| value.to_str()) != Some("yamlld") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_norway::from_str::<Value>(&text) else {
            continue;
        };
        if type_iris(&value)
            .iter()
            .any(|iri| contextreg::is_ontology_type(iri, "ExpansionMount"))
        {
            candidates.push(Candidate {
                rel: path.strip_prefix(root).unwrap_or(&path).to_path_buf(),
                text,
                value,
            });
        }
    }
    candidates
}

/// Direct situation/ child names claimed by discovered registrations. C1 uses
/// this only as an exclusion set; C12 still proves every registration and root.
pub(crate) fn exclusion_names(candidates: &[Candidate]) -> BTreeSet<String> {
    candidates
        .iter()
        .filter_map(|candidate| candidate.value.get("mount_path").and_then(Value::as_str))
        .filter_map(contextreg::path_from_iri)
        .filter_map(direct_mount_name)
        .map(str::to_string)
        .collect()
}

/// Validate registration semantics and the complete narrow mount boundary.
/// Shape errors remain C4; C12 names mount-specific correction and security
/// failures in the repository's standard line-oriented format.
pub(crate) fn validate(
    root: &Path,
    candidates: &[Candidate],
    out: &mut Vec<Violation>,
) -> Vec<Registration> {
    let mut registrations = Vec::new();

    for candidate in candidates {
        validate_registration_filename(candidate, out);

        let Some(id) = required_string(candidate, "@id", out) else {
            continue;
        };
        let Some(version) = required_u64(candidate, "mount_contract_version", out) else {
            continue;
        };
        let Some(name) = required_string(candidate, "mount_name", out) else {
            continue;
        };
        let Some(mount_path_iri) = required_string(candidate, "mount_path", out) else {
            continue;
        };
        if required_string(candidate, "checker_identity", out).is_none() {
            continue;
        }
        if !valid_string_array(candidate, "checker_arguments", out) {
            continue;
        }
        let Some(init_path_iri) = required_string(candidate, "init_path", out) else {
            continue;
        };
        let Some(init_sha256) = required_string(candidate, "init_sha256", out) else {
            continue;
        };
        let Some(manifest_path_iri) = required_string(candidate, "graph_manifest_path", out) else {
            continue;
        };
        let Some(manifest_sha256) = required_string(candidate, "graph_manifest_sha256", out) else {
            continue;
        };

        if !SUPPORTED_MOUNT_CONTRACT_VERSIONS.contains(&version) {
            out.push(registration_violation(
                candidate,
                "mount_contract_version",
                format!(
                    "mount contract version {version} is unsupported; explicitly migrate the registration to one of {:?} (bedrock never silently rewrites registrations)",
                    SUPPORTED_MOUNT_CONTRACT_VERSIONS
                ),
            ));
        }
        if !valid_mount_name(&name) {
            out.push(registration_violation(
                candidate,
                &name,
                "mount_name must match [a-z0-9][a-z0-9-]*",
            ));
        }
        if !valid_sha256(&init_sha256) {
            out.push(registration_violation(
                candidate,
                &init_sha256,
                "init_sha256 must be exactly 64 lowercase hexadecimal digits",
            ));
        }
        if !valid_sha256(&manifest_sha256) {
            out.push(registration_violation(
                candidate,
                &manifest_sha256,
                "graph_manifest_sha256 must be exactly 64 lowercase hexadecimal digits",
            ));
        }

        let Some(mount_rel) = decode_path_pointer(candidate, "mount_path", &mount_path_iri, out)
        else {
            continue;
        };
        let Some(init_rel) = decode_path_pointer(candidate, "init_path", &init_path_iri, out)
        else {
            continue;
        };
        let Some(manifest_rel) =
            decode_path_pointer(candidate, "graph_manifest_path", &manifest_path_iri, out)
        else {
            continue;
        };

        let registration = Registration {
            source_rel: candidate.rel.clone(),
            id,
            name,
            mount_rel,
            mount_path_iri,
            manifest_rel,
            manifest_path_iri,
            manifest_sha256,
        };

        validate_mount_root(root, candidate, &registration, out);
        if let Some(mount_canon) = canonical_mount_root(root, &registration.mount_rel) {
            walk_mount_boundary(root, candidate, &registration, out);
            let _ = verify_registered_file(
                root,
                &registration.mount_rel,
                &mount_canon,
                &init_rel,
                &init_sha256,
                &candidate.rel,
                line_of(&candidate.text, "init_path"),
                "registered init/pin file",
                out,
            );
            if let Some(manifest_bytes) = verify_registered_file(
                root,
                &registration.mount_rel,
                &mount_canon,
                &registration.manifest_rel,
                &registration.manifest_sha256,
                &candidate.rel,
                line_of(&candidate.text, "graph_manifest_path"),
                "registered graph manifest",
                out,
            ) {
                validate_manifest(
                    root,
                    &registration.mount_rel,
                    &mount_canon,
                    &registration.manifest_rel,
                    &manifest_bytes,
                    out,
                );
            }
        }
        registrations.push(registration);
    }

    validate_uniqueness(&registrations, out);
    registrations
}

fn validate_registration_filename(candidate: &Candidate, out: &mut Vec<Violation>) {
    let valid = candidate
        .rel
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("mount-"))
        .and_then(|name| name.strip_suffix(".yamlld"))
        .map(valid_mount_name)
        .unwrap_or(false);
    if !valid {
        out.push(registration_violation(
            candidate,
            "@type",
            "ExpansionMount registration must be situation/architecture/mount-<slug>.yamlld",
        ));
    }
}

fn validate_uniqueness(registrations: &[Registration], out: &mut Vec<Violation>) {
    for (index, registration) in registrations.iter().enumerate() {
        for prior in &registrations[..index] {
            if registration.name == prior.name {
                out.push(Violation::new(
                    "C12",
                    display_path(&registration.source_rel),
                    1,
                    format!(
                        "duplicate mount name `{}` also registered by {}",
                        registration.name,
                        display_path(&prior.source_rel)
                    ),
                ));
            }
            if registration.mount_rel == prior.mount_rel {
                out.push(Violation::new(
                    "C12",
                    display_path(&registration.source_rel),
                    1,
                    format!(
                        "duplicate mount path `{}` also registered by {}",
                        display_path(&registration.mount_rel),
                        display_path(&prior.source_rel)
                    ),
                ));
            } else if registration.mount_rel.starts_with(&prior.mount_rel)
                || prior.mount_rel.starts_with(&registration.mount_rel)
            {
                out.push(Violation::new(
                    "C12",
                    display_path(&registration.source_rel),
                    1,
                    format!(
                        "overlapping mount paths `{}` and `{}` are forbidden",
                        display_path(&prior.mount_rel),
                        display_path(&registration.mount_rel)
                    ),
                ));
            }
        }
    }
}

fn validate_mount_root(
    root: &Path,
    candidate: &Candidate,
    registration: &Registration,
    out: &mut Vec<Violation>,
) {
    if direct_mount_name(&display_path(&registration.mount_rel)).is_none() {
        out.push(registration_violation(
            candidate,
            &registration.mount_path_iri,
            format!(
                "mount_path must name a real directory directly under situation/, got `{}`",
                display_path(&registration.mount_rel)
            ),
        ));
        return;
    }

    let path = root.join(&registration.mount_rel);
    match std::fs::symlink_metadata(&path) {
        Err(_) => out.push(registration_violation(
            candidate,
            &registration.mount_path_iri,
            format!("registered mount directory `{}` does not exist", display_path(&registration.mount_rel)),
        )),
        Ok(metadata) if metadata.file_type().is_symlink() => out.push(registration_violation(
            candidate,
            &registration.mount_path_iri,
            format!("registered mount directory `{}` is a symlink; mount roots must be real directories", display_path(&registration.mount_rel)),
        )),
        Ok(metadata) if !metadata.is_dir() => out.push(registration_violation(
            candidate,
            &registration.mount_path_iri,
            format!("registered mount root `{}` is not a directory", display_path(&registration.mount_rel)),
        )),
        Ok(_) => {
            let situation = root.join("situation").canonicalize();
            let mount = path.canonicalize();
            if !matches!((situation, mount), (Ok(situation), Ok(mount)) if mount.parent() == Some(situation.as_path())) {
                out.push(registration_violation(
                    candidate,
                    &registration.mount_path_iri,
                    "registered mount root is not canonically contained directly under situation/",
                ));
            }
        }
    }
}

fn canonical_mount_root(root: &Path, mount_rel: &Path) -> Option<PathBuf> {
    let path = root.join(mount_rel);
    let metadata = std::fs::symlink_metadata(&path).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return None;
    }
    let canonical = path.canonicalize().ok()?;
    let situation = root.join("situation").canonicalize().ok()?;
    (canonical.parent() == Some(situation.as_path())).then_some(canonical)
}

fn walk_mount_boundary(
    root: &Path,
    candidate: &Candidate,
    registration: &Registration,
    out: &mut Vec<Violation>,
) {
    let mount = root.join(&registration.mount_rel);
    let mut directories = vec![mount];
    while let Some(directory) = directories.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            out.push(Violation::new(
                "C12",
                display_path(directory.strip_prefix(root).unwrap_or(&directory)),
                1,
                "cannot inspect registered mount boundary",
            ));
            continue;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
        paths.sort();
        let mut child_directories = Vec::new();
        for path in paths {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if name == "AGENTS.md" {
                out.push(Violation::new(
                    "C12",
                    display_path(rel),
                    1,
                    "AGENTS.md is forbidden anywhere inside a registered mount",
                ));
            }
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                continue; // opaque and never followed
            }
            if metadata.is_dir() {
                child_directories.push(path);
            } else if metadata.is_file() && is_structured_ld_source(&path) {
                inspect_ld_claims(&path, rel, candidate, out);
            }
        }
        for child in child_directories.into_iter().rev() {
            directories.push(child);
        }
    }
}

fn is_structured_ld_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("yamlld" | "jsonld")
    )
}

fn inspect_ld_claims(path: &Path, rel: &Path, _candidate: &Candidate, out: &mut Vec<Violation>) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    // Syntax belongs to the expansion checker. An unparseable file has no LD
    // claim Bedrock can recognize and is otherwise opaque.
    let Ok(value) = serde_norway::from_str::<Value>(&text) else {
        return;
    };
    if let Some(iri) = find_bedrock_ld_claim(&value) {
        out.push(Violation::new(
            "C12",
            display_path(rel),
            line_of(&text, iri),
            format!("structured LD source inside a mount claims Bedrock context/base type `{iri}`"),
        ));
    }
}

fn find_bedrock_ld_claim(value: &Value) -> Option<&str> {
    match value {
        Value::Object(values) => {
            if let Some(context) = values.get("@context")
                && let Some(iri) = find_owned_iri(context)
            {
                return Some(iri);
            }
            if let Some(types) = values.get("@type")
                && let Some(iri) = find_base_type(types)
            {
                return Some(iri);
            }
            values.values().find_map(find_bedrock_ld_claim)
        }
        Value::Array(values) => values.iter().find_map(find_bedrock_ld_claim),
        _ => None,
    }
}

fn find_owned_iri(value: &Value) -> Option<&str> {
    match value {
        Value::String(iri) if contextreg::is_bedrock_owned_iri(iri) => Some(iri),
        Value::Array(values) => values.iter().find_map(find_owned_iri),
        Value::Object(values) => values.values().find_map(find_owned_iri),
        _ => None,
    }
}

fn find_base_type(value: &Value) -> Option<&str> {
    match value {
        Value::String(iri) if contextreg::is_base_type(iri) => Some(iri),
        Value::Array(values) => values.iter().find_map(find_base_type),
        _ => None,
    }
}

fn validate_manifest(
    root: &Path,
    mount_rel: &Path,
    mount_canon: &Path,
    manifest_rel: &Path,
    bytes: &[u8],
    out: &mut Vec<Violation>,
) {
    let manifest_path = display_path(manifest_rel);
    let Ok(text) = std::str::from_utf8(bytes) else {
        out.push(Violation::new(
            "C12",
            &manifest_path,
            1,
            "graph manifest must be UTF-8 YAML",
        ));
        return;
    };
    let Ok(value) = serde_norway::from_str::<Value>(text) else {
        out.push(Violation::new(
            "C12",
            &manifest_path,
            1,
            "graph manifest must be valid YAML",
        ));
        return;
    };
    let Some(artifacts) = value.get("artifacts").and_then(Value::as_array) else {
        out.push(Violation::new(
            "C12",
            &manifest_path,
            1,
            "graph manifest requires an `artifacts` array (empty is valid)",
        ));
        return;
    };

    let mut previous: Option<&str> = None;
    for artifact in artifacts {
        let Some(path_value) = artifact.get("path").and_then(Value::as_str) else {
            out.push(Violation::new(
                "C12",
                &manifest_path,
                1,
                "every graph manifest entry requires a repo-relative `path` string",
            ));
            continue;
        };
        let Some(sha256) = artifact.get("sha256").and_then(Value::as_str) else {
            out.push(Violation::new(
                "C12",
                &manifest_path,
                line_of(text, path_value),
                "every graph manifest entry requires a `sha256`",
            ));
            continue;
        };
        if previous.is_some_and(|prior| prior >= path_value) {
            out.push(Violation::new(
                "C12",
                &manifest_path,
                line_of(text, path_value),
                "graph manifest artifacts must be strictly sorted by path with no duplicates",
            ));
        }
        previous = Some(path_value);
        if !valid_sha256(sha256) {
            out.push(Violation::new(
                "C12",
                &manifest_path,
                line_of(text, sha256),
                "artifact sha256 must be exactly 64 lowercase hexadecimal digits",
            ));
            continue;
        }
        let artifact_rel = PathBuf::from(path_value);
        if let Some(artifact_bytes) = verify_registered_file(
            root,
            mount_rel,
            mount_canon,
            &artifact_rel,
            sha256,
            manifest_rel,
            line_of(text, path_value),
            "registered graph artifact",
            out,
        ) {
            inspect_graph_artifact(&artifact_rel, &artifact_bytes, out);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_registered_file(
    root: &Path,
    mount_rel: &Path,
    mount_canon: &Path,
    file_rel: &Path,
    expected_sha256: &str,
    diagnostic_path: &Path,
    line: u32,
    kind: &str,
    out: &mut Vec<Violation>,
) -> Option<Vec<u8>> {
    let diagnostic_path = display_path(diagnostic_path);
    if !normalized_relative(file_rel) || file_rel == mount_rel || !file_rel.starts_with(mount_rel) {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!(
                "{kind} path `{}` must be a normalized repo-relative path contained by mount `{}`",
                display_path(file_rel),
                display_path(mount_rel)
            ),
        ));
        return None;
    }

    let path = root.join(file_rel);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!("{kind} `{}` does not exist", display_path(file_rel)),
        ));
        return None;
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!(
                "{kind} `{}` must be a regular non-symlink file",
                display_path(file_rel)
            ),
        ));
        return None;
    }
    let Ok(canonical) = path.canonicalize() else {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!("cannot canonicalize {kind} `{}`", display_path(file_rel)),
        ));
        return None;
    };
    if !canonical.starts_with(mount_canon) {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!(
                "{kind} `{}` escapes its registered mount",
                display_path(file_rel)
            ),
        ));
        return None;
    }
    let Ok(bytes) = std::fs::read(&path) else {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!("cannot read {kind} `{}`", display_path(file_rel)),
        ));
        return None;
    };
    let actual = sha256_hex(&bytes);
    if actual != expected_sha256 {
        out.push(Violation::new(
            "C12",
            &diagnostic_path,
            line,
            format!(
                "{kind} `{}` SHA-256 mismatch: declared {expected_sha256}, actual {actual}",
                display_path(file_rel)
            ),
        ));
        return None;
    }
    Some(bytes)
}

fn inspect_graph_artifact(path: &Path, bytes: &[u8], out: &mut Vec<Violation>) {
    let rel = display_path(path);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let result = match extension {
        "trig" => inspect_quads(oxttl::TriGParser::new().for_slice(bytes), &rel, out),
        "nq" | "nquads" => inspect_quads(oxttl::NQuadsParser::new().for_slice(bytes), &rel, out),
        "ttl" => inspect_triples(oxttl::TurtleParser::new().for_slice(bytes), &rel, out),
        "nt" | "ntriples" => {
            inspect_triples(oxttl::NTriplesParser::new().for_slice(bytes), &rel, out)
        }
        _ => {
            out.push(Violation::new(
                "C12",
                &rel,
                1,
                "registered graph artifact has unsupported RDF syntax; use .trig, .nq, .ttl, or .nt",
            ));
            return;
        }
    };
    if let Err(message) = result {
        out.push(Violation::new("C12", &rel, 1, message));
    }
}

fn inspect_quads<I, E>(quads: I, rel: &str, out: &mut Vec<Violation>) -> Result<(), String>
where
    I: IntoIterator<Item = Result<Quad, E>>,
    E: std::fmt::Display,
{
    for quad in quads {
        let quad =
            quad.map_err(|error| format!("registered graph artifact is not valid RDF: {error}"))?;
        if let Some(iri) = bedrock_iri_in_quad(&quad) {
            out.push(Violation::new(
                "C12",
                rel,
                1,
                format!("registered graph artifact carries Bedrock-owned RDF IRI `{iri}`"),
            ));
            return Ok(());
        }
    }
    Ok(())
}

fn inspect_triples<I, E>(triples: I, rel: &str, out: &mut Vec<Violation>) -> Result<(), String>
where
    I: IntoIterator<Item = Result<Triple, E>>,
    E: std::fmt::Display,
{
    for triple in triples {
        let triple = triple
            .map_err(|error| format!("registered graph artifact is not valid RDF: {error}"))?;
        if let Some(iri) = bedrock_iri_in_triple(&triple) {
            out.push(Violation::new(
                "C12",
                rel,
                1,
                format!("registered graph artifact carries Bedrock-owned RDF IRI `{iri}`"),
            ));
            return Ok(());
        }
    }
    Ok(())
}

fn bedrock_iri_in_quad(quad: &Quad) -> Option<&str> {
    named_subject_iri(&quad.subject)
        .filter(|iri| contextreg::is_bedrock_owned_iri(iri))
        .or_else(|| {
            contextreg::is_bedrock_owned_iri(quad.predicate.as_str())
                .then_some(quad.predicate.as_str())
        })
        .or_else(|| {
            named_term_iri(&quad.object).filter(|iri| contextreg::is_bedrock_owned_iri(iri))
        })
        .or_else(|| match &quad.graph_name {
            GraphName::NamedNode(graph) if contextreg::is_bedrock_owned_iri(graph.as_str()) => {
                Some(graph.as_str())
            }
            _ => None,
        })
}

fn bedrock_iri_in_triple(triple: &Triple) -> Option<&str> {
    named_subject_iri(&triple.subject)
        .filter(|iri| contextreg::is_bedrock_owned_iri(iri))
        .or_else(|| {
            contextreg::is_bedrock_owned_iri(triple.predicate.as_str())
                .then_some(triple.predicate.as_str())
        })
        .or_else(|| {
            named_term_iri(&triple.object).filter(|iri| contextreg::is_bedrock_owned_iri(iri))
        })
}

fn named_subject_iri(subject: &NamedOrBlankNode) -> Option<&str> {
    match subject {
        NamedOrBlankNode::NamedNode(node) => Some(node.as_str()),
        NamedOrBlankNode::BlankNode(_) => None,
    }
}

fn named_term_iri(term: &Term) -> Option<&str> {
    match term {
        Term::NamedNode(node) => Some(node.as_str()),
        _ => None,
    }
}

/// Canonical Bedrock linkage only: registration references manifest; mount
/// path produces manifest; manifest path carries the exact digest. Expansion
/// quads are never copied into the substrate dataset.
pub(crate) fn linkage_quads(registrations: &[Registration]) -> Vec<Quad> {
    let graph = NamedNode::new(contextreg::namespace_graph("architecture"))
        .expect("canonical architecture graph IRI");
    let references = NamedNode::new(contextreg::predicate("references"))
        .expect("canonical references predicate");
    let produces =
        NamedNode::new(contextreg::predicate("produces")).expect("canonical produces predicate");
    let digest = NamedNode::new(contextreg::predicate("artifact-digest"))
        .expect("canonical artifact-digest predicate");
    let mut quads = Vec::with_capacity(registrations.len() * 3);

    for registration in registrations {
        let (Ok(id), Ok(mount), Ok(manifest)) = (
            NamedNode::new(registration.id.clone()),
            NamedNode::new(registration.mount_path_iri.clone()),
            NamedNode::new(registration.manifest_path_iri.clone()),
        ) else {
            continue; // C3/C4 already report malformed source IRIs
        };
        quads.push(Quad::new(
            id,
            references.clone(),
            manifest.clone(),
            graph.clone(),
        ));
        quads.push(Quad::new(
            mount,
            produces.clone(),
            manifest.clone(),
            graph.clone(),
        ));
        quads.push(Quad::new(
            manifest,
            digest.clone(),
            Literal::new_simple_literal(registration.manifest_sha256.clone()),
            graph.clone(),
        ));
    }
    quads
}

/// C5 mounted-subject helper: lexical and canonical containment under one
/// registered mount. The target may be a file or directory but cannot escape
/// through a symlink.
pub(crate) fn contains_registered_path(
    root: &Path,
    target_rel: &Path,
    registrations: &[Registration],
) -> bool {
    if !normalized_relative(target_rel) {
        return false;
    }
    let Ok(target_canon) = root.join(target_rel).canonicalize() else {
        return false;
    };
    registrations.iter().any(|registration| {
        target_rel.starts_with(&registration.mount_rel)
            && canonical_mount_root(root, &registration.mount_rel)
                .is_some_and(|mount| target_canon.starts_with(mount))
    })
}

fn required_string(candidate: &Candidate, field: &str, out: &mut Vec<Violation>) -> Option<String> {
    match candidate.value.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Some(value.to_string()),
        _ => {
            out.push(registration_violation(
                candidate,
                field,
                format!("ExpansionMount registration requires string field `{field}`"),
            ));
            None
        }
    }
}

fn required_u64(candidate: &Candidate, field: &str, out: &mut Vec<Violation>) -> Option<u64> {
    match candidate.value.get(field).and_then(Value::as_u64) {
        Some(value) => Some(value),
        None => {
            out.push(registration_violation(
                candidate,
                field,
                format!("ExpansionMount registration requires integer field `{field}`"),
            ));
            None
        }
    }
}

fn valid_string_array(candidate: &Candidate, field: &str, out: &mut Vec<Violation>) -> bool {
    let valid = candidate
        .value
        .get(field)
        .and_then(Value::as_array)
        .is_some_and(|values| values.iter().all(|value| value.as_str().is_some()));
    if !valid {
        out.push(registration_violation(
            candidate,
            field,
            format!("ExpansionMount registration requires string-array field `{field}`"),
        ));
    }
    valid
}

fn decode_path_pointer(
    candidate: &Candidate,
    field: &str,
    iri: &str,
    out: &mut Vec<Violation>,
) -> Option<PathBuf> {
    let decoded: Option<PathBuf> = contextreg::path_from_iri(iri).map(PathBuf::from);
    let Some(path) = decoded else {
        out.push(registration_violation(
            candidate,
            iri,
            format!("`{field}` must be a canonical or legacy Bedrock path pointer"),
        ));
        return None;
    };
    if !normalized_relative(&path) {
        out.push(registration_violation(
            candidate,
            iri,
            format!("`{field}` must encode a normalized repo-relative path"),
        ));
        return None;
    }
    Some(path)
}

fn direct_mount_name(path: &str) -> Option<&str> {
    let path = Path::new(path);
    let mut components = path.components();
    match (components.next(), components.next(), components.next()) {
        (Some(Component::Normal(first)), Some(Component::Normal(name)), None)
            if first == "situation" =>
        {
            name.to_str()
                .filter(|name| !name.is_empty() && !name.starts_with('.'))
        }
        _ => None,
    }
}

fn normalized_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_mount_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        write!(&mut out, "{byte:02x}").expect("write to String");
    }
    out
}

fn type_iris(value: &Value) -> Vec<&str> {
    match value.get("@type") {
        Some(Value::String(value)) => vec![value],
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn registration_violation(
    candidate: &Candidate,
    needle: &str,
    message: impl Into<String>,
) -> Violation {
    Violation::new(
        "C12",
        display_path(&candidate.rel),
        line_of(&candidate.text, needle),
        message,
    )
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
