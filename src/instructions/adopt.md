# bedrock adopt

Epoch change. `bedrock adopt` installed `situation/` (skeleton + seed
floor), wrote an epoch record, generated root `AGENTS.md`, and listed every
nested `AGENTS.md`. After your next commit, bedrock governs this repo.

## 1. The epoch record

`situation/record/epoch-<utc-date>-<short-sha>.yamlld` fixes the cut line;
it stays cold source. Prior history — including old AGENTS.md — is reference,
never law. Root AGENTS.md is the generated resident working set; complete
source/history stays in situation/. Never hand-edit the artifact. Read the
base protocol at `situation/references/bedrock-operating.md` before
converting donor content.

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

## 4. Six base namespaces plus registered opaque mounts

Base vertices are flat under six namespaces. Definition, architecture,
current risk, Decisions, ExpansionMount registrations, and active Plan
routing faces are resident. Other Plans, episodic records, bodies, and
references are cold but validated. Only references may nest. No other direct
child is legal unless a complete ExpansionMount registration claims it; a
mount remains opaque and is not a seventh namespace.

## 5. The five work verbs

1. `think/`    explore, decide.
2. `plan/`     write the plan as a graph.
3. `execute/`  do the work.
4. `reflect/`  review after the fact.
5. `deploy/`   place the result where it is recorded.

## 6. Add a vertex and build

1. Write `situation/<ns>/<local-name>.yamlld`: compact routing face on top,
   node-local depth in `body: |`.
2. Every Plan declares draft|active|done|abandoned. Only active projects a
   routing face; execution payload stays cold.
3. Run `bedrock build`: validate complete source, regenerate the resident
   TriG in root AGENTS.md, and print resident/cold counts plus advisory SOFT
   budgets. Fix each hard `RULE path:line message`; build refuses while red.

## 7. Commit and PR

Commit the epoch record, converted vertices, deleted nested AGENTS.md, and
regenerated root AGENTS.md (the resident projection); open a PR. A human
merges — you do not merge your own.

Summary: this commit is the cut; prior law is reference, not binding.
Convert survivors, delete nested AGENTS.md files, build, PR for a human merge.
