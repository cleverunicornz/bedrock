# bedrock

`bedrock` is the origin tool: it installs a machine-readable situation graph
into a repository, compiles it into a deterministic triple format, and
generates the one `AGENTS.md` that governs work there.

Seed a new repo, epoch-change an existing one, validate the graph in CI, and
regenerate the file agents actually read — from a single binary.

## Install

```sh
cargo install yeetz-bedrock
```

The crate is `yeetz-bedrock` (the name `bedrock` is taken); the binary it
installs is `bedrock`.

## Commands

- `bedrock init`  — seed a new repo: install the `situation/` skeleton and
  seed floor, generate root `AGENTS.md`, print the init instruction set.
- `bedrock adopt` — epoch-change an existing repo: same install, plus an
  epoch record declaring the cut line.
- `bedrock check` — validate `situation/`; the CI entrypoint.
- `bedrock build` — compile YAML-LD to TriG and regenerate `AGENTS.md`;
  refuses to run while `check` fails.
- `bedrock update` — refresh the installed base files (schemas, context,
  operating reference) from this binary's embedded copies, then check and
  build; additive-safe, never touches repo-authored vertices.
- `bedrock help`  — the contract, short.

## The model

Six situation namespaces, five work verbs, one `AGENTS.md` per repository.

### Situation graph

Every repo carries `situation/` with exactly six namespace directories, each
holding flat `*.yamlld` vertices:

- `definition/` — invariants and terms: the seed floor plus repo-local law.
- `architecture/` — what this repo is and intends.
- `risk/` — present-tense warnings; deleted when the risk retires.
- `plan/` — execution graphs.
- `record/` — epoch records, deploy placements, reflect verdicts.
- `references/` — depth documents; the only namespace that may nest and hold
  non-YAML-LD files.

Vertices are YAML-LD mapped 1:1 to JSON-LD 1.1. The compiler emits
byte-stable TriG: same input, same bytes.

### Work verbs

Work runs in short-lived branches, one verb per state:

- `think/...`    explore and decide.
- `plan/...`     write the plan as a graph.
- `execute/...`  do the work.
- `reflect/...`  review what was done.
- `deploy/...`   place the result.

### One AGENTS.md

`bedrock build` regenerates the root `AGENTS.md` from the compiled graph:
invariants first, then breadcrumbs, then where things live. Hand-editing it
is drift; `check` fails on it rather than tolerating it.

### Brittleness with intent

`bedrock check` fails loudly and exactly: one violation per line, `RULE
path:line message`. The failure output is the fix instruction. The tool is
not forgiving by design — it is written to fail hard so agents fix the cause,
not the symptom.
