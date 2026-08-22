# Census 04: Research & Spike Lanes

Basket: 1000-research, 2000-spike-campaign-orchestrator, 2020-spike-worker-execution, 2030-spike-confirmer, 2040-spike-promotion-packager.
Verified: runs/1000/ (18), runs/2000/ (46), spikes/ (2 campaigns), tools/runs-validator, architecture/development-lifecycle/schemas/ (10), JSONL ledger/events.
All skill paths under `.agents/skills/<name>/SKILL.md`, abbreviated `SKILL.md`; line refs from those files.

## Rows

- skill: 1000-research
  binds_to:
    - {ref: runs/1000/, status: alive, evidence: runs/1000/ (18 entries; e.g. runs/1000/1785166581381-d3c4-work-protocol-epoch/findings.md:1)}
    - {ref: $search, $direct-repo-scout, $external-repo-bank, $external-source-research, $moon-task-graph, $development-environment, $adopted-protocols, $git-policy, $codex-goal-use, status: alive, evidence: SKILL.md:35,40-47}
    - {ref: architecture/protocols/AGENTS.md, status: alive, evidence: SKILL.md:44}
    - {ref: runs/2000/, status: alive, evidence: SKILL.md:75-82 (promotion routing)}
  knowledge_delta: mixed — "read-only investigation, cite sources, no mutation" is model-native; the branch-per-research + verdict-receipt + promotion-routing lane is repo-taught
  relation: PORTABLE-CORE
  loop_verb: think
  kernel: Directed read-only questions, no rig, no mutation: investigate, commit findings + citations + a verdict. If answering requires building, running, or mutating, that is a spike, not research. Spitballing in conversation is not research. A verdict can route straight to promise (6000) when it closes a direction, or to 2000 when it exposes an empirical unknown.
  breadcrumb_gate: Read-only question? investigate and record a verdict with citations; the moment an answer needs a build/run, stop and route to 2000 (SKILL.md:32,75-82).
  overreach: How-to-read-and-search. The whole investigation corpus ($search, scout, repo-bank, external-research) is a model already knows how to do; the skill mainly institutionalizes it in a branch.
  value_notes: buys a recorded, cited verdict per research question and an explicit read-only/no-rig boundary against escalation into spike ceremony (SKILL.md:10-15 reasons the lane exists). costs single-model/effort pinning (gpt-5.6-terra@max, SKILL.md:19-21) and a branch+goal+record per question — ceremony for a natively-trivial act.

- skill: 2000-spike-campaign-orchestrator
  binds_to:
    - {ref: architecture/development-lifecycle/AGENTS.md, status: alive, evidence: SKILL.md:7-8,135 (binding law)}
    - {ref: architecture/development-lifecycle/schemas/campaign-frontmatter.schema.yaml, claim.schema.yaml, run-event.schema.yaml, spike-frontmatter.schema.yaml, status: alive, evidence: SKILL.md:67-72; dir listing schemas/ (10 files)}
    - {ref: tools/runs-validator (yeetz-runs-validator), status: alive, evidence: tools/runs-validator/Cargo.toml:1 (name), Cargo.toml:181 (workspace member), tools/runs-validator/src/main.rs:4 (init/fmt/check/resolve-oracles)}
    - {ref: runs/2000/, status: alive, evidence: runs/2000/ (46 campaigns, e.g. codex-run-s3-v2-260722-7e16)}
    - {ref: spikes/, status: alive, evidence: spikes/ (2 campaigns, e.g. spikes/codex-run-s3-v2-260722-7e16/{01..04}-*); SKILL.md:144 (quarantine law)}
    - {ref: $oracle, $collapser, $witness, $search, $moon-task-graph, $development-environment, $tooling-object-storage, $adopted-protocols, $agent-run, $git-policy, status: alive, evidence: SKILL.md:12-16,30-44,80-93,121}
  knowledge_delta: mixed — hypothesis-before-run + criteria-before-results + timebox is portable; the campaign charter/ledger/branch-indexed-record machine is repo-taught
  relation: PORTABLE-CORE
  loop_verb: think
  kernel: Empirical unknown, not settled direction: state the question and load-bearing unknowns as a charter before anything runs; freeze criteria before results; give every spike a timebox and the campaign a decision point that resolves it (promote/discard/park) even with a tail. Refutation is a successful spike; an unchanged question/criteria/method is noise and gets no second run. A spike defends no promises — it earns no completion gate, no adversarial panel.
  breadcrumb_gate: Can the question only be answered by running a rig? author the hypothesis + criteria + method first and quarantine the rig; every element of the scientific tuple must differ to justify a follow-up (SKILL.md:138-142).
  overreach: What a hypothesis is, why you freeze criteria, why refuted-≠-failed.
  value_notes: buys genuinely held hypothesis-first discipline and hard quarantine; costs a heavy control plane — charter + independent ledger + orchestrator-owned verdict events + branch-per-campaign + validator checks (SKILL.md:65-135), much of it bookkeeping (see Ceremony Measurement).

- skill: 2020-spike-worker-execution
  binds_to:
    - {ref: architecture/development-lifecycle/AGENTS.md, status: alive, evidence: SKILL.md:5-6}
    - {ref: spikes/ defense, status: alive, evidence: SKILL.md:40-43; spikes/*/00*/ (own workspace per rig)}
    - {ref: tools/runs-validator, status: alive, evidence: SKILL.md:10 (docket runs under validator tree)}
    - {ref: $agent-run, $git-policy, $moon-task-graph, $development-environment, $tooling-object-storage, $oracle, $collapser, status: alive, evidence: SKILL.md:7,19-37}
  knowledge_delta: mixed — smallest-rig, run-control-then-decisive, keep raw evidence, mixed-stays-mixed is portable; the quarantine [[workspace]] / traced-docket / frontmatter-status machine is repo-taught
  relation: PORTABLE-CORE
  loop_verb: execute
  kernel: Build the smallest rig that can produce ONE decisive observable; run the control and decisive cases; keep raw decisive output and exact commands; classify strictly against frozen criteria (DEMONSTRATED = existence proof only, REFUTED = decisive miss, INCONCLUSIVE = did not decide + why); expired timebox = INCONCLUSIVE, not a license to keep running.
  breadcrumb_gate: The rig runs under quarantine with frozen criteria; record RESULT strictly against those criteria and classification, or BLOCKED if the docket is incomplete (SKILL.md:31,73,90-91).
  overreach: How to build a test rig and read its output.
  value_notes: buys honest existence-vs-refutation classification and raw-evidence retention; costs the quarantined-workspace + traced-command ceremony that keeps rigs off product paths — the useful part is the classification discipline, the wiring is repo-shaped.

- skill: 2030-spike-confirmer
  binds_to:
    - {ref: tools/runs-validator `check`, status: alive, evidence: SKILL.md:23 (runs `cargo run -p yeetz-runs-validator -- check`)}
    - {ref: immutable charter/result commits + evidence paths, status: alive, evidence: SKILL.md:19,32-42 (binding identity)}
    - {ref: $agent-run, status: alive, evidence: SKILL.md:12}
  knowledge_delta: repo-taught — "verify the record's integrity with an immutable commit-order + evidence-exercises-criteria check, don't re-derive merit" is a mechanical extension of the predeclare law
  relation: DEAD
  loop_verb: reflect-code
  kernel: One light integrity pass over a decided spike: does the record parse, did criteria precede results by immutable commit order, does the evidence actually exercise the pre-declared criteria, and does the verdict match the evidence direction. Never re-run the rig, re-derive the answer, or weigh merit (SKILL.md:43).
  breadcrumb_gate: Confirm the record's integrity and criteria-before-result commit order, never the finding's merit (SKILL.md:43).
  overreach: Everything a reviewer already does. Explicitly anti-adversarial (SKILL.md:3): it refuses to attack the finding.
  value_notes: demonstrably adds ONE real invariant — criteria-frozen-before-result enforced as git commit ordering (SKILL.md:32-33) — plus schema/evidence-plausibility checks the validator already partly owns (SKILL.md:23 delegates structure checks to the tool). Net value thin; mostly a per-spike ceremony pass (4 confirmer events for the 4-spike sample campaign, runs/2000/codex-run-s3-v2-260722-7e16/events/). Could be a script for the commit-order check + a one-line human glance for evidence-plausibility.

- skill: 2040-spike-promotion-packager
  binds_to:
    - {ref: runs/2000/<cid>/ledger/claims.jsonl + verdict events + decided records, status: alive, evidence: SKILL.md:11-13,19-25}
    - {ref: oracle_ref → spikes/ rig at pinned SHA, status: alive, evidence: SKILL.md:24; ledger sample runs/2000/codex-run-s3-v2-260722-7e16/ledger/claims.jsonl:1}
    - {ref: architecture/ (promise grounds), status: alive, evidence: SKILL.md:57-61 (Promise Grounding Disposition)}
    - {ref: runs/2000/<cid>/promotion/package.md, status: alive, evidence: SKILL.md:15; sample package.md:1}
    - {ref: $collapse-graph, $oracle, $witness, $agent-run, status: alive, evidence: SKILL.md:25,28}
  knowledge_delta: mixed — "promote only what the records show, name missing promise sources" is portable; the oracle-ref / tolerance-tier / promise-grounding / gauge-spec mapping into 3000/6000 is repo-taught
  relation: SITUATED
  loop_verb: reflect-situated
  kernel: Assemble the promotion package from recorded authority only (answer-ledger + verdict events + decided records), never from recollection; name resolved unknowns and explicitly-accepted residue; classify each obligation as grounded (name the pinned architecture source) or promise-authoring-required (name the 6000 predecessor that must land); human-gated — the packager prepares, the human promotes.
  breadcrumb_gate: Fold only recorded verdicts into the promotion package and state each obligation's promise ground (grounded vs promise-authoring-required), human-gated (SKILL.md:11,57-61,76).
  overreach: The routing judgment (does a landed promise source exist) is model-native; the oracle-ref/gauge-spec vocabulary is not.
  value_notes: buys the distillation boundary and forces a grounded-vs-missing-promise decision before 3000; costs a large fixed evidence-binding list (confirmation-pass id, passback ref, digest, immutable commits, controls, residual) that mostly re-encodes what the orchestrator's verdict events already hold (SKILL.md:19,24-28,33-41).

## Ceremony Measurement

Sample: runs/2000/codex-run-s3-v2-260722-7e16 (4 spikes, incl. one REFUTED: 02-known-authority-boundary).
Measured from curated human text (omitting raw machine evidence):

- Question/answer content (records + promotion + charter prose): 4 records 471 lines + promotion 173 + findings 33 + charter 64 = 741 lines.
- Scaffolding: events JSONL (16 campaign + 9 spike = 25 lines), ledger claims (17 rows), frontmatter blocks (~4×7 + charter ~9 = ~37 lines) ≈ 79 lines ≈ **10% of curated text**.
- Role ceremony: for 4 spikes there are **4 confirmer passes** (events/*-spike-confirmer.jsonl ×4) + **1 packager** + per-spike orchestrator/worker events — the cost concentrates in agent-role churn (an extra agent run per spike for confirmation), which line counts hide (each CONFIRMED is one JSONL line, SKILL.md:2030:44).
- Raw evidence dominates the byte budget: 56,718 lines total, mostly two bit-identical 18,107-line `lab.json` dumps triplicated across run timestamps (04-v2-protocol-gauge/evidence/*/lab.json:1) — evidence, not ceremony, but heavily duplicated.
- Organization: ceremony ≈ 1/10 of curated text; role-pass churn ≈ 1.25 confirmer+packager passes per spike, invisible in line counts.

## Kernels

Consolidated think-discipline kernel (from the five skills):

1. State the hypothesis before anything runs, and freeze criteria before results: RESULT is judged strictly against pre-declared CRITERIA, never against a moved target (1000 SKILL.md:32; 2000 SKILL.md:131,134; 2020 SKILL.md:73).
2. Everything runs under a timebox and the campaign under a decision point; when the point arrives the campaign resolves even if a tail remains (2000 SKILL.md:138). An expired timebox is INCONCLUSIVE with what you have, not a license to keep running (2020 SKILL.md:90-91).
3. A refuted hypothesis is a successful spike — it collapses a possibility; refutation is an earned answer, not a failure (2020 SKILL.md:73; the sample REFUTED record states it plainly: runs/2000/codex-run-s3-v2-260722-7e16/02-known-authority-boundary/record.md:1,99-107).
4. Keep the rig quarantined from product paths and keep the raw evidence that exercised the criteria; a bare verdict without that evidence is inadmissible (2000 SKILL.md:144; 2020 SKILL.md:40-41,73).
5. Wrong-early is expected: the sample campaign retained and corrected two pre-decisive rig attempts before the decisive run (02-known-authority-boundary/record.md:104-107) — iteration is visible in the record, not sanitized. This is implied by (1)+(3) and demonstrated in the records, though no skill states "wrong-early" as an explicit law.
6. Mixed outcomes stay mixed; never round toward a verdict (2020 SKILL.md:70-71).

## Dead & Delete

- 2030-spike-confirmer → DEAD. Its one real invariant (criteria-before-result enforced by commit ordering, SKILL.md:32-33) is either scriptable or a one-line glance; it refuses to add merit judgment (SKILL.md:43), so it buys record hygiene at a per-spike agent-run cost (4 passes for 4 spikes).
- 1000-research's cited reader toolkit ($search, scouts, repo-bank) is not a distinct capability — fold into the portable "read-only research" breadcrumb; only the verdict-record + no-rig-boundary survives.

## Surprises

- The single most load-bearing, repo-external idea in the whole family is not in a "research" skill but in the worker's classification law: DEMONSTRATED = existence proof only, REFUTED = decisive miss, INCONCLUSIVE = didn't decide (2020 SKILL.md:73). That is the anti-overclaim core worth keeping.
- The orchestrator (2000) and packager (2040) each duplicate a routing graph (1000→6000, 1000→2000→6000, 2000→6000→3010, 3000→3020→3030) with heavy evidence-binding ceremony. The routing itself is portable; the binding lists are situated.
- spikes/ is genuinely small (2 campaigns) despite runs/2000/ holding 46 campaigns — most 2000 entries are bookkeeping-light, and rig code never pollutes product paths (own empty [workspace], spikes/apparatus-requalification-260727-38f9/*).
- The confirmer/packager roles are honest about their own limits (no panel, no repair, human-gated promotion) — the bureaucracy is self-aware, which is why most layers are thin ceremony rather than harmful overreach.

---
HEADLINE: Research is a native read-only act; only the no-rig boundary + verdict record survive (1000 = PORTABLE think). Spike discipline — thermometer of the family — is hypothesis-first, criteria-frozen, timeboxed, rig-quarantined, refuted-equals-success (2000/2020 = PORTABLE think/execute). 2030 confirmer is a per-spike ceremony pass with one scriptable invariant → DEAD. 2040 packager = situated distillation+routing, real but repo-bound (SITUATED maybe not a promise). Ceremony ≈ 10% of curated text but ~1 extra agent pass per spike. Refused "wrong-early" is demonstrated, not legally stated.
