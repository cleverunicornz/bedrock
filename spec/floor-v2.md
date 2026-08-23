# Floor v2 — Re-envisioned Against the Reference Repos

Inputs: censuses 01–12 (monorepo extraction), censuses 13–15 (yeetz forge,
yeetz-s3-kernel, gh-runners), orchestrator spot-checks of every load-bearing
claim. The reference repos were built free of the workflow headspace; they are
the criteria the floor hypothesis was tested against, per the predeclared
method: RELIES confirms, CONTRADICTS mutates, EXTENDS absorbs, falsified
deletes.

---

## 1. The Register Correction (the biggest finding)

Both reference repos independently converged on the same AGENTS.md shape:

> **N numbered invariants ("breaking any of these is wrong, whatever else is
> right") + "where things live" + a handful of repo-specific process skills.**
> yeetz: 37 lines, 5 skills. yeetz-s3-kernel: 60 lines, 2 skills.

No procedure. No loop choreography. No vocabulary tables. Principles moved
into code gates (boundary lints, CI) and immutable ADRs; the agent-facing law
is one screen. **Floor v1's "~7–8 pages" was still monorepo-brained.** Floor
v2 is written in the reference register: portable invariants + breadcrumbs;
each repo appends its own situated invariants below them. The compiled
projection (graph epoch) renders into exactly this register.

## 2. Moon — Final Verdict

- **yeetz-s3-kernel: ABSENT.** Zero references; raw cargo + 2 boundary lints
  + nextest in CI sufficed (verified).
- **yeetz: named, never executed.** `.moon/` holds only `workspace.yml`; no
  tasks/toolchain files; CI runs raw `cargo build --locked` + `cargo nextest
  run --workspace` (verified `.github/workflows/ci.yml:22-24`). AGENTS.md #7
  fixes "Moon + cargo-nextest; task names build/check/nextest/clippy — never
  invent a second one." Moon's surviving role is a *vocabulary pin against
  toolchain proliferation*, and the invariant carries that value without the
  tool.
- **gh-runners: provisions Moon 2.4.5** — for the monorepo's benefit only; a
  platform, not a user.
- **Verdict: Moon is not floor and is not seeded into new repos.** It is
  drawdown machinery of the monorepo and dies with the extraction. What
  survives into the floor is the invariant it was standing in for: *one fixed
  toolchain per repo; canonical task names; never invent a second one.* The
  monorepo keeps Moon until teardown completes; yeetz's parity label is
  harmless but optional.

## 3. Distribution Model — Confirmed Operating

`origin/main:Cargo.toml` in yeetz already implements the model end-to-end
(verified): kernel closure consumed as git deps at tag `v0.1.0`, source of
record declared to be the kernel repo, extension human-gated there, consumers
take tagged releases only, kernel-closure pins mirrored with "assured at
these versions — do not bump casually." The extraction chain is
monorepo → yeetz → yeetz-s3-kernel: extraction is already recursive.

**The boundary law (new floor, stated from evidence):**
The dependency boundary is the assurance boundary. What crosses it
version-pinned is assured-by-default at that pin; you verify your
*consumption*, never the dependency's internals — the same law root already
held for adopted protocols (S3, Matrix, Git), generalized to every crate
boundary. Pins are assured versions; bumps are deliberate acts. A missing
capability at a consumed boundary is BLOCKING — stop and escalate; working
around a boundary is the disqualifying failure.

## 4. Floor v2 — The Portable Invariants

Every org repo inherits these; each repo appends its own situated invariants
and its 2–5 repo-specific process skills below. Final prose.

1. **Code is a possibility space.** A surface does everything it *can* do,
   not what its author meant. Work is collapsing possibilities to defined
   behavior. A green test proves a behavior exists — never that nothing else
   happens. Say what else could happen; declare what you did not collapse.
2. **A gap has three suspects** — the code, the requirement, or the
   instrument. Interrogate in the open before displacing any of them.
3. **Situate before acting; re-situate after.** Relationships carry unequal
   mass; the loudest, freshest signal is usually the smallest. Interrogate a
   contradiction before displacing what it contradicts. Authority attaches to
   landed sentences, not to your readings of them.
4. **Completion is behavior at the promised boundary.** Nothing in a
   deliverable is a stub, placeholder, or deferred branch. Not done:
   "compiles", "structure in place", "foundation for later."
5. **Make it first, prove it after.** Build the slice, prove it, fix,
   continue. Teardown on demand, never a precondition. Deep adversarial
   review is human-invoked (→ Review Playbook breadcrumb).
6. **Verify by regenerating from source**, never by reading the claim. Gate
   claims cite a CI run URL, never a local attestation.
7. **Predeclare criteria** before the run that answers them; judge results
   only against them. A refuted hypothesis is a successful experiment. Mixed
   outcomes stay mixed — never round toward a verdict. Timebox exploration;
   when the decision point arrives, resolve with what you have.
8. **The dependency boundary is the assurance boundary.** (§3 verbatim.)
9. **Public-first.** Before building inside, ask why this cannot be a public
   crate or repo; work backwards from that. Extract when the rubric (§5)
   passes; private only for named domain knowledge or secrets.
10. **Git:** force push does not exist; nothing pushed is deleted; one writer
    per ref; PRs always — a human merges the default branch. Same policy in
    every repo.
11. **CI:** every Linux/platform-neutral job runs on `org-ci-linux-x64`;
    WarpBuild only for native macOS/Windows artifacts
    (`warp-macos-15-arm64-6x`, `warp-windows-latest-x64-4x`); fork-PR code is
    mechanically rejected before runner assignment; a missing host tool is a
    P0 infrastructure defect — never install a hidden substitute; CI runs the
    real suite (fmt, clippy `-D warnings`, locked build, nextest).
12. **One fixed toolchain per repo,** canonical task names; never invent a
    second one.
13. **State:** one storage truth through the kernel; records are immutable,
    created by conditional write; pointers move by CAS; every projection is
    rebuildable from records; reads are fenced; an integrity failure is never
    translated to absence.
14. **Tailscale-first:** all internal reach rides the tailnet; SSH-over-
    Tailscale for operators; nothing internal is served outside it;
    public-IP access is break-glass only.
15. **Decisions are append-only:** ADRs are immutable — supersede, never
    edit.
16. **AGENTS.md is the compiled graph:** one artifact — the complete TriG
    compiled from `situation/`, injected into every agent context by the
    harness. Never hand-edited; always regenerated. Each node carries its
    description, its relationships, and a pointer to its own source
    document; humans read prose in `situation/`. After work that changes
    reality, re-situate it — emit knowledge, never rules.

**Breadcrumbs (gate + pointer, depth elsewhere):**
- *Adversarial review:* human-invoked (expansive review, hard debug, hard
  code review) → Review Playbook (framing → recon fan-out → dedupe/rank →
  independent re-verification with evidence tiers → root-cause → report).
- *Spike rig:* when a question needs a build — quarantine it, timebox it,
  predeclare criteria → Spike Playbook.
- *Rollout & runners:* Ansible, owned by `gh-runners` (roles/inventory/
  toolchain manifest); performance topology there is human-gated.
- *Org crate manifest:* check the org's published-crates manifest before
  reaching for third-party or writing new (**gap: does not exist yet —
  proposed home: the bootstrap repo**).
- *Structural search:* ast-grep preferred where provisioned; blind to
  macros/codegen — an empty structural result is never evidence of absence.
- *External source research:* read the checked-out clone (shared bank at
  `$YEETZ_EXTERNAL_REPOS` / `~/data/external-repositories`, `<owner>-<repo>`,
  `pull --ff-only`), not docs.
- *Storage classification (tooling):* ephemeral-test default — fresh bucket +
  scoped key, torn down on exit; retention needs a named owner and lifetime.
- *Settings poison guard:* a merged branch's agent-settings file is never
  accepted (`merge=ours`).

## 5. Externalization Rubric (derived from the two live cases)

Extract a surface into its own repo when all four hold:

1. **Universal concern** — consumed (or consumable) by multiple products; its
   semantics are not owned by one domain.
2. **Boundary-clean** — domain-agnostic API; no domain state, secrets, or
   private vocabulary crosses the boundary.
3. **Self-contained proof** — its tests, rigs, and contract suites travel
   with it and prove it at its own boundary, in its own CI.
4. **Mechanically enforceable boundary** — lints/CI can hold the line
   (extend-don't-bypass), so the boundary survives without vigilance.

Then **public-first**: public unless a named reason (domain knowledge,
secrets); private registry only when package count warrants it.

**Extraction template** (as performed by yeetz-s3-kernel, verified):
fresh history, provenance pinned in the root commit message; ADRs carried
verbatim with provenance recorded; rigs/contract suites travel; volunteer
full CI from day one; `v0.1.0` tag + exact pins; consumers move
path-dep → git-dep@tag → registry; consumer mirrors the closure pins
("assured at these versions"). Remaining for crates.io in the kernel's case:
README, `repository`/keywords/categories metadata, rustdoc pass.

## 6. Cross-Examination Verdicts (floor v1 → v2 movements)

| Floor v1 item | Evidence | v2 disposition |
| --- | --- | --- |
| Completion gate | Re-derived verbatim in yeetz `slice` skill; kernel CI enforces | KEEP (strongest confirmation) — adopt yeetz's sharper phrasing |
| State vertex | Re-derived twice from scratch (kernel invariants 3–7; yeetz data-model) | KEEP + absorb "integrity failure ≠ absence" |
| Witness practice | Kernel: "gate claims cite a CI run URL, never a local attestation" | MUTATE — CI-URL witness rule replaces witness-path machinery |
| Git rules 1,2,6 | Relied on in all three repos | KEEP |
| Git rule 4 (delegation docket) | Both repos: "No lanes, closures, or delegates — do not import them" (ADR 0006) | DEMOTE — harness-level concern, not repo floor; lives with delegation tooling |
| Git rule 5 (goals) | Silent in reference repos; tooling-specific | DEMOTE to harness level |
| 12-verb vocabulary | Behaviors survive in plain English; table nowhere needed | MUTATE — 5 load-bearing verbs folded into invariants 5–7; table dropped as artifact |
| Think/plan kernels (spike, proof-boundary) | SILENT (young repos, no campaigns yet) | KEEP compressed; monorepo-derived, unfalsified |
| Engineering values | Consistent where visible (typed DTOs, measurement rig in yeetz B9, exact pins) | KEEP |
| Tailscale vertex | Extended: runners tailnet-isolated under `tag:github-runner` | KEEP + fabric detail stays situated |
| Ops how-to depth | Reference repos carry none; process skills are repo-specific | CONFIRMED: depth lives with owners, 2–5 skills per repo |
| Moon (judgment #5) | §2 | CLOSED: not floor, not seeded; invariant 12 replaces it |
| CI-tests posture (judgment #3) | Both new repos volunteer full CI suites | CLOSED: monorepo's test-less gate was pathology, not intent; invariant 11 states the org posture |

## 7. Still Open For The Human

1. **Trunk-and-leaf** (judgment #1) — corrected dossier stands: corridor
   enforced and used in the monorepo; reference repos use plain PRs with no
   lane machinery and lose nothing observable. Choice: keep the corridor for
   the monorepo drawdown only (recommended shape: it dies with the monorepo,
   like Moon), or keep `merge/*` org-wide. Floor v2 is invariant to it.
2. **Independent verification cadence** (judgment #2) — reference repos rely
   on CI + human merge + on-demand teardown. Proposal stands: independent
   re-manufacture is an opt-in act for high-stakes claims, standing machinery
   nowhere.
3. **6020 reframe** (judgment #4) — graph-epoch question: compiled-AGENTS.md
   validation as a workflow gate vs a validator role. Bootstrap-repo design
   decision.
4. **Org crate manifest** — doesn't exist; proposed to live in the bootstrap
   repo beside the seed floor.
5. **Morph key rotation** — still pending your go (rotate at provider, then
   tree-side strip).
