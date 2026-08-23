# bedrock adopt

Epoch change. `bedrock adopt` installed `situation/` (skeleton + seed
floor), wrote an epoch record, generated root `AGENTS.md`, and listed every
nested `AGENTS.md`. After your next commit, bedrock governs this repo.

## 1. The epoch record

`situation/record/epoch-<utc-date>-<short-sha>.yamlld` fixes the cut line:
after it, bedrock governs. Prior history — including the old AGENTS.md — is
reference, never law. Root `AGENTS.md` is generated law: floor invariants,
repo-local, breadcrumbs, where things live. Never hand-edit it. The base
protocol lives at `situation/references/bedrock-operating.md` — the CHAIN
(promise, oracle, witness, residual), the ontology, every rule. Read it
before converting donor content.

## 2. Convert donor content, forward-only

Old AGENTS.md is donor material. Convert by hand, forward-only, only what
this repo still relies on — never by sweep.
1. Survivors → `definition/` vertices tagged `layer: situated`, with a
   `statement`.
2. Structure and intent → `architecture/`.
3. Live warnings → `risk/`.
4. Edge targets resolve to vertices or existing repo paths; check enforces.

## 3. Delete nested AGENTS.md files

Delete every `AGENTS.md` outside the root that was listed, once its law is
in vertices. Only root `AGENTS.md` remains — the one-AGENTS.md law.

## 4. The six namespaces

Vertices are flat files under `situation/` — six namespaces, no new
top-level dirs; only `references/` may nest and hold markdown. `definition/`
invariants; `architecture/` intent (its `role: identity` vertex names the
repo); `risk/` warnings, deleted when retired; `plan/` graphs; `record/`
epochs, deployments, verdicts, decisions; `references/` depth docs.

## 5. The five work verbs

1. `think/`    explore, decide.
2. `plan/`     write the plan as a graph.
3. `execute/`  do the work.
4. `reflect/`  review after the fact.
5. `deploy/`   place the result where it is recorded.

## 6. Add a vertex and build

1. Write `situation/<ns>/<local-name>.yamlld` — `@id`, `@type`, `label` or
   a one-line description, and typed edges; copy an existing vertex.
2. Run `bedrock build`: re-checks, compiles YAML-LD to deterministic TriG
   (`situation/graph.trig`), regenerates `AGENTS.md`. Failure prints `RULE
   path:line message`; fix the cause — build refuses while red.

## 7. Commit and PR

Commit the epoch record, converted vertices, deleted nested AGENTS.md,
`graph.trig`, and regenerated `AGENTS.md`; open a PR. A human merges — you
do not merge your own.

Summary: this commit is the cut; prior law is reference, not binding.
Convert survivors, delete nested AGENTS.md files, build, PR for a human merge.
