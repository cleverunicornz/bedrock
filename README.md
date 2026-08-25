# bedrock

`bedrock` validates a repository's complete YAML-LD situation and compiles its
current resident working set into one injected artifact: root `AGENTS.md`, a
comment preamble followed by deterministic TriG.

Crate: `yeetz-bedrock`. Binary: `bedrock`.

## Install

```sh
cargo install --locked yeetz-bedrock
```

## Commands

- `bedrock init [DIR]` — form a new repository; crates.io version-gated.
- `bedrock adopt [DIR]` — epoch-change an existing repository; version-gated.
- `bedrock check [DIR]` — validate complete source plus resident projection,
  C1–C12.
- `bedrock build [DIR]` — regenerate root AGENTS.md, the only artifact.
- `bedrock update [DIR]` — refresh only Bedrock-owned base files and a missing
  workflow; never authored vertices, mounts, or an existing workflow.
- `bedrock migrate-iris [DIR]` — explicitly migrate base-namespace legacy
  Bedrock IRIs to canonical URNs, never mount contents.

`--offline` deliberately bypasses only the init/adopt version lookup and is
stamped into the epoch record.

## 0.7.0 identity

Canonical public coordinates use `urn:bedrock:`:

| identity | coordinate |
|---|---|
| context | `urn:bedrock:context/v1` |
| ontology | `urn:bedrock:ontology/<Term>` |
| vertex | `urn:bedrock:vertex/<slug>` |
| path | `urn:bedrock:path/<repo-relative>` |
| predicate | `urn:bedrock:<predicate>` |
| graph | `urn:bedrock:graph/<namespace>` |

New authoring emits URNs only. Reads continue accepting former
`https://yeetz.dev/bedrock/...` source and `https://yeetz.dev/graph/...` named
graphs. Update never rewrites authored source; migration is explicit.

## Resident graph

`situation/` remains the complete canonical store. AGENTS.md contains current
operational knowledge:

- definition, architecture, current risks, Decisions: resident;
- active Plans: compact routing faces resident;
- draft/done/abandoned Plans, episodic records, references, bodies: cold;
- ExpansionMount registrations and Bedrock pointer linkage: resident.

Every vertex is one YAML-LD file. Optional `body: |` is unbounded node-local
depth and never compiles. C11 prevents resident vertex edges from targeting
cold vertices. The soft 4096-character face budget is advisory only.

## Six base namespaces plus registered opaque mounts

The base namespaces remain exactly:

`definition`, `architecture`, `risk`, `plan`, `record`, `references`.

A mount becomes legal only through one consumer-authored
`situation/architecture/mount-<slug>.yamlld` vertex with base type
`urn:bedrock:ontology/ExpansionMount`. Required generic fields:

```yaml
"@context": "urn:bedrock:context/v1"
"@id": "urn:bedrock:vertex/mount-example-expansion"
"@type":
  - "urn:example:ontology/ExampleMount"
  - "urn:bedrock:ontology/ExpansionMount"
label: "example-expansion"
mount_contract_version: 1
mount_name: example-expansion
mount_path: "urn:bedrock:path/situation/example-expansion"
checker_identity: "urn:example:checker/v1"
checker_arguments:
  - check
init_path: "urn:bedrock:path/situation/example-expansion/example-init.yaml"
init_sha256: "<64 lowercase hex digits>"
graph_manifest_path: "urn:bedrock:path/situation/example-expansion/graph-manifest.yaml"
graph_manifest_sha256: "<64 lowercase hex digits>"
```

Checker data is never executed. The mount root is a unique real non-symlink
direct child of `situation/`.

The mount owns one stable manifest:

```yaml
artifacts:
  - path: situation/example-expansion/runs/run-1/graph.trig
    sha256: "<64 lowercase hex digits>"
```

`artifacts: []` is valid. Paths are strictly sorted normalized repo-relative
strings.

## C12 opaque boundary

Bedrock never compiles or base-schema-validates mount contents. It only:

1. proves registration/root/version/uniqueness;
2. rejects overlaps and any nested `AGENTS.md`;
3. inspects structured LD source for Bedrock context/base-type claims;
4. parses registered RDF only to reject Bedrock-owned IRIs in every RDF
   position;
5. verifies containment, regular files, and SHA-256 for init, manifest, and
   manifest-listed graphs.

Other bytes are opaque. Registered graph formats are `.trig`, `.nq`, `.ttl`,
and `.nt`.

## One-artifact pointer linkage

Bedrock emits registration triples and canonical pointer linkage inside the
resident architecture graph in AGENTS.md:

```text
registration --references------> manifest path
mount path   --produces---------> manifest path
manifest path --artifact-digest-> exact manifest SHA-256
```

AGENTS.md's comment preamble also has a deterministic `Mounted expansions`
section, one line per registration. Expansion graph quads are never included.
Mount-owned run graphs remain inside mounts.

This is the Mount Contract v1 adaptation to Bedrock's post-0.4 single-artifact
model: no separate substrate graph file is reintroduced.

## ReflectVerdict and existing anatomy

A ReflectVerdict may subject an existing base vertex or existing path
canonically contained by a registered mount. It remains an episodic cold
record. Decision residency/supersession, Plan face/body rules, witness gates,
and projection closure remain unchanged.

## Substrate lock and central CI

`seed/substrate-lock.json` pins the consumer's exact Bedrock checker package/ref
and supported Mount Contract versions. C10 guards it. The installed workflow is
a caller stub pinned to the immutable `v0.8.0` reusable gate in this repository;
the central gate resolves that lock, runs check/build, and verifies AGENTS.md.

The caller gates every current PR head, including `synchronize`. Its first step
rejects fork PRs before checkout or execution of fork-controlled bytes. Init
installs a missing caller; update preserves present workflow bytes. New gate
tags are adopted by reviewed stub bumps, and the release propagation workflow
opens that one-file change in existing adopters. Mounted repositories retain
independent expansion CI; they do not replace the Bedrock caller.

Failures remain:

```text
RULE path:line message
```

Fix the cause; never hand-edit AGENTS.md.
