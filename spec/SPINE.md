# Bedrock Spine — binding substrate spec

This document is binding. Donor material under `spec/donor-*` is evidence,
never law. `spec/floor-v2.md` defines the seed floor.

## 1. Product and commands

Bedrock is one Rust binary, published as crate `yeetz-bedrock` and installed as
`bedrock`.

- `bedrock init [--offline] [DIR]` forms a new repository and writes a
  canonical-URN epoch record.
- `bedrock adopt [--offline] [DIR]` forms an existing repository at an epoch
  cut and writes a canonical-URN epoch record.
- `bedrock check [DIR]` validates complete source and the resident projection
  under C1–C12; CI entrypoint.
- `bedrock build [DIR]` validates source and regenerates the one artifact:
  root `AGENTS.md`, comment preamble plus resident TriG.
- `bedrock update [DIR]` refreshes only Bedrock-owned installed base files and
  a missing workflow, then checks/builds.
- `bedrock migrate-iris [DIR]` explicitly rewrites bridge-era Bedrock IRIs in
  base-namespace YAML-LD, never mount contents, then regenerates AGENTS.md.
- `bedrock help`, `bedrock --version` expose the installed contract/version.

The crates.io version gate runs only for `init`/`adopt`. Current or newer
proceeds; stale or unverifiable refuses before writes. `--offline` deliberately
bypasses lookup and stamps the local version plus `offline: true`. A 404 or
explicit zero-version response is the only inactive first-publication path.
Check/build/update/migrate never use the network. A local 0.7.0 binary is newer
than the published 0.6.1 protocol and therefore clears the gate.

## 2. Public identity and bridge

The canonical owner-controlled base is `urn:bedrock:`:

| identity | canonical form |
|---|---|
| context | `urn:bedrock:context/v1` |
| ontology/type | `urn:bedrock:ontology/<Term>` |
| vertex | `urn:bedrock:vertex/<slug>` |
| path pointer | `urn:bedrock:path/<repo-relative>` |
| predicate | `urn:bedrock:<predicate>` |
| namespace graph | `urn:bedrock:graph/<namespace>` |

New authoring (`init`, `adopt`, epoch records, seed source) emits only these
URNs. Every read-side comparison accepts the former
`https://yeetz.dev/bedrock/...` coordinates; named graphs also accept
`https://yeetz.dev/graph/...`. This includes context loading, C3 graph
membership, C4 schemas, C5 path stripping, C8, C9, Decision residency, active
Plan routing, automatic document/state edges, register rendering, and C12
mount validation. Accepted legacy graph names normalize to canonical graph
IRIs in AGENTS.md.

`bedrock update` installs canonical base files but never rewrites authored
vertices. `bedrock migrate-iris` is the only source rewrite and only when
explicitly invoked after a green check.

## 3. Store and mount layout

```text
seed/
  context.yamlld
  schemas/*.json
  substrate-lock.json
situation/
  definition/             complete base source
  architecture/           identity, structure, ExpansionMount registrations
  risk/                   current risks
  plan/                   complete Plan source
  record/                 Decisions and episodic records
  references/             nested cold depth
  <registered-mount>/     opaque expansion-owned exclusion
AGENTS.md                  ONE generated resident artifact
```

The invariant phrase is: **six base namespaces plus registered opaque mounts**.
`contextreg::NAMESPACES` remains exactly six. A mount is not a namespace and
owns no Bedrock named graph. An unregistered direct child of `situation/`
fails C1.

Exactly one `AGENTS.md` exists, at repository root. No mount may contain one.
There is no Bedrock-owned `situation/graph.trig` or per-Plan TriG artifact.
Mount-owned per-run graph files remain expansion business under the mount.

## 4. Vertex anatomy, residency, and rules

Every base YAML-LD source is parsed, expanded, schema-validated, and
edge-validated. A vertex is one file:

- face: structured identity/routing fields;
- optional `body: |`: unbounded node-local prose, always stripped before
  expansion and cold behind the automatic `document` edge.

AGENTS.md is the current resident working set, not the archive:

| source | residency |
|---|---|
| definition, architecture, current risk | resident face |
| ExpansionMount registration | resident architecture face |
| Plan active | routing allowlist + synthesized state + document edge |
| Plan draft/done/abandoned | cold |
| Decision | resident; supersession chain remains walkable |
| EpochRecord, DeployRecord, ReflectVerdict | cold |
| references and every body | cold |

`bedrock check` enforces:

### C1 — placement and generated-artifact confinement

Registration discovery runs before root judgment. Accept the six base
namespaces and roots claimed by ExpansionMount registrations. Reject every
unregistered direct child, legacy Bedrock graph artifact, malformed base
placement, nested base directory, `AGENTS.md` outside root, and root AGENTS.md
byte drift. Registered roots are excluded from the global tree walk and
traversed only by C12.

### C2 — YAML syntax

YAML 1.2 parses under `serde_norway`; anchors, aliases, and merge keys are
forbidden at token level. Comments are projection-irrelevant.

### C3 — JSON-LD profile and graph membership

Exactly one inline context or locally served canonical/legacy Bedrock context;
all remote loading refused; absolute `@id`/`@type`; no blank nodes. Base source
may graph only into its own namespace graph. Canonical and legacy graph names
are read-compatible and emitted canonically.

Mount source never enters C3. C12 inspects only Bedrock context/base-type
claims inside structured mount source.

### C4 — closed schemas

Five Bedrock-owned schemas validate base source. Every schema preserves face,
body, Decision, Plan lifecycle, and dual-coordinate behavior. Architecture
adds only generic ExpansionMount fields. Record permits
`ReflectVerdict.subject` to be a vertex pointer or path pointer; C5 provides
repository containment knowledge.

### C5 — complete-source edge resolution

Every named-node edge target resolves to a source vertex or existing path
under either Bedrock path prefix. A ReflectVerdict path subject additionally
must exist and be lexically/canonically contained by one registered real mount
root. Symlink escape is not containment.

### C6 — determinism

Same source produces byte-identical AGENTS.md. The fixed prefix table is:

```text
rdf      http://www.w3.org/1999/02/22-rdf-syntax-ns#
xsd      http://www.w3.org/2001/XMLSchema#
graph    urn:bedrock:graph/
bedrock  urn:bedrock:
```

The four IRI lengths are pairwise distinct, avoiding the pinned serializer's
unstable equal-length tie break.

### C7 — emitted-artifact parse-back

The complete generated AGENTS.md reparses as TriG to the exact resident quad
multiset. The harness-visible file is the verified RDF artifact.

### C8 — witness gate

A Plan or ReflectVerdict with `disposition.state: done` has at least one HTTPS
CI-run witness. Both coordinate families trigger the same gate.

### C9 — base-type intersection

The conceptual base set has twelve terms:

```text
Invariant Breadcrumb Term Identity SituationStructure Risk Plan
EpochRecord DeployRecord ReflectVerdict Decision ExpansionMount
```

Canonical and legacy spellings are read-compatible. Repo/expansion archetypes
may ride alongside one base type and never stand alone.

### C10 — Bedrock-owned digest skew

The current binary owns exactly:

- `seed/schemas/{definition,architecture,risk,plan,record}.json`;
- `seed/context.yamlld`;
- `seed/substrate-lock.json`;
- `situation/references/bedrock-operating.md`.

Update refreshes this set and may install a missing workflow. It never changes
a present workflow, authored vertex, registration, or mount byte.

### C11 — resident projection closure

A resident vertex may not point by vertex IRI at cold source. Use a path
pointer for history or promote the target into resident knowledge. Generated
mount linkage uses path resources, not cold vertex targets.

### C12 — Mount Contract v1 boundary

C12 is the whole of mount validation:

1. supported registration version, generic shape, unique name/path;
2. real non-symlink root directly under `situation/`, no duplicates/overlap,
   no `AGENTS.md` anywhere inside;
3. `.yamlld`/`.jsonld` mount source inspected only for canonical or legacy
   Bedrock context/base-type claims;
4. registered RDF parsed only to reject every Bedrock-owned subject,
   predicate, object, or graph IRI;
5. init/pin, manifest, and every manifest-listed graph are normalized,
   lexically/canonically contained regular files with matching lowercase
   SHA-256.

Interior symlinks are never followed. Registered paths traversing one must
still canonicalize inside the root. All other mount content is opaque.
Registered RDF syntax is `.trig`, `.nq`, `.ttl`, or `.nt`; unknown syntax
fails closed rather than being guessed.

Hard failures remain `RULE path:line message`. Advisory projection reporting
remains non-failing: exact AGENTS bytes, source/resident counts, Plan states,
record residency, and resident faces over the 4096-character soft budget.

## 5. Compile and one-artifact mount linkage

```text
base YAML-LD
  -> syntax/profile/schema validation
  -> body strip + JSON-LD expansion into full source quads
  -> automatic canonical document edge
  -> full-source C5
  -> resident projection (including ExpansionMount registrations)
  -> append canonical Bedrock mount pointer linkage
  -> C11 resident closure
  -> deterministic sort + TriG serialization
  -> AGENTS.md comment preamble + resident TriG
  -> C7 parse-back on AGENTS.md itself
```

### Contract v1 implementation note — single-artifact adaptation

Mount Contract v1 predates Bedrock's 0.4 single-artifact decision and says
“build emits registration triples plus references/produces edges.” In 0.7,
those triples and edges physically land in the resident architecture graph
inside root `AGENTS.md`, Bedrock's only generated artifact. Specifically:

```text
registration --references------> manifest path
mount path   --produces---------> manifest path
manifest path --artifact-digest-> exact manifest SHA-256 literal
```

The resident TriG body digest therefore pins the registration and linkage.
The deterministic `Mounted expansions` section is a legal-TriG comment block
in the AGENTS.md preamble, one line per registration: name, decoded path,
checker identity. The same registrations/linkage also exist as RDF in the
body. Zero mount graph quads are included. Mount-owned per-run `graph.trig`
files remain inside the mount and are validated only through C12. This is the
faithful v1 adaptation; no separate Bedrock graph artifact is reintroduced.

## 6. ExpansionMount registration and manifest

One consumer-authored flat vertex:

```text
situation/architecture/mount-<slug>.yamlld
```

Base type: `urn:bedrock:ontology/ExpansionMount`; an expansion-owned archetype
may ride alongside. Required fields:

| field | shape |
|---|---|
| `mount_contract_version` | integer; 0.7 supports `1` |
| `mount_name` | unique `[a-z0-9][a-z0-9-]*` |
| `mount_path` | Bedrock path IRI naming a direct `situation/` child |
| `checker_identity` | nonempty single-line literal; never executed |
| `checker_arguments` | ordered string vector serialized as RDF JSON; never executed |
| `init_path` | contained Bedrock path IRI |
| `init_sha256` | SHA-256 of init/pin bytes |
| `graph_manifest_path` | one stable contained Bedrock path IRI |
| `graph_manifest_sha256` | SHA-256 of manifest bytes |

Unsupported versions refuse with an explicit registration-migration message;
Bedrock never rewrites registrations.

Manifest shape:

```yaml
artifacts:
  - path: situation/example-expansion/runs/run-1/graph.trig
    sha256: "<64 lowercase hex digits>"
```

Paths are normalized repo-relative strings, strictly sorted and unique.
`artifacts: []` is the valid empty adoption.

## 7. Locks, workflow, and ownership

`seed/substrate-lock.json` carries:

```json
{
  "contract": "bedrock-expansion-mount/v1",
  "checker": { "package": "yeetz-bedrock", "ref": "0.7.0" },
  "supported_mount_contract_versions": [1]
}
```

The installed workflow is a caller pinned to the v0.8.0 central reusable gate.
That gate reads this lock, runs the resident-projection report and AGENTS.md
drift check, and keeps Cargo home, target, and install root under runner temp.
It runs for every current PR head. Its first step rejects fork PRs before
checkout or execution; GitHub Actions cannot fail a step before assigning its
runner.

Update never rewrites a present caller. New immutable gate tags are adopted by
reviewed stub-only propagation PRs. Mount consumers keep this Bedrock caller
and run expansion-owned checks independently; they do not combine or replace
the central Bedrock job.

The expansion checker remains independently pinned by its own pack init.
Neither checker ref derives from the other. Bedrock never executes registration
checker data and learns no expansion concepts.

## 8. Decisions, Plans, and promotion seam

Decision remains resident, write-once, timestamped, and linked through
`supersedes`; it has no disposition/witnesses. ExpansionMount does not change
Decision anatomy.

Only active Plan routing faces are resident. Complete draft promotion proposals
remain cold until separately activated. Mount campaign proposals use existing
base shapes:

- retained risk: complete Risk with source/path and exact manifest digest;
- accepted action: complete draft Plan with intent, criteria, consumes mount
  path, ordered tasks, proposal-only residual;
- close: ReflectVerdict whose subject is an existing contained mount path,
  with criteria, completed two-checker CI URL, findings/residual as applicable.

ReflectVerdict remains episodic/cold; its source still fully validates.
Promotion never rewrites prior mount source/evidence.

## 9. Authoring and release sequence

Epoch records carry commit, local binary version, mode, offline stamp, and the
cut statement under canonical URNs.

Graph-changing order:

1. write source;
2. expansion check/build when mounted;
3. `bedrock check` and `bedrock build`;
4. both generated-output no-diff gates;
5. commit source plus AGENTS.md; open a PR; an agent merges a green gate, a
   human merges at a fork in the road.

The installed Bedrock workflow is a caller pinned to the immutable v0.8.0
central reusable gate. It runs on every current PR head, rejects fork PRs
before checkout or execution, and is advanced by reviewed tag-bump propagation
PRs; mounted repositories keep independent expansion CI.

0.7.0 carries the still-supported dual-read identity bridge and generic Mount
Contract v1 on top of 0.6.1's version gate, face/body anatomy, Decision type,
resident working set, and one-artifact model.
