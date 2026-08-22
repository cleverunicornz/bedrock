# Census 17: LD Format & Rust RDF Stack

Decision-support for the "bedrock" bootstrap tool: which `*-LD` authoring format and which Rust RDF library stack. Source-verified in the external clone bank.

Bank: `$HOME/data/external-repositories`. Clones inspected:
- `oxigraph-oxigraph` @ `6757cd8f` (2026-08-17)
- `timothee-haudebourg-json-ld` @ `17854d0` (0.21.4, 2026-02-19)
- `pchampin-sophia_rs` @ `e9d4a4b` (2026-08-20)

crates.io versions/dates queried from the crates.io API (all links are `https://crates.io/crates/<name>`).

---

## Oxigraph Subcrates

`oxigraph/oxigraph/lib/` contains 13 crates (dir listing, `oxigraph-oxigraph/lib`). The load-bearing ones:

| Crate | Purpose | Standalone on crates.io? | Version / date | JSON-LD | TriG | Maintenance |
|---|---|---|---|---|---|---|
| **oxjsonld** | JSON-LD **parser AND serializer** (`JsonLdParser`, `JsonLdSerializer` in `lib/oxjsonld/src/lib.rs:1,30`); full 1.1 expansion in `expansion.rs` (137 KB), serialization in `to_rdf.rs`/`from_rdf.rs` | **YES** (`lib/oxjsonld/Cargo.toml:3` `name="oxjsonld"`; docs.rs badge in README) | 0.2.5, 2026-04-19 (master is 0.3.0-dev) | **FULL JSON-LD 1.1**: `JsonLdProcessingMode::JsonLd1_1` (`profile.rs:257-258`); validates `@version 1.1`, `@import`, `@protected`, `@propagate`, `@direction` (`context.rs:228,243,253,392,421,439`); streaming profile (`JsonLdProfile::Streaming`, `README.md`) | n/a (not a TriG crate) | Active (oxigraph releases Apr 2026) |
| **oxttl** | Turtle/TriG/N-Triples/N-Quads/N3 parser **and serializer** (`lib/oxttl/README.md:1`) | YES | 0.2.3, 2026-02-14 | n/a | **TriGSerializer** with prefix control + auto graph grouping + pretty (see below) | Active |
| **oxrdf** | RDF 1.1/1.2 data model (Quad, Dataset, Graph) | YES | 0.3.3, 2026-02-14 | n/a | quad model (`Quad` at `lib/oxrdf/src/triple.rs:1102`) | Active |
| **oxsdatatypes** | RDF literal datatypes (XSD) | YES | 0.2.2, 2025-01-11 | n/a | n/a | Slower cadence, still maintained |
| **oxrdfio** | unified RDF read/write facade over oxttl/oxrdfxml/oxjsonld | YES | 0.2.5, 2026-04-19 | wraps oxjsonld | wraps oxttl | Active |
| sparql-smith / spargeo / sparopt / spargebra / spareval / sparesults / oxrdfxml / oxigraph | SPARQL engine + RDF/XML | YES | — | n/a | n/a | Active |

**Decisive finding:** `oxjsonld` exists and is **published standalone** (0.2.5). It is a full JSON-LD 1.1 processor, not a streaming-only quad emitter.

**Remote-context control (the security-critical bit):** loading is **OFF BY DEFAULT and hard-forbidable**. If a document references a remote `@context` and no `load_document_callback` was set, expansion fails with `JsonLdErrorCode::LoadingRemoteContextFailed` — `"No LoadDocumentCallback has been set to load remote contexts"` (`lib/oxjsonld/src/context.rs:1374-1377`). The callback is wired via `JsonLdParser::with_load_document_callback` (docs mirror the JSON-LD API `documentLoader`, `lib/oxjsonld/src/to_rdf.rs:372,408`). A callback can therefore serve exactly ONE local context (e.g. the `https://yeetz.dev/context/execution/v1` map) and reject every other URL — precisely the org's "no remote contexts" rule. `max_context_recursion` caps at `MAX_CONTEXT_RECURSION = 8` (`lib.rs:31`).

**TriGSerializer (near-full emitter provided):** `TriGSerializer::new().with_prefix(...).for_writer(...)` (`lib/oxttl/src/trig.rs:772-775`). Prefixes live in a `BTreeMap<String,String>` (`trig.rs:774`) → **lexical, deterministic `@prefix` prelude**. The low-level writer auto-groups quads into `GRAPH <g> { … }` blocks, joins repeated same-subject/same-predicate into `;`/`,` Turtle syntax, and indents graph bodies with tabs (`trig.rs:1116-1201`). Deterministic output therefore reduces to: **sort quads** by a stable key and feed them in order; the serializer does grouping, prefix compression, and pretty-printing for you. `Quad` derives `Eq/PartialEq/Clone/Hash` but **not `Ord`** (`lib/oxrdf/src/triple.rs:1102-1107`) → sort by your own key (e.g. `(graph, subj, pred, obj)` strings). This is the single strongest reason to build the emitter on oxrdf+oxttl.

**JSON-LD open-issue posture note:** oxjsonld implements the JSON-LD 1.1 algorithm from the W3C spec directly and ships its own `json-event-parser`; it does not reuse the separate `json-ld` crate. Dep tree is tiny: `json-event-parser`, `oxiri`, `oxrdf`, `ryu-js`, `thiserror` (`lib/oxjsonld/Cargo.toml:20-26`).

---

## json-ld Crate

`timothee-haudebourg-json-ld` @ 0.21.4 (2026-02-19), crates.io `json-ld`.

- **Capabilities:** full JSON-LD **1.1** processor, split into a 7-crate workspace (syntax, core, context-processing, expansion, compaction, serialization, testing) — `Cargo.toml:25-31`. Since 0.21 it ships a JSON-LD **serializer** (`Serialize RDF as JSON-LD Algorithm`) as well as expansion/compaction (`src/processor/mod.rs`, `58 KB`).
- **Loader control (hard-forbid + one local context):** the `Loader` trait (`crates/core/src/loader/mod.rs:468`) ships a **`NoLoader`** that always errors with `CannotLoad` (`crates/core/src/loader/none.rs:18-23`), and **`HashMap<IriBuf, RemoteDocument>` implements `Loader`** (`crates/core/src/loader/map.rs:12-18`) — i.e. an embedded one-entry table mapping the single permitted context URL to its parsed map; every other URL rejects. Also `FsLoader` and `ReqwestLoader` (the latter behind the `reqwest` feature). **Default features are empty** (`Cargo.toml:13-14`) → no network loader compiled in unless explicitly opted in.
- **Dependency weight:** heavy for the use case. The umbrella `json-ld` pulls `json-syntax`, `locspan`, `iref`, `rdf-types`, `contextual`, `futures`, `thiserror` + 6 internal crates (`Cargo.toml:6-19`; workspace pins `hashbrown`, `smallvec`, `mown`, `educe` in `[workspace.dependencies]` `Cargo.toml:55-85`). It is async (`futures`), span-aware (`locspan`), and emits its **own `rdf-types` graph model — not oxrdf quads**. That means a JSON-LD→oxrdf bridge would have to convert models or round-trip through N-Quads, adding a conversion step oxjsonld avoids.
- **Maintenance:** actively maintained (0.21.4, Feb 2026; `json-ld-expansion` etc. all at 0.21.4). Rust-version 1.83 (`Cargo.toml:44`).
- **API ergonomics:** capable but heavier; the processor is async and vocabulary/interning-centric (`IriVocabularyMut`). For "expand one document against one embedded context," Oxford's `oxjsonld` is a slimmer synchronous fit with the same output model as the TriG emitter.

---

## sophia

`pchampin-sophia_rs` @ 0.10.0 (2026-05-19, crates.io `sophia`), a workspace (`jsonld/`, `turtle/`, `iri/`, `term/`, `inmem/`, …).

- **jsonld feature:** `sophia_jsonld` (ships parser + serializer for JSON-LD 1.1; `jsonld/README` "`sophia_jsonld` provides parsers and serializers for JSON-LD 1.1"). `JsonLdSerializer` defaults to `NoLoader` (`jsonld/src/serializer.rs:26,34`); loaders include `static_loader`, `closure_loader`, `file_url_loader`, `chain_loader` (`jsonld/src/loader/`). Relies on **`sophia_api` / `sophia_term`** as its data model (`jsonld/Cargo.toml` deps) — again **not oxrdf**.
- **TriG serialization:** `sophia_turtle` has `serializer/trig.rs` (`TriGSerializer` with `new`, `serialize_quads`, `new_stringifier` — `turtle/src/serializer/trig.rs:46-147`); no visible per-prefix ordering API at that boundary (`trig.rs` shows no `with_prefix` beyond the streaming serializers).
- **Quad model for the emitter:** `SophiaTerm`-based; a working TriG emitter exists but the prefix/grouping control is less explicit and symmetric than oxttl's `BTreeMap`-prefix `TriGSerializer`. Reuses sophia's own graph traits rather than oxrdf.
- **Verdict vs oxigraph crates for our narrow use:** functional but second-choice. It would introduce its own term model (extra conversion) and its TriG serializer surface is thinner than oxttl's for deterministic prefix control. Use it only if oxrdf/oxjsonld were a blocker (they are not).

---

## YAML libraries (serde_yaml deprecation / 2026 choice)

- `serde_yaml` is **deprecated**: crates.io max_version `0.9.34+deprecated`, last publish 2024-03-25; upstream archived by dtolnay.
- `serde-yml` (the original fork) is **itself now deprecated**: its crates.io description reads *"DEPRECATED — `serde_yml` is unmaintained. This release is a thin compatibility shim that forwards every call to `noyalib`"* (2026-05-27). 
- **Sanest 2026 YAML choice: `noyalib`** — crates.io desc "A pure Rust YAML library with zero unsafe code and full serde integration", max 0.0.27. (Alternative: `yaml-rust2`, "fully YAML 1.2 compliant", 0.12.0, 2026-08-18; or `serde_yaml_ng` 0.10.0, 2024.) For a JSON→YAML bridge we only need YAML → `serde_json::Value`; `noyalib` gives zero-unsafe + serde, the migration target the ecosystem itself names.

---

## Pipeline Options (ranked)

**Ranking criterion: least moving parts.** The deterministic TriG emitter needs sorting in every option; the question is which stack makes sorting → `TriGSerializer` cheapest.

### (1) JSON-LD source → oxjsonld → quads → sorted → oxttl TriGSerializer — **least moving parts**
- Stack: `oxjsonld 0.2.5` + `oxrdf 0.3.3` + `oxttl 0.2.3`.
- `JsonLdParser` yields **oxrdf `Quad`s directly** (`oxjsonld/README.md` example iterates `Triple`s; `to_rdf.rs` expands to quads). Zero model conversion into the emitter — the same `oxrdf::Quad` the `oxttl::TriGSerializer` consumes.
- Determinism: sort quads by your key, feed `TriGSerializer` (auto `GRAPH{}` grouping, sorted `@prefix` prelude, `;`/`,` joining). Custom work = the sorter + fixed prefix table.
- Remote context: forbidden by default via `load_document_callback`; serve the one embedded context map.
- Schema-validation gate: JSON parses straight to `serde_json::Value` → `jsonschema 0.50` (0 conversions).
- Conversions total: **1** (LD→quads). Crates: 4 direct.

### (2) YAML-LD source → noyalib → JSON value → oxjsonld → quads → oxttl TriGSerializer — **one added parser**
- Stack: `noyalib 0.0.27` + `oxjsonld` + `oxrdf` + `oxttl` (+ `jsonschema`).
- YAML 1.2 is a superset of JSON, so a YAML doc's tree == the JSON tree; `noyalib → serde_json::Value` is ~5 lines and feeds the *same* oxjsonld path as (1). Same emitter, same determinism, same remote-context control.
- Schema-validation gate: YAML → `serde_json::Value` (already parsed for the pipeline) → `jsonschema`. 1 conversion reused, not extra.
- Conversions total: **2** (YAML→JSON, LD→quads). Crates: 5 direct. Adds exactly one parser dependency; nothing else forks.

### (3) Direct Turtle/TriG authoring (no LD source) — **zero conversion, loses the schema/expansion layer**
- Stack: `oxrdf` + `oxttl` (`TriGParser`/`TriGSerializer`). oxjsonld not even present.
- **Gained:** native `#` comments (the one format with real comments), no conversion step, authors write the final artifact, no LD processor dependency at all.
- **Lost:** no JSON Schema structural validation (would need a hand-rolled quad-walker instead of `jsonschema`); no `@context` compact-term expansion (authors write full `<http://…>` IRIs or a primitive prefix table by hand); no graph-shape guarantees beyond what a walker enforces. This is the format you project TO, not the one you want humans/agents authoring as canonical source.

**Assessment of each stack's quad model for the custom emitter (all options):** oxrdf is the clean winner — `Quad` + `Dataset` (`dataset.rs` SPOG indices, `iter()`, `quads_for_*`), and the native `oxttl::TriGSerializer` speaks oxrdf `Quad` natively with graph grouping + prefix ordering built in. json-ld (rdf-types) and sophia (SophiaTerm) both require a model conversion to reach the same emitter.

**Rank: (1) ≈ (2) for machinery; (2) for authoring; (3) only as the output target.** Because the org's canonical sources are already YAML with comments/multiline prose (next section), **option (2)** is the recommended build; option (1) is the identical machinery with the identical determinism path and one fewer parser.

---

## Format Ergonomics (side-by-side, real artifacts)

### A. Execution record — `situation/execution/exec_1786896702093_000000007493.yamlld` (Commission transition /0000)

**YAML-LD original** (verbatim; note the leading `#` comment is real):

```yaml
# Live revision-three canary republished for cache-projection qualification.
"@context": "https://yeetz.dev/context/execution/v1"
"@id": "https://yeetz.dev/graph/execution/exec_1786896702093_000000007493"
"@graph":
  - "@id": "https://yeetz.dev/execution/exec_1786896702093_000000007493/transition/0000"
    "@type": "https://yeetz.dev/ontology/Commission"
    sequence: 0
    actor: "urn:yeetz:actor:gateway-v3-integration-orchestrator"
    causalClass: "https://yeetz.dev/definition/orchestrator"
    lane: "https://yeetz.dev/definition/lane-90000"
    intent: "Prove that one exact pushed execution projection reaches a clean external harness through Gateway V3 without repository mutation."
    acceptanceCriteria:
      - "The selected worker returns only the sentinel carried by its bound execution generation."
      - "The worker performs no tool call and makes no repository mutation."
    admittedActs:
      - "https://yeetz.dev/act/implement"
      - "https://yeetz.dev/act/validate"
      - "https://yeetz.dev/act/close"
    consumes:
      - "https://yeetz.dev/execution/exec_1786896702093_000000007493/artifact/inherited-contract"
```

**JSON-LD equivalent** (embedded context required, since remote loading is forbidden; the `#` comment has nowhere to live):

```json
{
  "@context": {
    "sequence": "https://yeetz.dev/ontology/sequence",
    "actor": { "@id": "https://yeetz.dev/ontology/actor", "@type": "@id" },
    "causalClass": { "@id": "https://yeetz.dev/ontology/causal-class", "@type": "@id" },
    "lane": { "@id": "https://yeetz.dev/ontology/lane", "@type": "@id" },
    "intent": "https://yeetz.dev/ontology/intent",
    "acceptanceCriteria": { "@id": "https://yeetz.dev/ontology/acceptance-criteria", "@container": "@set" },
    "admittedActs": { "@id": "https://yeetz.dev/ontology/admitted-acts", "@type": "@id", "@container": "@set" },
    "consumes": { "@id": "https://yeetz.dev/ontology/consumes", "@type": "@id", "@container": "@set" }
  },
  "@id": "https://yeetz.dev/graph/execution/exec_1786896702093_000000007493",
  "@graph": [
    {
      "@id": "https://yeetz.dev/execution/exec_1786896702093_000000007493/transition/0000",
      "@type": "https://yeetz.dev/ontology/Commission",
      "sequence": 0,
      "actor": "urn:yeetz:actor:gateway-v3-integration-orchestrator",
      "causalClass": "https://yeetz.dev/definition/orchestrator",
      "lane": "https://yeetz.dev/definition/lane-90000",
      "intent": "Prove that one exact pushed execution projection reaches a clean external harness through Gateway V3 without repository mutation.",
      "acceptanceCriteria": [
        "The selected worker returns only the sentinel carried by its bound execution generation.",
        "The worker performs no tool call and makes no repository mutation."
      ],
      "admittedActs": [
        "https://yeetz.dev/act/implement",
        "https://yeetz.dev/act/validate",
        "https://yeetz.dev/act/close"
      ],
      "consumes": [
        "https://yeetz.dev/execution/exec_1786896702093_000000007493/artifact/inherited-contract"
      ]
    }
  ]
}
```

### B. Knowledge vertex — `future-epoch-candidate/memory-attenuator/situation.yamlld` (Invariant "Causal first")

**YAML-LD original:**

```yaml
  - "@id": "https://yeetz.dev/memory-attenuator/causal-first"
    "@type": "https://yeetz.dev/ontology/Invariant"
    label: "Causal premise before recent activity"
    instruction: "First preserve why the supported agent is here. Its causal premise and expected outcome or inquiry posture outrank the most recent local action. Explicit human steering may evolve the premise; a recent fix may not silently re-originate the agent's agenda."
    priority: 2
```

**JSON-LD equivalent (embedded context, cut to the object):**

```json
    {
      "@id": "https://yeetz.dev/memory-attenuator/causal-first",
      "@type": "https://yeetz.dev/ontology/Invariant",
      "label": "Causal premise before recent activity",
      "instruction": "First preserve why the supported agent is here. Its causal premise and expected outcome or inquiry posture outrank the most recent local action. Explicit human steering may evolve the premise; a recent fix may not silently re-originate the agent's agenda.",
      "priority": 2
    },
```

### Concrete comparison

| Axis | YAML-LD (real files) | JSON-LD |
|---|---|---|
| **Comments** | **YES.** Real exec file opens with `# Live revision-three canary republished…`. Knowledge-vertex files are grep-friendly prose. | **None.** The canary line has no home; would either be dropped (provenance loss) or forced into an `rdfs:comment`-style quad (adds graph semantics for a non-semantic note). This is the highest-value difference. |
| **Multiline prose** (`intent`/`instruction`/`description` strings) | Natural. Long strings stay single-line or use block scalars; diffs read word-level. | A long prose sentence is one quoted `\n`-escaped blobby line; structured wrap must be hand-nailed; diff noise on any reflow. |
| **Quoting noise** | Minimal: `@id`/`@type` and IRI-valued keys quoted (`"@"` keys at top level + context); data keys (`sequence:`, `priority:`) bare. | **Every** key and value quoted. The `@context` map alone adds ~15 quoted lines to every file. |
| **Diff-friendliness** | Small per-line deltas; the sparse quoting keeps meaningful tokens visible. | Full-rewrap of quoted JSON on any nesting change; `@context` prelude churns every structural edit. |
| **Agent-authoring failure modes** | YAML: indentation drift, "norway problem" (unquoted value containing `: ` becomes a map) — mitigated here by the org's quote-`@`-keys convention; **anchors/aliases must be banned** (profile already says no blank nodes, but YAML `&`/`*` aliases are an extra hazard to reject). | JSON: **trailing commas** (the classic LLM emit bug), missing quotes, no comments → agents cannot leave explanatory notes. Strict-parse gate catches both, but YAML's syntax surface produces fewer of the silk-moth failures. |
| **JSON Schema checkability** | **Yes, via YAML→JSON then draft 2020-12** (`execution-record.schema.yaml` is already draft 2020-12 with `$defs`/regex patterns). One parse, reused by the pipeline. | **Yes, natively** (parse JSON, validate). Zero extra hop. |

Bottom line for ergonomics: both are JSON-Schema-checkable; the decisive differences are **comments** (YAML has them, JSON cannot) and **multiline prose + quoting noise** (YAML winning). The org's own documents lean on comments (`trig-projection.md` "Validate YAML syntax…", the real `#` canary) — that preference is load-bearing, not incidental.

---

## Recommendation Matrix

| Criterion | JSON-LD-only | YAML-LD-only | Hybrid (JSON-LD exec-graphs + YAML-LD vertices) | Direct TriG |
|---|---|---|---|---|
| Conversion count (to quads) | 1 | 2 (YAML→JSON, LD→quads) | 2 code paths | 0 |
| Library risk | Low (oxjsonld only) | Low (adds noyalib, a young 0.0.x) | **2× authoring+validation surfaces** | Lowest |
| Agent ergonomics | Worst (no comments, quotes, trailing-comma trap) | Best (comments, minimal quoting) | Split-brain; agents pick format by mood | Worst for agents (raw IRIs) |
| Human review | Poor (no annotations) | Good (comments + compact) | Mixed | Fair (readable, but verbose) |
| Schema validation | Native JSON | Yes (via YAML→JSON) | Yes, two schemas | **None** (quad-walker only) |
| Determinism path | Strong (oxttl sorted-prefix) | Strong (same emitter) | Strong but duplicated | Strong (same oxttl) |
| **Totals** | 4 crates, 1 conv | 5 crates, 2 conv | ~6 crates, 2 emitters, 2 schemas | 3 crates, 0 conv, no schema/comments-emitter redundancy |

**Recommended: YAML-LD-only**, stack `noyalib 0.0.27` → `serde_json::Value` → `oxjsonld 0.2.5` → `oxrdf 0.3.3` quads → sorted feed into `oxttl 0.2.3::TriGSerializer`, `jsonschema 0.50` gate on the same value. It keeps the org's real authoring posture (comments, compact `@`-quoted keys, draft-2020-12 schema), and the only added moving part over the pure-JSON path is one YAML→JSON parse that the schema gate reuses. Determinism is identical to option (1) because both converge on the same oxrdf→oxttl emitter.

**Runner-up: JSON-LD-only** (same oxrdf/oxttl stack, drop noyalib). Choose it only if the org accepts: no in-file comments, mandatory embedded `@context` block in every file, and the trailing-comma/quote agent-failure surface. Machinery is otherwise identical and equally deterministic.

**Explicitly not recommended: hybrid.** The org's machine-heavy execution records are **also YAML today** and also benefit from comments/provenance notes; there is no class of artifact that gains from being JSON. The two-grammar cost is real and measurable (two authoring profiles, two schema suites, two load/expansion paths, reviewers switching contexts) and buys nothing the YAML path lacks. **Direct-TriG** stays as the *output* target only; as a *source* it forfeits JSON Schema validation and `@context` compact terms, which the whole `situation/`+`future-epoch-candidate/` design depends on (`execution-record.schema.yaml` is a draft-2020-12 schema with regex `$defs` — not TriG-checkable).

---

## crates.io Name Check

- **`bedrock` is taken**: `bedrock 1.1.70`, last publish **2018-04-09**, description "Glue library between Vulkan and Rust". Stale/unmaintained, so squatting isn't active, but the name is **not available** for a fresh `bedrock` crate.
- Alternatives (unchecked availability): `bedrock-cli`, `bedrock-ld`, `bedrock-situation`, `bedrock-tool`, `bedrockbootstrap`, `situation-ld`, `yeetz-bedrock`. Given the org's lane naming, `yeetz-bedrock` is the safest collision-avoiding choice; verify any pick at `crates.io/api/v1/crates/<name>` before publishing.

---

## Surprises

1. **oxjsonld is a full published JSON-LD 1.1 processor — the "no Rust JSON-LD" assumption is dead.** It serializes AND expands, forbids remote context by default, and outputs oxrdf quads (same model as the TriG emitter). The decisive question resolves in oxigraph's favor.
2. **The deterministic TriG emitter is ~90% already written**: `oxttl::TriGSerializer` auto-`GRAPH{}`-groups, emits a sorted `BTreeMap` `@prefix` prelude, and `;`/`,`-joins — you only sort quads (Quad lacks `Ord`, `oxrdf/src/triple.rs:1102`).
3. **`serde-yml` — the fork meant to replace deprecated `serde_yaml` — is itself deprecated** in favor of `noyalib`. The ecosystem has moved twice in two years.
4. **`bedrock` (the tool's own name) is squatted by a 2018 Vulkan glue crate.** Pick a suffixed name.

---

## Headline Summary

- `oxjsonld 0.2.5` is a published, standalone **JSON-LD 1.1 parser+serializer**, remote-context-loading forbidden by default → the JSON-LD gap is closed.
- Best emitter path: oxrdf quads → `oxttl::TriGSerializer` (auto graph-grouping, sorted prefixes) → determinism is mostly free; you only sort quads.
- **YAML-LD is the recommended source** (`noyalib`→JSON→oxjsonld): real comments, multiline prose, and compact quoting that JSON-LD cannot provide.
- `json-ld` (rdf-types/async) and `sophia` both work but insert a model conversion oxjsonld avoids — second-choice.
- `bedrock` name is taken (2018 Vulkan glue); use a suffixed alternate.
