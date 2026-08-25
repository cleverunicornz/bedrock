# bedrock init

Your new repo is seeded. `bedrock init` installed the skeleton, generated
root `AGENTS.md`, ran `check`, and it passed. Go in order.

## 1. What was installed

- `situation/` skeleton — six base namespaces, ready for vertices.
- Seed floor — invariants and terms tagged `layer: floor`.
- `seed/substrate-lock.json` plus a lock-consuming dry-run workflow.
- The operating reference — `situation/references/bedrock-operating.md`, the
  base protocol: the CHAIN (promise, oracle, witness, residual), the ontology,
  every rule. Read it before writing your first vertex.
- Root `AGENTS.md`, the resident situation graph. Read it first; never
  hand-edit it. Complete source and cold history stay under `situation/`.

## 2. Six base namespaces plus registered opaque mounts

Vertices live flat in their namespace: `situation/<ns>/<local-name>.yamlld`.

- `definition/` — resident invariants, breadcrumbs, terms.
- `architecture/` — resident current structure and intent.
- `risk/` — resident present-tense warnings; delete when retired.
- `plan/` — complete Plans; only active compact routing faces are resident.
- `record/` — Decisions resident; epoch/deploy/reflect records cold.
- `references/` — cold shared depth; may nest and hold markdown.

No other direct child is legal. An expansion root becomes legal only through
a complete architecture ExpansionMount registration; it is never a seventh
namespace.

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

1. Write `situation/<ns>/<local-name>.yamlld`: compact routing face on top,
   node-local depth in `body: |`.
2. Every Plan declares `disposition.state`: draft|active|done|abandoned.
   Only active projects a routing face; all execution payload stays cold.
3. Run `bedrock build`. It validates the complete situation, emits the
   resident TriG into root AGENTS.md, and reports resident/cold counts plus
   advisory SOFT face budgets.
4. Fix each hard `RULE path:line message`; build refuses while red.

## 6. Commit and PR

Commit source and regenerated AGENTS.md (the resident projection); open a
pull request. An agent merges once the gate is green; a fork in the road — an
architecture choice, a removal, a cascading blast radius — is the human's call.

Summary: skeleton, floor, substrate lock, workflow, and root AGENTS.md are
live. Add law as vertices, run `bedrock build`, then open a PR.
