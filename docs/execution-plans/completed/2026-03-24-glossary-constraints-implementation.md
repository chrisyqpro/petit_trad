# Implement glossary constraints with TDD gates

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `docs/PLANS.md`.

## Purpose / Big Picture

After this change, `petit_trad` can optionally load a local glossary, retrieve a short deterministic
set of term candidates with local embeddings plus an in-process HNSW index, and inject those terms
into the existing TranslateGemma prompt so terminology stays more consistent. The visible proof is
not only new runtime knobs in config, env, and CLI, but also a deterministic non-model test suite
that proves glossary parsing, ranking, prompt formatting, and startup failure behavior before any
human depends on a local GGUF model.

This plan uses strict test-driven development. Every milestone starts by adding failing tests that
describe the intended behavior. Implementation is complete for a milestone only when the new tests
pass, the old tests still pass, and the milestone pass criteria below are satisfied.

## Progress

- [x] (2026-03-24 22:14Z) Reviewed `ARCHITECTURE.md`, `docs/PLANS.md`, `docs/BUILD.md`, the glossary
      research note, the persistent glossary design docs, and the current `petit-core` / `petit-tui`
      code to draft a current-state plan.
- [x] (2026-03-24 22:29Z) Drafted the Milestone 1 red tests and compile seams for glossary config,
      glossary prompt formatting, glossary precedence/default-cache behavior, and CLI parsing;
      intentionally stopping before implementation.
- [x] (2026-03-24 22:30Z) Verified the red phase with targeted tests: the glossary prompt, glossary
      precedence, platform cache fallback, and conflicting CLI flag tests fail as intended; the TOML
      coverage and glossary CLI usage seam compile.
- [x] (2026-03-24 22:42Z) Corrected the Milestone 1 red-test harness: fixed the precedence assertion
      direction and added guard-based env/cwd cleanup so later red runs stay isolated.
- [x] (2026-03-24 22:55Z) Implemented Milestone 1 behavior: glossary CLI/env/file precedence now
      flows into `petit-tui`, the embedding cache default resolves through the platform cache
      directory, conflicting glossary CLI flags are rejected, and glossary prompt formatting now
      emits the compact block.
- [x] (2026-03-24 23:56Z) Verified Milestone 1 green with `./scripts/check.sh --fix` after the
      implementation pass; all Milestone 1 tests now pass.
- [x] Add glossary configuration types, glossary CLI/env/file precedence, and prompt-format test
      seams under failing tests first.
- [x] (2026-03-24 23:58Z) Drafted the Milestone 2 red tests and compile-only scaffolding for TSV
      parsing, pair partitioning, ranking, prompt projection, and glossary error paths; stopping
      before any retrieval or embedding implementation.
- [x] (2026-03-25 00:04Z) Verified the Milestone 2 red state with
      `cargo test -p petit-core     --features cpu-only glossary`; the glossary tests fail only at
      the intentional placeholder seams in `crates/petit-core/src/glossary.rs`.
- [x] (2026-03-25 00:16Z) Corrected the Milestone 2 red-test harness so selection failures stay
      scenario-specific instead of collapsing into a shared empty-result panic.
- [x] (2026-03-25 00:22Z) Tightened the Milestone 2 red-test expectations so parser and config cases
      remain red on contract mismatch instead of accidentally matching the scaffolded placeholder
      errors.
- [x] (2026-03-25 00:31Z) Corrected the Milestone 2 red harness again so the header and error-path
      tests assert the intended glossary error categories while the selection and dedupe cases stay
      red.
- [x] (2026-03-25 00:41Z) Drafted the Milestone 3 red-test harness for prompt-path and worker
      startup coverage, intentionally stopping before any translator wiring or glossary behavior
      implementation.
- [x] (2026-03-25 00:45Z) Verified the Milestone 3 red state under both feature sets: the new
      prompt-path test fails on the missing glossary prompt wiring, and the worker-startup test
      fails on missing-model masking instead of glossary failure precedence.
- [x] (2026-03-25 01:20Z) Corrected the Milestone 3 test harness so the prompt-path coverage uses a
      production-shared prompt helper with an injected glossary lookup seam, keeping the lookup
      calls red until the translator wiring exists.
- [x] (2026-03-25 01:45Z) Verified the restored Milestone 3 red state under both feature sets: the
      core prompt-path tests now fail because the injected glossary lookup is never called, and the
      worker-startup test still fails because missing-model loading masks glossary failure
      precedence.
- [x] (2026-03-25 02:15Z) Implemented Milestone 3: `GemmaTranslator` now initializes glossary
      support before model loading, consults `GlossaryStore` candidates during prompt construction,
      and keeps non-glossary translation behavior unchanged.
- [x] (2026-03-25 02:18Z) Verified Milestone 3 under both feature sets and repo-wide checks:
      targeted `gemma` and worker-startup tests pass in `cpu-only` and `metal`, and both
      `./scripts/check.sh --fix` and `./scripts/check.sh --fix --features metal` pass.
- [x] Implement the `petit-core` glossary subsystem with deterministic parsing, embedding seams,
      HNSW retrieval, and candidate ranking under failing tests first.
- [x] Wire `GemmaTranslator` and `petit-tui` startup paths to the glossary subsystem, then finish
      verification and doc alignment under failing tests first.
- [x] (2026-03-25 00:30Z) Implemented the Milestone 2 `petit-core` glossary subsystem: TSV parsing
      now uses `csv`, rows are normalized and deduplicated deterministically, the store builds
      per-pair HNSW indices through a narrow embedding-provider seam, and candidate selection now
      performs exact-match promotion plus stable ANN ranking.
- [x] (2026-03-25 00:30Z) Verified Milestone 2 under both required feature sets with
      `cargo test -p petit-core --features cpu-only glossary`,
      `cargo test -p petit-core --features metal glossary`, `./scripts/check.sh --fix`, and
      `./scripts/check.sh --fix --features metal`.
- [x] (2026-03-25 04:18Z) Review follow-up fixed: the default real-translation harnesses now clear
      `PETIT_TRAD_GLOSSARY_*`, use `--no-glossary`, and keep the model-backed smoke run isolated
      with `--no-config`; both `./scripts/check.sh --fix` and
      `SMOKE_MODEL_PATH=... ./scripts/smoke.sh` pass after the fix.

## Surprises & Discoveries

- Observation: the durable docs are ahead of the code. `ARCHITECTURE.md`,
  `docs/design-docs/glossary-constraints.md`, and `docs/design-docs/prompt-format.md` already
  describe glossary retrieval, but `crates/petit-core/src/lib.rs` still exports only config, error,
  gemma, language, and model manager, and there is no `glossary.rs` module yet. Evidence: the
  current `GemmaTranslator` only builds the plain `[src->tgt] text` prompt.
- Observation: the platform cache default for embedding artifacts cannot live entirely inside
  `config/default.toml`, because that file is static and the only existing platform-directory
  dependency is `directories` in `crates/petit-tui`. Evidence: `config/default.toml` is plain TOML
  while `crates/petit-tui/src/config.rs` already performs runtime path resolution for user config.
- Observation: if `GemmaTranslator::new` continues to load the GGUF model before validating glossary
  startup, bad glossary config will be masked by `Model file not found` in tests and in model-free
  environments. Evidence: `crates/petit-tui/src/main.rs` currently exercises startup failure only
  through missing-model tests.
- Observation: the Milestone 3 implementation stayed model-free. Evidence: the core prompt-path
  tests exercise the real glossary lookup path with injected candidates, and the worker-startup test
  still proves glossary failure precedence using a missing glossary file plus a fake model path.
- Observation: after implementation, the Milestone 3 gates pass in both `cpu-only` and `metal`
  builds, and the repo-wide check scripts pass as well.

## Decision Log

- Decision: structure the work as TDD milestones with explicit red, green, refactor, pass criteria,
  and named test targets. Rationale: the user asked for later execution to be reviewed against
  concrete tests, and the glossary design already requires deterministic non-model verification.
  Date/Author: 2026-03-24 / Codex.
- Decision: keep glossary logic entirely inside `petit-core`, with `petit-tui` limited to precedence
  handling, user-facing flags, and startup-status surfacing. Rationale: this matches the stable
  architecture boundary and keeps stdin mode, benchmark mode, and future frontends on one retrieval
  path. Date/Author: 2026-03-24 / Codex.
- Decision: validate glossary configuration before GGUF model loading, and fully initialize the
  glossary store before the translator reports readiness. Rationale: this makes startup failures
  observable without a local model and preserves the product requirement that invalid glossary
  configuration must fail fast instead of silently degrading. Date/Author: 2026-03-24 / Codex.
- Decision: make the embedding model fixed in code (`EmbeddingGemma300M`) and keep ANN threshold and
  search breadth as internal constants rather than user-facing knobs in v1. Rationale: this
  preserves deterministic behavior and matches the durable glossary design docs. Date/Author:
  2026-03-24 / Codex.
- Decision: stop after drafting the Milestone 1 red tests and only run targeted verification before
  handing the work to the next implementation subagent. Rationale: the user explicitly requested a
  red-phase handoff for review before any feature implementation begins. Date/Author: 2026-03-24 /
  Codex.
- Decision: keep the `CliArgs::parse_from` seam and the glossary config test-only env lock so the
  Milestone 1 tests can be invoked deterministically without depending on process-global argv state.
  Rationale: the red-phase tests need a stable harness while the real parsing behavior remains
  incomplete. Date/Author: 2026-03-24 / Codex.
- Decision: use RAII guards for env var and cwd restoration in the config tests. Rationale: the
  tests are intentionally expected to fail during the red phase, so cleanup must not depend on code
  below an assertion executing. Date/Author: 2026-03-24 / Codex.
- Decision: resolve the glossary embedding cache directory at `petit-tui` config-load time using the
  platform cache directory when no explicit value is provided. Rationale: `config/default.toml`
  cannot encode a machine-specific cache location, but the milestone still needs a deterministic
  runtime default. Date/Author: 2026-03-24 / Codex.
- Decision: reject simultaneous `--glossary` and `--no-glossary` flags during CLI parsing instead of
  silently taking the last one. Rationale: the red test explicitly codifies the expected failure
  mode and avoids ambiguous command-line behavior. Date/Author: 2026-03-24 / Codex.
- Decision: add a private glossary test seam with a stub embedding provider and placeholder store
  methods before implementation. Rationale: the Milestone 2 tests need deterministic red coverage
  for parser, ranking, and error behavior without introducing the real embedding/HNSW stack yet.
  Date/Author: 2026-03-24 / Codex.
- Decision: keep the Milestone 2 red tests inside `crates/petit-core/src/glossary.rs` so the future
  implementation can share private fixtures and test-only helpers without exporting extra API
  surface. Rationale: the milestone is about behavior coverage, not widening the public crate
  interface before the subsystem exists. Date/Author: 2026-03-25 / Codex.
- Decision: tighten the Milestone 2 red harness after the first verification pass so exact-match
  promotion fails with an explicit empty-shortlist assertion instead of an index panic. Rationale:
  the red phase should isolate missing behavior per scenario and remain reviewable. Date/Author:
  2026-03-25 / Codex.
- Decision: have `build_store_from_rows` preflight the query-embedding seam during store
  construction. Rationale: the red harness requires a deterministic `GlossaryEmbeddingGenerate`
  failure path during initialization, and this keeps the testable initialization contract explicit
  without exposing a second public startup API. Date/Author: 2026-03-25 / Codex.
- Decision: treat `--features metal` as a required verification path alongside the default check
  script for glossary work. Rationale: the user explicitly required both feature sets to stay green,
  and the core crate now builds cleanly in both configurations. Date/Author: 2026-03-25 / Codex.
- Decision: keep the Milestone 3 harness model-free by using a production-shared prompt helper and
  an injected glossary lookup seam, instead of requiring a local GGUF or embedding asset. Rationale:
  the user asked to stop only if real assets were actually required, and the red phase can still
  prove the intended startup ordering without downloads. Date/Author: 2026-03-25 / Codex.
- Decision: initialize glossary support before model loading in `GemmaTranslator::new` and thread
  glossary candidates through prompt construction directly from `GlossaryStore`. Rationale: this
  preserves the required startup precedence while keeping prompt assembly deterministic and
  non-glossary behavior unchanged. Date/Author: 2026-03-25 / Codex.

## Outcomes & Retrospective

Milestone 1 is complete. The repository now resolves glossary configuration through the existing
precedence chain, computes a platform cache default at config-load time, rejects contradictory CLI
glossary flags, and preserves the compact glossary prompt format when glossary terms are present.
Milestone 2 is complete as well. `petit-core` now owns glossary TSV parsing, normalization,
deduplication, deterministic HNSW-backed candidate selection, and the glossary-specific error paths.
Milestone 3 is now complete too: `GemmaTranslator` initializes glossary support before model
loading, translates through the real glossary candidate path, and `petit-tui` reports glossary
startup failures before missing-model failures. The repository passed both
`./scripts/check.sh --fix` and `./scripts/check.sh --fix --features metal` after the implementation
pass. The review follow-up also landed before closeout: the default runtime harnesses are now
isolated from caller glossary config and environment, while a later tech-debt item tracks adding
dedicated glossary-specific harness regression coverage.

## Context and Orientation

`petit_trad` currently has two Rust crates. `crates/petit-core` owns translation logic, language
validation, prompt construction, and llama.cpp inference. `crates/petit-tui` owns CLI parsing,
config precedence, TUI orchestration, stdin mode, benchmark mode, and worker-thread startup. The
top-level architecture in `ARCHITECTURE.md` and the deeper design docs in
`docs/design-docs/glossary-constraints.md`, `docs/design-docs/prompt-format.md`, and
`docs/design-docs/architecture.md` already define the target glossary feature: local glossary TSV
input, `fastembed` embeddings using `EmbeddingGemma300M`, one HNSW index per normalized language
pair, exact-match promotion ahead of ANN-only hits, and a compact glossary block injected into the
existing direct TranslateGemma prompt.

The code has not caught up to that design yet. `crates/petit-core/src/config.rs` only carries model
and logging settings. `crates/petit-core/src/error.rs` only knows generic config, model, inference,
and unsupported-language errors. `crates/petit-core/src/gemma.rs` has no glossary state and always
builds the plain one-line prompt. `crates/petit-tui/src/config.rs` only merges model, translation,
and UI settings. `crates/petit-tui/src/cli.rs` has no glossary flags. `config/default.toml` has no
`[glossary]` section. There are no glossary-specific dependencies in the workspace manifests, no
glossary fixtures, and no deterministic glossary tests.

Three existing pieces matter for implementation. First, `crates/petit-core/src/language.rs` already
defines `normalize_lang`, `is_supported`, and `validate_pair`; glossary pair partitioning must reuse
those rules so one request path does not invent its own language normalization. Second,
`crates/petit-tui/src/main.rs` already has a worker startup status path that reports
`TranslatorInitializing`, `TranslatorReady`, and `TranslatorInitFailed`; glossary startup failures
must flow through that same path. Third, `docs/BUILD.md` declares `./scripts/check.sh --fix` as the
canonical verification command, and that command is required before handing implementation over for
review.

In this plan, “exact match” means a normalized source term appears as a normalized substring within
the normalized request text. “ANN” means approximate nearest-neighbor retrieval through HNSW. “TDD”
means each feature slice starts by adding a failing automated test that names the required behavior,
then implementing just enough code to pass it, then refactoring without changing that behavior.

## Plan of Work

The work is split into three milestones so each one produces a usable, independently verifiable
slice of the final feature. The first milestone creates configuration and prompt-construction seams
without touching live embedding code yet. The second milestone builds the glossary subsystem in
`petit-core` behind a deterministic, testable interface that can run entirely with stub embeddings
inside unit tests. The third milestone wires translator startup and the frontend surfaces together,
adds the remaining failure-path tests, and updates durable docs only where implementation details
must be made explicit.

### Milestone 1: configuration and prompt seams under tests

Start by extending the configuration shape, but do it under tests first. In `crates/petit-core`, add
a `GlossaryConfig` type and nest it into `Config`. Keep runtime fields concrete rather than optional
so the translation path never has to guess whether glossary defaults were resolved. The new shape
should be:

    pub struct Config {
        pub model_path: PathBuf,
        pub gpu_layers: u32,
        pub context_size: u32,
        pub threads: u32,
        pub log_to_file: bool,
        pub log_path: PathBuf,
        pub glossary: GlossaryConfig,
    }

    pub struct GlossaryConfig {
        pub enabled: bool,
        pub path: PathBuf,
        pub embedding_cache_dir: PathBuf,
        pub max_matches: usize,
    }

Do not put platform-cache discovery into `petit-core`. `crates/petit-tui/src/config.rs` already owns
precedence and path resolution, so it should be the place that computes the default embedding cache
directory using `directories::ProjectDirs`. `config/default.toml` should gain a disabled
`[glossary]` block with a stable example path such as `config/glossary.tsv` and a default
`max_matches = 6`. If the file omits `embedding_cache_dir`, `petit-tui` must synthesize the platform
cache path before constructing `petit_core::Config`.

The prompt builder must stop being an untestable implicit detail. Extract the direct-format prompt
construction in `crates/petit-core/src/gemma.rs` into a pure helper that accepts a glossary
shortlist. Keep the existing no-glossary prompt byte-for-byte identical. When glossary candidates
exist, emit exactly the compact block already described in `docs/design-docs/prompt-format.md`, with
each line formatted as `- source_term -> target_term`, a blank line before `Text:`, and no extra
prose in the model turn.

The TDD work for this milestone is not optional. Write the tests first, watch them fail, and only
then change code:

- In `crates/petit-core/src/config.rs`, add serialization and TOML parsing tests for a fully
  populated glossary section and a disabled glossary section.
- In `crates/petit-core/src/gemma.rs`, add prompt-format tests that prove the legacy prompt stays
  unchanged when there are no glossary candidates and that glossary candidates render in the exact
  multi-line format required by the durable docs.
- In `crates/petit-tui/src/config.rs`, add precedence tests proving glossary values resolve in the
  order `CLI > env > config file > defaults`, including a test that the embedding cache directory
  falls back to a platform path when not supplied in file or env settings.
- In `crates/petit-tui/src/cli.rs`, add parsing tests for new glossary flags and for invalid flag
  combinations such as `--glossary` with `--no-glossary`.

Add the minimum new CLI and env surfaces needed to satisfy the product requirement that glossary
behavior follows the same deterministic precedence rules as the rest of the app. Use these names so
the docs and implementation stay stable:

- CLI flags: `--glossary`, `--no-glossary`, `--glossary-path <path>`,
  `--glossary-embedding-cache-dir <path>`, `--glossary-max-matches <n>`.
- Environment variables: `PETIT_TRAD_GLOSSARY_ENABLED`, `PETIT_TRAD_GLOSSARY_PATH`,
  `PETIT_TRAD_GLOSSARY_EMBEDDING_CACHE_DIR`, `PETIT_TRAD_GLOSSARY_MAX_MATCHES`.

Pass criteria for Milestone 1:

- The new glossary flags appear in `CliArgs::usage()`.
- `AppConfig::load_config` can construct a complete `petit_core::Config` with glossary settings even
  when the embedding cache directory was omitted from TOML.
- The no-glossary prompt string is unchanged from current behavior.
- The glossary prompt string matches the design doc exactly, including line breaks.
- All new tests for config and prompt behavior pass before any embedding code is introduced.

### Milestone 2: deterministic glossary subsystem in `petit-core`

Create `crates/petit-core/src/glossary.rs` and expose it from `crates/petit-core/src/lib.rs`. This
module owns glossary parsing, normalization, deduplication, embedding, HNSW indexing, candidate
selection, and prompt-ready term formatting. Keep this subsystem self-contained so no frontend code
needs to know how embeddings or HNSW work.

Use `fastembed` and `hnsw_rs` in production, but add one narrow test seam so unit tests do not need
network access or real embedding downloads. Introduce a private `EmbeddingProvider` trait inside
`glossary.rs` with one method for batch passage embeddings and one method for a query embedding.
Production code uses a `FastEmbedProvider` pinned to `EmbeddingGemma300M`. Tests use a deterministic
stub provider that returns hard-coded vectors. Use the real `hnsw_rs` index in tests with those stub
vectors so the test suite still exercises the HNSW path.

Use the `csv` crate in TSV mode instead of ad hoc `split('\t')` parsing. The glossary file should
require headers `source_lang`, `target_lang`, `source_term`, and `target_term`, and accept an
optional `note` column. Validate empty required fields. Normalize language codes through
`crate::language::normalize_lang`. Normalize source terms for deduplication and exact matching by
trimming, lowercasing, and collapsing internal whitespace to a single ASCII space. Deduplicate file
rows by normalized language pair plus normalized source term plus target term, keeping the first
encountered row so output order is stable.

Model the runtime store with explicit per-pair partitions. One reasonable shape is:

    pub struct GlossaryStore {
        pair_indices: HashMap<LangPairKey, PairGlossaryIndex>,
        max_matches: usize,
    }

    struct PairGlossaryIndex {
        entries: Vec<GlossaryEntry>,
        source_embeddings: Vec<Vec<f32>>,
        hnsw: Hnsw<'static, f32, DistCosine>,
    }

    pub struct GlossaryCandidate {
        pub source_term: String,
        pub target_term: String,
    }

`LangPairKey` can be a small private struct or a `(String, String)` tuple alias. Keep the HNSW
partition private so only `GlossaryStore::select_candidates` is exposed.

Retrieval must be deterministic. For one translation request, `GlossaryStore::select_candidates`
should:

1. Normalize the request language pair and request text.
2. Return an empty list immediately when glossary support is disabled or the pair has no entries.
3. Create a query embedding from the full input text.
4. Query the HNSW index with fixed internal constants for `k` and `ef`.
5. Compute exact normalized substring matches over the same entry set.
6. Rank exact matches first, sorted by descending normalized source-term length and then by source
   term and target term for stable ties.
7. Rank ANN-only matches next by descending cosine similarity, then by source term and target term
   for stable ties.
8. Drop ANN-only matches below one fixed internal similarity threshold.
9. Deduplicate the final shortlist by normalized source term, keeping the first ranked candidate.
10. Truncate to `max_matches`.

Keep the `note` field in memory for future work but do not inject it into the prompt in v1.

Write the following tests before implementing the subsystem:

- TSV parser tests for missing headers, empty required fields, optional `note`, and duplicate-row
  collapse.
- Pair partitioning tests proving `en->fr` entries do not leak into `en->de`.
- Exact-match promotion tests proving an exact substring outranks a semantically close ANN-only
  candidate.
- Deterministic ordering tests proving tie behavior is stable and `max_matches` truncation is
  repeatable.
- Prompt-candidate projection tests proving the public shortlist exposes only the source and target
  terms in the expected order.
- Error tests for glossary file read failures, parse failures, embedding initialization failures,
  embedding generation failures, and index-build failures.

Pass criteria for Milestone 2:

- `petit-core` builds with new dependencies declared in `Cargo.toml` and
  `crates/petit-core/Cargo.toml`.
- All glossary parsing and ranking tests pass in a model-free environment.
- The glossary subsystem can be instantiated entirely through tests with stub embeddings.
- No TUI code imports `fastembed` or `hnsw_rs`; those crates remain confined to `petit-core`.

### Milestone 3: translator wiring, startup behavior, and final verification

Wire the glossary store into `GemmaTranslator`. The translator should hold `Option<GlossaryStore>`
alongside `ModelManager`. `GemmaTranslator::new(config)` must validate and initialize glossary
support before declaring readiness. That means validating glossary configuration and loading the
glossary store before model-backed translation requests can begin. Keep `with_model_manager` useful
for tests by either adding a second constructor that accepts an optional `GlossaryStore` or by
replacing it with a helper that constructs a translator from already-initialized parts.

Update `translate()` so it asks the glossary store for candidates only after `validate_pair`
succeeds and before calling inference. Then pass those candidates into the new pure prompt builder.
If no candidates survive selection, use the unchanged plain prompt path. Add new glossary-specific
errors to `crates/petit-core/src/error.rs` with messages that are explicit enough to surface in the
TUI worker footer and in stdin mode.

`crates/petit-tui/src/main.rs` already contains a startup failure test for a missing model. Extend
that coverage so glossary failures are observable even when no real GGUF file is present. The
cleanest path is to ensure glossary validation runs before model loading and then add a worker test
that passes a missing glossary file plus a fake model path and asserts that the emitted failure
mentions the glossary problem rather than the missing model.

Only after the code behavior is stable should durable docs be touched. Update the persistent docs
that need implementation-level precision, not every document that mentions glossary support. At
minimum, verify and adjust:

- `docs/design-docs/glossary-constraints.md` if any field defaults or flag names differ from the
  current design text.
- `docs/design-docs/prompt-format.md` if the tested prompt string differs in whitespace or wording.
- `docs/product-specs/requirements.md` if the accepted glossary surfaces differ from the stated
  configuration requirements.
- `docs/BUILD.md` if there are new glossary-specific verification commands or operator caveats worth
  documenting.

Write the remaining failing tests before implementation:

- A `GemmaTranslator` prompt-path test proving glossary candidates are consulted only after
  language-pair validation and that the prompt falls back to the plain format when no candidates are
  returned.
- A worker startup test in `crates/petit-tui/src/main.rs` proving invalid glossary config fails
  startup before model loading masks it.
- A config-to-core integration test proving the default platform cache path is threaded into
  `petit_core::Config`.
- If needed, a CLI help snapshot-style test proving the new glossary flags remain documented.

Pass criteria for Milestone 3:

- Enabling glossary support with an invalid glossary path fails startup with a glossary-specific
  error message.
- Existing non-glossary translation behavior stays unchanged when glossary support is disabled.
- `./scripts/check.sh --fix` passes from the repository root.
- Durable docs reflect the implementation rather than the earlier conceptual shorthand.

## Concrete Steps

Run the following commands from `/Users/dzr/src/repo/petit_trad` while implementing. After each
red-green-refactor cycle, update `Progress`, `Decision Log`, and `Outcomes & Retrospective`.

Start with targeted failing tests for Milestone 1:

    cargo test -p petit-core --features cpu-only
    cargo test -p petit-tui --features cpu-only

Add only the Milestone 1 tests, rerun them, and confirm they fail for the expected reasons. Then
implement the smallest code changes needed to make those tests pass.

After Milestone 1 is green, repeat the same pattern for the glossary subsystem in Milestone 2:

    cargo test -p petit-core --features cpu-only glossary
    cargo test -p petit-core --features cpu-only

The `glossary` filter assumes the new tests live in a module or test names containing `glossary`.
Keep those names explicit so later contributors can run just this slice while iterating.

After Milestone 3 wiring is complete, run the full repo verification command required by
`docs/BUILD.md`:

    ./scripts/check.sh --fix

Then run one model-free startup proof that demonstrates glossary failure precedence:

    printf 'Hello\n' | cargo run -p petit-tui --features cpu-only -- \
      --no-config \
      --model /tmp/petit-missing-model.gguf \
      --source-lang en \
      --target-lang fr \
      --stdin \
      --glossary \
      --glossary-path /tmp/petit-missing-glossary.tsv

The expected stderr must mention a glossary read or glossary config failure. It must not stop first
at `Model file not found`.

If a local GGUF model is available, run one optional end-to-end sanity check with a tiny glossary
fixture after all unit tests are green:

    cargo run -p petit-tui --features cpu-only -- \
      --stdin \
      --model /absolute/path/to/model.gguf \
      --source-lang en \
      --target-lang fr \
      --glossary \
      --glossary-path /absolute/path/to/glossary.tsv < /absolute/path/to/input.txt

Use this only as a final confidence check. The milestone gates remain the deterministic automated
tests plus `./scripts/check.sh --fix`.

## Validation and Acceptance

Implementation is accepted only if all of the following are true:

- The new glossary unit and integration tests were added before the corresponding production code,
  failed first, and are recorded in the plan as the red step for each milestone.
- `cargo test -p petit-core --features cpu-only` passes with glossary parsing, ranking, and prompt
  tests included.
- `cargo test -p petit-tui --features cpu-only` passes with glossary precedence and worker startup
  tests included.
- `./scripts/check.sh --fix` passes from the repository root.
- Running `cargo run -p petit-tui -- --help` shows the glossary CLI flags.
- The model-free startup proof reports glossary failure before missing-model failure.
- Disabling glossary support preserves the old prompt format and does not require a glossary file or
  embedding cache.

The feature should be considered incomplete if code exists without the matching tests, if the tests
only exercise helper functions and never cover startup behavior, or if the final verification relies
on having a local model instead of passing in a model-free environment.

## Idempotence and Recovery

All changes in this plan are additive and safe to rerun. The glossary tests should use temporary
files and stub embeddings so repeated test runs do not depend on persistent machine state. If
first-run embedding downloads are triggered during manual experimentation, they should be directed
to the configured embedding cache directory so retrying does not corrupt repository files.

If a milestone stalls, leave the worktree in one of two states only: either the new failing tests
exist and are documented in `Progress` as intentionally red, or the implementation plus tests are
green for that milestone. Do not leave half-renamed interfaces or undocumented threshold changes.

## Artifacts and Notes

Important artifacts to capture during execution:

- A short excerpt of the new glossary prompt-format tests showing the exact expected prompt string.
- A short excerpt of the glossary ranking test proving exact-match promotion.
- The stderr line from the model-free startup proof showing glossary failure precedence.
- The final `./scripts/check.sh --fix` success summary.

If implementation reveals that `fastembed` or `hnsw_rs` require a different concrete type than the
interface prescribed above, update this section and the `Decision Log` immediately with the exact
reason.

## Interfaces and Dependencies

Add these workspace dependencies in `Cargo.toml`, then wire them into `crates/petit-core` only:

- `fastembed` for text embeddings.
- `hnsw_rs` for the in-process HNSW index.
- `csv` for deterministic TSV parsing.

At the end of implementation, these interfaces should exist in approximately this form:

In `crates/petit-core/src/config.rs`, define:

    pub struct GlossaryConfig {
        pub enabled: bool,
        pub path: PathBuf,
        pub embedding_cache_dir: PathBuf,
        pub max_matches: usize,
    }

In `crates/petit-core/src/glossary.rs`, define:

    pub struct GlossaryCandidate {
        pub source_term: String,
        pub target_term: String,
    }

    pub struct GlossaryStore { ... }

    impl GlossaryStore {
        pub fn from_config(config: &GlossaryConfig) -> crate::Result<Self>;
        pub fn select_candidates(
            &self,
            source_lang: &str,
            target_lang: &str,
            source_text: &str,
        ) -> crate::Result<Vec<GlossaryCandidate>>;
    }

In `crates/petit-core/src/gemma.rs`, keep or introduce a pure helper with a shape equivalent to:

    fn build_prompt(
        text: &str,
        source_lang: &str,
        target_lang: &str,
        glossary_candidates: &[GlossaryCandidate],
    ) -> String

In `crates/petit-tui/src/config.rs`, extend `FileConfig`, env parsing, and CLI application so
`load_config` always constructs a complete `petit_core::Config` with resolved glossary settings.

In `crates/petit-core/src/error.rs`, add glossary-specific error variants rather than squeezing all
failures into the generic `Config` error.

## Change Note

Initial draft created on 2026-03-24 to turn the glossary HNSW research note plus the newer durable
docs into an implementation-ready ExecPlan grounded in the current codebase and explicit TDD
acceptance gates.
