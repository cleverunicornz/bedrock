# bedrock

`bedrock` installs a complete machine-readable YAML-LD situation store,
validates every source, and compiles its current resident working set into
the one deterministic `AGENTS.md` every agent receives.

Seed a new repo, epoch-change an existing one, validate source and projection
in CI, and regenerate the injected graph — from a single binary.

## Install

```sh
cargo install yeetz-bedrock
```

The crate is `yeetz-bedrock` (the name `bedrock` is taken); the binary it
installs is `bedrock`.

`init`/`adopt` query crates.io before writing: stale or unverifiable binaries
refuse and name the locked install command. `--offline` is the deliberate
bypass and is stamped into the epoch record. `check`/`build`/`update` remain
network-free.

## Commands

- `bedrock init`  — seed a new repo: install the `situation/` skeleton and
  seed floor, generate root `AGENTS.md`, print the init instruction set.
- `bedrock adopt` — epoch-change an existing repo: same install, plus an
  epoch record declaring the cut line.
- `bedrock check` — validate complete source plus the resident projection;
  print exact resident/cold counts and soft face budgets; the CI entrypoint.
- `bedrock build` — regenerate resident TriG in AGENTS.md; refuses while
  `check` fails.
- `bedrock update` — refresh installed base files from this binary's embedded
  copies, then check and build; never touches repo-authored vertices.
- `bedrock help`  — the contract, short.

## The model

Six situation namespaces, five work verbs, one `AGENTS.md` per repository.

### Canonical store and resident graph

Every repo carries `situation/`, the complete validated store:

- `definition/`, `architecture/`, current `risk/` — resident knowledge.
- `plan/` — complete Plans. Only `active` contributes a compact routing face;
  draft/done/abandoned and all execution payload stay cold.
- `record/` — Decisions resident; epoch/deploy/reflect records cold.
- `references/` — cold shared depth; may nest and hold non-YAML-LD files.

Cold means source-only, not discarded: the resident SituationStructure node
discloses each path. Agents pull history or depth only when the task needs it.
Every source stays schema- and edge-validated.

AGENTS.md is the deterministic resident TriG projection, not the archive.
Same source and states, same bytes.

### Work verbs

Work runs in short-lived branches, one verb per state:

- `think/...`    explore and decide.
- `plan/...`     write the plan as a graph.
- `execute/...`  do the work.
- `reflect/...`  review what was done.
- `deploy/...`   place the result.

### One AGENTS.md

`bedrock build` regenerates root AGENTS.md from the resident working set.
Hand-editing it is drift; `check` fails instead of tolerating it.

### Brittleness with intent

`bedrock check` fails loudly and exactly: one violation per line, `RULE
path:line message`. The failure output is the fix instruction. The tool is
not forgiving by design — it is written to fail hard so agents fix the cause,
not the symptom.
