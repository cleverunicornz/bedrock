# bedrock init

Your new repo is seeded. `bedrock init` installed the skeleton, generated
root `AGENTS.md`, ran `check`, and it passed. Go in order.

## 1. What was installed

- `situation/` skeleton — six empty namespaces, ready for vertices.
- Seed floor — invariants and terms as vertices tagged `layer: floor`.
- A workflow hook — `bedrock check` + `bedrock build` on `situation/` changes.
- The operating reference — `situation/references/bedrock-operating.md`, the
  base protocol: the CHAIN (promise, oracle, witness, residual), the ontology,
  every rule. Read it before writing your first vertex.
- Root `AGENTS.md`, compiled from the graph. Read it first. It is law; never
  hand-edit it.

## 2. The six namespaces

Vertices live flat in their namespace: `situation/<ns>/<local-name>.yamlld`.

- `definition/` — invariants and terms: law agents must not break.
- `architecture/` — what this repo is and intends; the vertex marked
  `role: identity` names the repo in AGENTS.md.
- `risk/` — present-tense warnings; delete the vertex when the risk retires.
- `plan/` — execution graphs, the plans of record for each branch.
- `record/` — epoch records, deploy placements, reflect verdicts.
- `references/` — depth documents; the only namespace that may nest and hold
  non-YAML-LD files (markdown).

No new top-level directories under `situation/`.

## 3. The five work verbs

Prefix every work branch, one verb per state:

1. `think/...`    explore, decide.
2. `plan/...`     write the plan as a graph.
3. `execute/...`  do the work.
4. `reflect/...`  review it after the fact.
5. `deploy/...`   place the result where it is recorded.

## 4. One AGENTS.md

Exactly one `AGENTS.md` per repository, at the root, generated. Never edit
it by hand; delete any other copy. This law is non-negotiable.

## 5. Add a vertex

1. Write `situation/<ns>/<local-name>.yamlld`: `@id`, `@type`, `label` or a
   one-line description, and typed edges. Copy an existing vertex's shape;
   add a `statement` to invariants.
2. `@id` bases: floor vertices (shipped by the seed, `layer: floor`) live
   under the bedrock base `https://yeetz.dev/bedrock/vertex/<local>`; your
   own situated vertices use your repo base `https://yeetz.dev/<repo>/vertex/
   <local>`. Same slug pattern under both; the schemas enforce that floor is
   bedrock-namespaced.
3. Run `bedrock build`. It re-checks, compiles YAML-LD to deterministic TriG
   (`situation/graph.trig`), and regenerates `AGENTS.md`.
4. If check fails: one violation per line, `RULE path:line message`. Fix the
   cause — `build` refuses to proceed while check is red.

## 6. Commit and PR

Commit the vertices, `situation/graph.trig`, and regenerated `AGENTS.md`;
open a pull request. A human merges; you do not merge your own work.

Summary: skeleton, floor, workflow hook, and root AGENTS.md are live. Add
law as vertices, run `bedrock build`, then PR for a human merge.
