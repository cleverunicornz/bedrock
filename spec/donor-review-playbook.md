# Census 06: 7000 Review Family

Survey of the 15-skill adversarial-review family against real outputs:
`workflowenginereview.md` (7-parallel-reviewer teardown, verified C1/C2/C3),
`YEETZ_DB_REVIEW_FINDINGS.md` (275k-line end-to-end read, `[SRC-ME]/[SRC-MULTI]/[SRC-1]` tiers),
`code-reviews/` archive (yeetz-db-stage2-readiness / demand-census / product-classification / unresolved-closure),
`origin/*review*` branch namespace, `reviews/*` frozen namespace.

Core loop is provably small. `workflowenginereview.md:4` states the entire
method: *"7 parallel deep-read reviewers, findings de-duplicated and ranked;
the highest-impact items re-verified against source by direct read."* That one
line — with zero charter ceremony, zero G###/H###/packet machinery, zero
validator roles, zero code-review-archive — produced all three
line-cited/verified criticals and the PR decomposition. The 15 skills exist to
operate a whole-assurance-campaign bureaucracy; the value they add beyond that
one line is concentrated in a handful of stages.

## Rows

- skill: 7000-code-review-orchestrator
  evidence_of_value: Entire `code-reviews/` archives are its run-state
    (`run-state.json`, `dispatch/*.json`, stage routing) — runs would not
    exist without it
  relation: SITUATED
  disposition: KEEP-IN-PLAYBOOK
  kernel: Run order, evidence ledger, mechanical sweep (all ids terminal, every
    reproduced finding independently verified), human gate that never
    self-promotes. Model pinning, `runs/7000` paths, `$agent-run`, run-state
    schema are repo-specific.

- skill: 7005-review-charter-guarantees
  evidence_of_value: `00-charter.md` + `05-guarantees.md` in every archive; the
    G-### certificate that makes a report falsifiable (G-001..G-020 in stage2)
  relation: PORTABLE-CORE
  disposition: FOLD(playbook stage "Framing & guarantees")
  kernel: Pin the target + human question into explicit falsifiable guarantees,
    each with a counterexample observable and completion rule. Inventory
    rule + exclusions explicit before proof. (Drop risk-registry/3000/4000-collapser
    coupling — situational.)

- skill: 7010-review-recon
  evidence_of_value: `10-recon/survey.md` + `scout-*.md` in every archive;
    YEETZ_DB "...every source file read end-to-end by a dedicated reviewer"
    (:1); workflowenginereview "7 parallel deep-read reviewers"
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK
  kernel: Map the surface, partition by disjoint surface/lens, fan out bounded
    deep-read reviewers, collect candidates. This is where findings are born.

- skill: 7011-review-recon-scout
  evidence_of_value: `10-recon/scout-*.md` (5 in stage2, 3 in demand-census)
  relation: PORTABLE-CORE
  disposition: FOLD(7010 recon fan-out)
  kernel: One reviewer, one disjoint assignment, candidate only when a specific
    input/state/race yields a specific forbidden outcome, strong out-of-scope
    sightings go to referrals.

- skill: 7020-review-triage
  evidence_of_value: `15-triage.md` (dedupe "only identical mechanisms",
    hypothesis registry H-###, ranked packets); workflowenginereview "findings
    de-duplicated and ranked" is this method distilled
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK
  kernel: Merge only identical counterexample mechanisms, preserve lineage,
    rank by impact × likelihood × proof value (not severity), never silently
    drop work (mark budget-cut).

- skill: 7025-review-test-integrity
  evidence_of_value: `18-test-integrity.md` in stage2 only (T-### obligations,
    "does-not-protect" verdicts); absent from both smaller archives — track is
    conditional
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK (optional stage)
  kernel: Passing tests are claims, not proof; design the smallest break that
    should make each test fail; only independently validated sensitivity
    yields does/does-not-protect. Run only when a delivery claim rests on
    tests.

- skill: 7030-review-integrity-plan
  evidence_of_value: `20-integrity/<packet>-plan.md`, P-## source-adjudication
    packets (demand-census ran 8 packets on 0 executors)
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK (simplified)
  kernel: Attack on paper before building any rig — assume each claim wrong,
    walk path, construct counterexample, kill unsupported claims with specific
    evidence; then write a spec a leaf can run without redesign. (Predeclared
    immutability mapping is situational ceremony, drop.)

- skill: 7035-review-integrity-execute
  evidence_of_value: `20-integrity/` proof reports + `repros/` (7.3 MB of rigs);
    `dispatch/integrity-P-0*.json`; every "independently reproduced hard stop"
    in the report traces here
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK
  kernel: Build the exact specified rig, run controls + decisive case, preserve
    replayable evidence (rig, commands, README, run count), keep mutations
    out of reviewed source. Executor observation is not the verdict.

- skill: 7036-review-proof-validation
  evidence_of_value: `dispatch/integrity-V-0*.json`; independent-validator
    lineage lines in report (`Proof Lineage`); caught W-01 `INVALID_PROOF`
    (absent M11/M12 artifacts, unsafe provider arm — `feedback/integrity-W-01-2a6f.md`)
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK
  kernel: A disjoint actor replays the control + decisive observable against the
    untouched target without author context; this alone emits the
    reproduces/refutes/invalid disposition. Highest-value independence — this
    is what the human rates highest.

- skill: 7040-review-gapfill
  evidence_of_value: `30-gapfill/gapfill.md` (exactly one bounded wave-2 plan
    W-01) — closed genuine coverage gaps (G-018 UNRESOLVED from INVALID_PROOF)
  relation: TOOLING
  disposition: FOLD(report-stage coverage audit)
  kernel: After first pass, audit guarantee × possibility × coverage, then run
    at most one bounded second wave; name over-budget work as residual risk,
    never silently expand.

- skill: 7050-review-rootcause-trace
  evidence_of_value: `40-rootcause-trace/D-01..D-12-trace.md` — the report's
    "12 independently supported root-cause families" derive here; D-01 trace
    shows H-001/H-002 → one defect, blast radius, variant hunt, fix hint
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK
  kernel: Group admitted symptoms only when one root mechanism + one fix
    boundary explain them; trace blast radius (provably-affected vs suspected);
    variant hunt; read may clear a variant, never confirm it.

- skill: 7060-review-feedback-synthesis
  evidence_of_value: `90-report/process-feedback.md` + `feedback/*.md` —
    quantified W-01 evidence loss and 4 concrete tooling recommendations
  relation: SITUATED
  disposition: MUTATE (see Dead & Delete)
  kernel: Capture leaf-agent obstacles as raw friction text; collate once into
    the report; never spend a dedicated reasoning stage re-deriving it.

- skill: 7065-review-assurance-retrospective
  evidence_of_value: none observed — no `assurance-retrospective.md` in any
    archive despite the skill declaring itself "standing on every campaign";
    the stage2 report omits the required retrospective section
  relation: DEAD
  disposition: DELETE (see Dead & Delete)
  kernel: none survives.

- skill: 7070-review-report
  evidence_of_value: `90-report/report.md` (309 lines: executive verdict,
    guarantee certificate, reproduced defects, recommended actions) — the
    terminal artifact the human gate reads; demand-census shows a defensible
    bounded minimum even with 0 executors
  relation: PORTABLE-CORE
  disposition: KEEP-IN-PLAYBOOK
  kernel: Compile guarantee/falsification verdicts, reproduced defects with
    proof lineage, coverage vs inventory, open/budget-cut risk, and one bounded
    conclusion; never invent evidence or upgrade an unvalidated result.
    Severity exactly as validated.

- skill: 7080-review-promotion
  evidence_of_value: none observed — no `ACTION.md` in any archive (all runs
    pre-gate; runs stay `complete-awaiting-promotion`)
  relation: SITUATED
  disposition: DELETE (choreography)
  kernel: none survives beyond a one-line decision record + human route.

## Distilled Playbook

Stored at depth as a breadcrumb-gated playbook. What actually produces the
value in the real outputs, in order. Six stages; three are the workflowengine
core loop.

**0. Framing & guarantees.** Pin target (SHA/diff/dir) and the human question.
Turn it into 3–20 falsifiable guarantees, each with an observable
counterexample and a completion rule. State the inventory whose completeness
makes the review meaningful, and exclusions. (Origin: 7005.)

**1. Recon fan-out.** Map the connected surface, then dispatch bounded
deep-read reviewers over *disjoint* surfaces or lenses. Each reviewer reads
whole functions (not diff hunks), follows callers/callees/registrations/
persistence, and reports a candidate only when a specific input/state/race
yields a specific forbidden outcome. The fan-out is the discovery engine
(`workflowenginereview`'s "7 parallel deep-read reviewers"; YEETZ_DB's
"every source file read end-to-end by a dedicated reviewer"). (7005-scout +
7010 + 7011.)

**2. Triage: dedupe + rank.** Merge candidates only when they assert the same
mechanism and proof path; preserve lineage. Rank by contract impact ×
likelihood × proof value — priority, not severity. Never silently drop: mark
budget-cut. (7020.)

**3. Re-verify high-impact items independently.** For the top tier, a disjoint
reviewer or rig re-checks against source. Tag every finding with an evidence
tier — VERIFIED (line-cited direct read) / VERIFIED-partial / reviewer-noted —
or SRC-ME / SRC-MULTI / SRC-1 for fan-out runs. Assign `recheck-first` to the
SRC-1/self-reported class; never rest anything load-bearing on unverified
reporting. Where a claim is cheap to disprove, build the smallest faithful
rig (controls + decisive case), keep it replayable, and have an independent
actor run it against the untouched target. (7030 attack-on-paper + 7035 + 7036.)

**4. Root-cause consolidation.** Group reproduced symptoms into defect
families (D-##) by root mechanism + single fix boundary; trace blast radius
as provably-affected vs suspected; hunt variants. A read may clear a variant,
never confirm it. (7050.)

**5. Report + human gate.** Compile guarantee verdicts (falsified/qualified/
unresolved), reproduced defects with proof lineage, coverage vs inventory,
open and budget-cut risk, recommended actions. Never invent evidence, never
upgrade an unvalidated result, never self-promote — the human decides. (7070.)

**Optional:** *test-integrity* stage (only when the claim rests on tests;
design the break that should fail each test, 7025) and *one bounded coverage
wave* (7040) fold into the report audit. Raw leaf friction notes are collated
into the report as-is, never re-derived by a dedicated self-audit stage.

Evidence rules, unchanged from the real outputs: cite `path:line` for every
finding; keep replayable rigs with exact commands, run count, and untouched
target; every reproduced finding carries an independent-validation lineage;
separate "reviewer-reported" from "independently verified" in the artifact's
own legend.

That is the whole method behind C1/C2/C3, the 12 root-cause families, and the
"NOT READY FOR STAGE 2" verdict — without 15 skills.

## Breadcrumb

> **Adversarial review (reflect).** Must be invoked by the human for a deep,
> expanded, or hard review — delivery-proof audit of a change bank, hard
> debugging of a difficult defect, a hard code review where a self-review
> would be trusted too cheaply, or an explicit "prove me wrong" request. Never
> auto-triggered by a passed test, a merge, or a scoped code review; this
> spends real reviewer budget to *attack* shipped behavior. Gate: run only
> when the answer must be adversarially bought, not when a normal review
> suffices. Point to the stored **Review Playbook** (framing → recon fan-out →
> dedupe/rank → independent re-verification → root-cause → report + human
> gate); fan out 2–n parallel deep-read reviewers by disjoint surface, rank
> their findings, and independently re-verify the top tier by direct source
> read, tagging every finding VERIFIED / partial / reviewer-reported.

## Dead & Delete

- **7065-review-assurance-retrospective — DELETE.** Triple-kill. (1) Zero
  observed output — `find code-reviews -name assurance-retrospective.md`
  returns nothing; the stage2 report, which the 7070 skill demands carry this
  section, omits it. The machinery is aspirational, not exercised. (2) It is
  the purest self-audit: audits the campaign's *own* proofs, retries, and
  wall-clock to render cost verdicts on itself — exactly what the anti-self-audit
  ruling forbids (the review spends Sol on introspecting its own mechanics
  instead of the target, and invites regress: who audits the auditor?). (3)
  Its own discipline admits it must never machine-verify anything ("Do not
  relitigate findings, weigh proof merits, inspect product source, run
  Cargo"), i.e. it produces human-discussion prose with no load-bearing
  verdict — highest ceremony, lowest yield.

- **7060-review-feedback-synthesis — MUTATE, not delete.** Same self-audit
  sin *as a stage* (it reviews the campaign's own W-01 failure), and its
  output is genuinely useful (quantified evidence loss, 4 concrete tooling
  fixes) — but it earned that value from raw `feedback/*.md` friction written
  by *leaf agents during execution*, not from a dedicated Sol synthesis. Keep
  the portable kernel: leaves report what blocked them as passive passback
  notes; the report stage collates them once, verbatim. Delete the dedicated
  "synthesis" role — no review budget spent auditing the review.

- **7080-review-promotion — DELETE as a skill.** Zero observed output (no
  `ACTION.md` in any archive; every run parks at `complete-awaiting-promotion`).
  Purely SITUATED bookkeeping — `runs/7000` paths, git-policy, downstream
  1000/2000/3000/5000/6000 route decisions. Survives as one line in the
  playbook: "record the human decision, prune rejected/refuted repros, route
  accepted actions to their lane." No adversarial value.

- **7040-review-gapfill — FOLD** into the report-stage coverage audit. Its
  one real contribution (the single allowed W-01 wave) is a refinement of
  stage sweep, not discovery; the "never silently expand budget" rule is
  already the playbook's framing/coverage invariant.

## Surprises

1. **The archive already IS the proof of the anti-bureaucracy case.** The
   15-skill machinery produced at most one distinct valuable output per stage,
   and the four strongest value-demonstrating artifacts
   (`workflowenginereview.md`, `YEETZ_DB_REVIEW_FINDINGS.md`) are *free-form
   files* that use zero of the ceremony — just fan-out, tiers, dedupe/rank,
   re-verify. The formal run archives (stage2) replicate that value in
   structured form while spending 24 child spawns to do it.
2. **The two skills the human flagged as self-audit-suspect have the worst
   reinforcement possible:** 7065 produced *no* output and 7060's own output
   proves its best material was captured downstream of execution, not by a
   stage. The family's cheap-to-delete members are the ones doing self-audit.
3. **Evidence tiers are the portable crown jewel** — VERIFIED /
   VERIFIED-partial / reviewer-reported and SRC-ME / SRC-MULTI / SRC-1 are
   identical in kind, appear verbatim in both free-form teardowns, and are
   what let an engineer trust C1/C2/C3 at a glance. This survives distillation
   unchanged.
4. **Demand-census proves the loop works with zero rigs:** 8 source-adjudication
   packets, 0 executors, 0 validators, yet a defensible bounded minimum and a
   1-QUALIFIED/12-UNRESOLVED honest verdict. Independent *source* re-verification
   is a first-class proving mode, not a degraded one — workflowenginereview's
   C1/C2/C3 were all source-verified reads.
5. **The 7065-retrospective-mandated-in-report coupling is a dangling
   dependency:** the 7070 report demands a section the archive never
   materializes. Deleting 7065 requires deleting that required-section line
   from the distilled report stage, else the playbook inherits an impossible
   obligation.

---
**Headline:** The 15-skill family compresses to a 6-stage playbook whose first
three stages are exactly the one-line method in `workflowenginereview.md`:
fan out parallel deep readers, dedupe/rank, independently re-verify — with
evidence tiers as the portable spine. Recon fan-out, triage/dedupe/rank,
independent validation, root-cause, and report demonstrably produced every
real finding; charter and gapfill fold into framing/sweep. 7060/7065 are the
self-audit stragglers the ruling predicted — 7065 had zero observed output and
gets DELETEd, 7060 keeps only its raw leaf-friction as passive passback.
Promotion is SITUATED git-policy bookkeeping with no observed output and
drops to a one-liner. What portables (method, tiers, dedupe/rank, independent
re-verification) is repo-agnostic; what dies (Sol/Terra names, runs/7000
paths, guarantee-registry ceremonies, retrospection) is the bureaucracy.
