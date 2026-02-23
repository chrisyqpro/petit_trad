# Phase 5: Polish, Performance, and UX Feedback

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this work, users can see clearer status and error feedback in the TUI and maintainers can measure translation
performance in a repeatable way. The visible outcome is more predictable first-use behavior, better runtime failure
messages, and documented latency expectations for short and medium inputs.

This plan restarts Phase 5 from the beginning. A previous Phase 5 plan file was intentionally discarded after reading,
and only its `Purpose / Big Picture` and `Progress` sections were used to reconstruct requirements.

## Progress

- [x] (2026-02-23 12:00Z) Reconstructed Phase 5 requirements from discarded plan `Purpose` and `Progress` sections.
- [x] (2026-02-23 12:10Z) Wrote reset research artifact:
      `docs/execution-plans/research/2026-02-23-phase-5-polish-performance-ux-reset.md`.
- [x] (2026-02-23 12:20Z) Authored fresh Phase 5 ExecPlan with concrete milestones, file targets, and validation
      commands.
- [x] (2026-02-23 12:45Z) Defined and documented a benchmark matrix (short/medium inputs, cold/warm interpretation) in
      `docs/BUILD.md` and this plan's `Artifacts and Notes`.
- [x] (2026-02-23 12:35Z) Extended `petit --benchmark` mode with startup timing, warmup runs, measured runs, summary
      stats, and `--max-new-tokens`.
- [x] (2026-02-23 13:25Z) Gathered and recorded baseline latency/startup measurements for short and medium inputs on
      CPU-only and Metal (27B local model), with successful translation outputs.
- [x] (2026-02-23 12:40Z) Implemented focused optimization: pre-initialize the TUI translator worker and surface
      readiness/init-failure events.
- [x] (2026-02-23 12:40Z) Improved TUI status and error messaging with typed status state and severity-aware footer
      rendering.
- [x] (2026-02-23 12:42Z) Added tests covering status/error state transitions and worker initialization failure
      signaling.
- [x] (2026-02-23 12:47Z) Updated `README.md` and `docs/BUILD.md` with the benchmark workflow and hardware variability
      guidance.
- [x] (2026-02-23 13:35Z) Re-ran `./scripts/check.sh --fix` after migrating benchmarking into `petit --benchmark` and
      recorded the final output summary in `Artifacts and Notes`.

## Surprises & Discoveries

- Observation: The first translation request in the TUI includes lazy translator/model initialization because the worker
  creates `GemmaTranslator` on first request, not at worker startup. Evidence: `crates/petit-tui/src/main.rs`
  initializes `translator` as `None` in `start_translation_worker` and creates it inside the request loop.

- Observation: At planning time, there was no repeatable benchmark matrix or structured benchmark output in the repo;
  only a simple benchmark example printed one total elapsed time. Evidence: the original
  `crates/petit-core/examples/translate.rs` printed a single `Elapsed:` value for one translation run before the
  `petit --benchmark` mode replaced it.

- Observation: TUI status feedback is currently an untyped `Option<String>`, which prevents severity-aware styling and
  durable handling of errors. Evidence: `crates/petit-tui/src/app.rs` stores `status_message` and
  `crates/petit-tui/src/ui.rs` renders all messages with the same style.

- Observation: The local workspace contains a 27G `translategemma-27b` GGUF file, while the default config path targets
  a missing 12B model file. Evidence: `models/translategemma-27b-it-GGUF/translategemma-27b-it.Q8_0.gguf` exists;
  `models/translategemma-12b-it-GGUF/` does not.

- Observation: A bounded CPU-only benchmark attempt against the available 27B model did not complete within 90 seconds,
  even with `--max-new-tokens 4`. Evidence: The timed command in `Artifacts and Notes` exited with Python
  `subprocess.TimeoutExpired` after 90 seconds.

- Observation: Benchmark mode initially reused `AppConfig.stdin_mode`, so non-TTY execution could be treated as implicit
  stdin input and fail with `benchmark input is empty` when `--text` was omitted. Evidence: a `petit --benchmark` smoke
  run without `--text` failed until benchmark input reading was changed to require explicit `--stdin`.

## Decision Log

- Decision: Rebuild Phase 5 as a new ExecPlan instead of editing the discarded file, while preserving only the old
  plan's purpose and progress intent. Rationale: The user explicitly requested a workflow restart from the beginning and
  disposal of the original plan file. Date/Author: 2026-02-23 / codex.

- Decision: Treat cold-start latency and warm translation latency as separate measurements and acceptance evidence.
  Rationale: Current TUI behavior hides model initialization inside the first translation request, so a single latency
  number is misleading. Date/Author: 2026-02-23 / codex.

- Decision: Introduce typed status/error state in `petit-tui` app state instead of continuing to pass arbitrary strings
  directly to the footer renderer. Rationale: Typed state enables clearer messages, styling, and test coverage without
  brittle string matching in UI logic. Date/Author: 2026-02-23 / codex.

- Decision: Use TUI worker pre-initialization as the focused Phase 5 optimization, and treat improved first-use
  predictability as the primary user-visible gain. Rationale: It is low-risk, directly addresses hidden cold-start
  latency in the first request path, and pairs naturally with clearer startup status messaging. Date/Author: 2026-02-23
  / codex.

- Decision: Start with the benchmark example for fast implementation, then migrate the final benchmark interface into
  `petit --benchmark`. Rationale: The example provided a quick path to prove output shape, and moving the final
  interface into `petit` removed the extra execution layer and enabled normal config/env/CLI precedence. Date/Author:
  2026-02-23 / codex.

- Decision: Record benchmark collection as blocked after a bounded timeout rather than inventing latency numbers or
  committing machine-specific guesses. Rationale: The available local model is a 27B Q8 GGUF and the bounded CPU-only
  probe did not finish quickly enough to produce reliable short/medium matrix measurements in this execution session.
  Date/Author: 2026-02-23 / codex.

- Decision: Preserve `--src`/`--tgt` aliases in `petit` as benchmark-friendly aliases for `--source-lang` and
  `--target-lang`, and require explicit `--stdin` for benchmark stdin input.
  Rationale: This keeps older example-style benchmark commands working while avoiding accidental empty-stdin failures in
  non-TTY runs. Date/Author: 2026-02-23 / codex.

## Outcomes & Retrospective

Implemented:

- TUI worker startup now pre-initializes the translator and emits explicit worker lifecycle events (`initializing`,
  `ready`, `init failed`) to the app.
- TUI footer messaging now uses typed status state with severity-aware colors and a separate spinner path for background
  translator initialization.
- `petit --benchmark` mode now supports repeatable startup/warm-run timing output with warmup and measured run counts.
- Docs now describe the benchmark workflow and how to interpret startup versus warm translation latency.
- Tests cover status/error transitions and worker initialization failure.

Baseline evidence captured:

- Short and medium inputs were benchmarked successfully on CPU-only and Metal for the local 27B model, with correct
  translation outputs recorded in `Artifacts and Notes`.

Remaining optional work:

- Additional benchmark variants (different threads, token caps, or models) and future before/after comparisons if
  another optimization pass is made.

Lesson learned:

- Phase 5 performance evidence must explicitly account for local model size and runtime feature selection; tooling and
  documentation should be in place before attempting comparisons.

## Context and Orientation

This repository is a Rust workspace with two production crates:

- `crates/petit-core` is the translation backend. It owns model loading, inference, prompt formatting, language
  validation, and core errors.
- `crates/petit-tui` is the terminal UI frontend. It owns input/output state, status messaging, keyboard handling, and
  the background translation worker.

Relevant current files and behavior:

- `petit --benchmark` mode now supports startup timing, warmup runs, repeated measured runs, summary stats, and
  `--max-new-tokens` for repeatable local benchmarking.
- `crates/petit-core/src/model_manager.rs` performs GGUF model loading (`ModelManager::new`) and per-request inference
  (`ModelManager::infer`).
- `crates/petit-core/src/error.rs` defines user-visible core error categories (`Config`, `ModelLoad`, `Inference`,
  `UnsupportedLanguage`).
- `crates/petit-core/src/language.rs` validates supported language codes.
- `crates/petit-tui/src/main.rs` starts a worker thread that now initializes `GemmaTranslator` immediately and emits
  worker lifecycle events to the app.
- `crates/petit-tui/src/app.rs` now stores typed footer state (`StatusLine`, `StatusKind`) and tracks background worker
  initialization state.
- `crates/petit-tui/src/ui.rs` renders status severity (info/success/error) with distinct colors and shows a spinner
  during worker initialization.
- `config/default.toml` defines default runtime knobs (`gpu_layers`, `context_size`, `threads`) that must be recorded in
  benchmark evidence.
- `README.md` and `docs/BUILD.md` are the user-facing docs to update with measured expectations and benchmark commands.

Assumptions for this plan:

- A local GGUF model exists at the configured path when collecting real performance evidence. If no model is available,
  implement the tooling and TUI UX changes first, then mark benchmark evidence as blocked in `Progress`.
- Phase 5 should remain additive and preserve existing CLI/TUI behavior except for clearer messages and more predictable
  first-use feedback.

## Plan of Work

Milestone 1 creates a repeatable benchmark workflow. The final implementation in this branch lives in
`petit --benchmark` mode so it uses normal config precedence. A novice must be able to run a small matrix (short and
medium text, cold and warm runs) and capture consistent output. The first deliverable is not an optimization; it is
measurement and evidence collection capability.

Milestone 2 gathers baseline data and records it in this plan and in user-facing docs. The measurements must explicitly
separate startup/model-load cost from warm translation cost. This milestone also defines the benchmark matrix in plain
language so future contributors can reproduce the same runs.

Milestone 3 implements one focused performance improvement tied to the measured hot path and user experience. The
primary target is TUI first-request predictability by moving translator initialization from "first request" to worker
startup (or another measured low-risk optimization if data clearly shows better payoff). The chosen change must be
logged in `Decision Log` and validated with before/after numbers.

Milestone 4 improves TUI status and error feedback. `petit-tui` app state will use typed status information (for example
info/success/error) so `ui.rs` can render clearer visual feedback and preserve meaningful runtime failures. Common
failures to improve include missing model file, unsupported language codes, worker unavailable, and translator
initialization failure.

Milestone 5 updates tests and docs. Add state-level tests for new status/error behavior, then update `README.md` and
`docs/BUILD.md` with the benchmark workflow, measured expectations, and caveats about hardware-dependent results.

## Concrete Steps

Run all commands from the repository root unless stated otherwise:

    cd /Users/dzr/src/repo/petit_trad

1. Baseline verification before edits:

   cargo test --workspace --features cpu-only

2. Extend `petit --benchmark` mode to support a repeatable benchmark mode with:
   - configurable number of warm runs
   - configurable measured runs
   - explicit startup timing (`GemmaTranslator::new`)
   - per-run translation timing and summary stats (at least average)

   Expected output shape (example; values will differ by hardware):

   Startup: 12.34s Run 1: 1.42s Run 2: 1.31s Run 3: 1.29s Average: 1.34s

3. Define and execute the benchmark matrix (document exact input strings used):
   - short input: one sentence
   - medium input: short paragraph (3-5 sentences)
   - cold measurement: first run after process start (records startup + run)
   - warm measurement: repeated runs in same process after initialization

   Example command shape:

   cargo run -p petit-tui --features cpu-only -- --benchmark \
    --source-lang en --target-lang fr --text "Hello, how are you?" --warmup-runs 1 --runs 3

4. Implement the chosen optimization and update `Progress`, `Decision Log`, and `Artifacts and Notes` with before/after
   evidence. If the TUI worker startup initialization path is selected, edit `crates/petit-tui/src/main.rs` so the
   worker initializes the translator before receiving the first request and emits status-ready / init-failed signals for
   the app.

5. Refactor TUI status handling in `crates/petit-tui/src/app.rs` and `crates/petit-tui/src/ui.rs` to typed status state
   and severity-aware rendering. Update worker response handling in `crates/petit-tui/src/main.rs` to preserve
   actionable error messages.

6. Add or update tests in `crates/petit-tui/src/app.rs` (and `ui.rs` if needed) for status transitions and error
   presentation behavior.

7. Update docs:
   - `docs/BUILD.md`: benchmark workflow and how to interpret startup vs warm latency.
   - `README.md`: concise measured expectations and link/reference to benchmark command(s), with hardware caveat.

8. Final verification and evidence capture:

   ./scripts/check.sh --fix

   Record the final output summary and benchmark evidence in this plan.

## Validation and Acceptance

This phase is complete when all of the following are true:

- A repeatable benchmark workflow exists in-repo and can measure startup and warm translation latency for short and
  medium inputs.
- This plan's `Artifacts and Notes` section contains baseline and post-change measurements with the exact commands used.
- The chosen optimization is documented in `Decision Log` and shows measurable improvement (or a justified tradeoff such
  as improved predictability) against the baseline.
- The TUI displays clearer status/error feedback, with visibly different handling for loading and error states, and
  common failures are understandable without reading source code.
- Status/error state transitions are covered by tests that fail before the new status model and pass after it.
- `README.md` and `docs/BUILD.md` describe how to run the benchmark and set user expectations about hardware
  variability.
- `./scripts/check.sh --fix` passes and its output is recorded in `Artifacts and Notes`.

## Idempotence and Recovery

Benchmark commands must be safe to run repeatedly. Measurements should be recorded with the configuration used (`model`,
`gpu_layers`, `context_size`, `threads`, feature flags) so later reruns can be compared fairly.

If real-model benchmarks are blocked (missing model file or unsupported local hardware backend), continue with code,
tests, and docs updates, then record the blocker in `Progress` and `Artifacts and Notes` instead of inventing numbers.

For TUI message changes, keep the worker protocol additive during the refactor until the app and UI render paths are
updated together, then remove any unused legacy string-only paths in the same milestone.

## Artifacts and Notes

Record evidence here as work proceeds:

- Benchmark matrix definition (short/medium text, cold/warm runs): Short input = one sentence (for example "Hello, how
  are you?"). Medium input = 3-5 sentence paragraph. Cold result = `Startup` + first measured `Run`. Warm result =
  subsequent `Run` values after warmup in the same process.
- Baseline command(s) and output summary: Feasibility / tooling validation (missing model):
  `cargo run -p petit-tui --features cpu-only -- --benchmark --model /no/such/model.gguf --runs 1` Output showed
  benchmark header fields and failed with `Failed to load model: Model file not found: /no/such/model.gguf`. Bounded
  local-model probe: `python3 -c 'import subprocess,sys; subprocess.run(sys.argv[1:], timeout=90, check=False)'`
  `cargo run -p petit-tui --features cpu-only -- --benchmark --model`
  `models/translategemma-27b-it-GGUF/translategemma-27b-it.Q8_0.gguf --gpu-layers`
  `0 --threads 1 --max-new-tokens 4 --warmup-runs 0 --runs 1 --text Hi` Result: timed out after 90 seconds before
  printing `Startup`/`Run 1` in this session. User-verified successful run (same settings) produced `Startup: 69.72s`,
  `Run 1: 10.95s`, `Target: Salut.`, and `Average/Min/Max: 10.95s`. User-verified short Metal run (`--features metal`,
  `--gpu-layers 999`, `threads=4`, `text=Hi`, `max-new-tokens=4`) produced `Startup: 2.40s`, `Run 1: 618.39ms`,
  `Target: Salut.`, and `Average/Min/Max: 618.39ms`. User-verified medium CPU-only run (`--features cpu-only`,
  `--gpu-layers 0`, `--threads 1`, `--text` 117-char paragraph, `--max-new-tokens 64`, `--warmup-runs 1`, `--runs 3`)
  produced `Startup: 65.03s`, `Warmup 1: 29.34s`, `Run 1: 24.60s`, `Run 2: 26.65s`, `Run 3: 24.36s`, `Average:
  25.20s`, `Min: 24.36s`, `Max: 26.65s`, and a correct French translation output. User-verified medium Metal run
  (`--features metal`, `--gpu-layers 999`, `threads=4`, same 117-char paragraph, `--max-new-tokens 64`,
  `--warmup-runs 1`, `--runs 3`) produced `Startup: 8.14s`, `Warmup 1: 3.23s`, `Run 1: 3.20s`, `Run 2: 3.21s`,
  `Run 3: 3.23s`, `Average: 3.21s`, `Min: 3.20s`, `Max: 3.23s`, and a correct French translation output.
- Optimization selected and rationale: Pre-initialize the TUI translator worker at startup and emit readiness/init
  failure events. This surfaces cold-start cost separately from translation and improves first-use predictability
  without changing `petit-core` inference semantics.
- Post-change command(s) and output summary: `cargo test -p petit-tui --features cpu-only` Result: 10 tests passed,
  including typed status transitions and worker init failure signaling. `cargo test --workspace --features cpu-only`
  Result: 34 total tests passed across `petit-core` and `petit-tui`.
- TUI failure scenarios exercised (missing model, unsupported language, worker unavailable) and observed messages:
  Missing model worker startup is covered by `tests::worker_reports_init_failure_when_model_missing` and
  `app::tests::worker_initialization_status_is_typed`, which verify an init failure event and the typed footer error
  prefix `Translator initialization failed: ...`. Unsupported language entry is covered by
  `app::tests::invalid_language_edit_sets_error_status`, which verifies an error status containing
  `Unsupported language`. Worker unavailable is covered by
  `app::tests::worker_unavailable_sets_error_and_clears_loading`, which verifies the footer error
  `Translation worker unavailable`.
- `./scripts/check.sh --fix` final output summary: `fmt`, `clippy`, `check`, and `test` all passed for `cpu-only`; final
  line was `==> all checks passed` (rerun after migrating to `petit --benchmark` and fixing benchmark stdin handling).

## Interfaces and Dependencies

Implemented interface changes (names and responsibilities):

- `crates/petit-tui/src/app.rs`
  - typed status state via `StatusKind` and `StatusLine`
  - helper methods that centralize info/success/error status updates
  - `is_worker_initializing` state for background startup feedback
- `crates/petit-tui/src/ui.rs`
  - render footer style based on typed status severity while preserving spinner behavior during loading
- `crates/petit-tui/src/main.rs`
  - `WorkerEvent` protocol differentiating initializing, ready, init-failed, and translation result messages
  - worker thread pre-initialization of `GemmaTranslator`
- `petit --benchmark` mode
  - accepts repeat-run benchmark flags and prints startup + per-run timing summary (`--warmup-runs`, `--runs`,
    `--max-new-tokens`)

Dependencies and constraints:

- Keep `petit-core` as the translation backend boundary; do not move inference logic into `petit-tui`.
- Use existing Rust standard library timing (`std::time::Instant`) for benchmark output unless a stronger need appears.
- Preserve existing feature-flag paths (`cpu-only`, `cuda`, `metal`, `vulkan`) and document which feature set was used
  for evidence.

## Revision Note

- 2026-02-23: Recreated Phase 5 ExecPlan from scratch after the prior Phase 5 plan file was intentionally discarded.
  Requirements were reconstructed from the discarded file's `Purpose / Big Picture` and `Progress` sections and grounded
  with fresh research in `docs/execution-plans/research/2026-02-23-phase-5-polish-performance-ux-reset.md`.
- 2026-02-23: Executed the plan implementation for benchmark tooling and TUI UX feedback improvements, then updated
  living sections with test/verification evidence and a documented benchmark blocker (27B CPU-only timeout).
- 2026-02-23: Migrated benchmarking into `petit --benchmark`, removed the standalone benchmark example, added
  `--src`/`--tgt` aliases, fixed benchmark stdin handling for non-TTY runs, recorded user CPU/Metal benchmark results,
  and re-ran `./scripts/check.sh --fix`.
