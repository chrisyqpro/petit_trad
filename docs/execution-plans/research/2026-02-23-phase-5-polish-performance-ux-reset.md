# Phase 5 Reset Research: Polish, Performance, and UX Feedback

## Why this research exists

The prior active Phase 5 ExecPlan was discarded after reading, per request. Its `Purpose / Big Picture` and `Progress`
sections were used as requirement input for a fresh workflow restart.

Requirement signal extracted from the discarded plan:

- Measure and document translation performance behavior.
- Improve user feedback for configuration and runtime failures.
- Deliver faster and more predictable interaction, plus clearer status/error messaging.
- Complete the previously listed unfinished work: benchmark matrix, baseline measurements, focused optimizations with
  evidence, UX message improvements, and documentation updates.

## Current architecture and ownership

`ARCHITECTURE.md` confirms the split:

- `crates/petit-core` owns translation logic, model runtime integration, and inference.
- `crates/petit-tui` owns terminal UX, status feedback, and worker-thread orchestration.

This phase spans both crates because performance changes are mainly in `petit-core`, while clearer user feedback is
rendered and managed in `petit-tui`.

## Translation flow in current code

### Core inference path (`crates/petit-core`)

- `crates/petit-core/src/gemma.rs` implements `GemmaTranslator`, validates the language pair, builds the TranslateGemma
  prompt, calls `ModelManager::infer`, and cleans the output.
- `crates/petit-core/src/model_manager.rs` is the latency-critical path. `ModelManager::new` validates model file
  existence, initializes the llama.cpp backend, configures logging, and loads the GGUF model.
- `ModelManager::infer` does per-request work:
  - creates a new `LlamaContext`
  - tokenizes the prompt
  - builds and decodes the prompt batch
  - runs a greedy token-generation loop
  - detokenizes generated tokens into a `String`

Important observation: there is no built-in timing breakdown for these steps. At research time, the only timing present
was coarse total elapsed time in the then-existing `crates/petit-core/examples/translate.rs` benchmark example (later
replaced by `petit --benchmark` mode).

### TUI orchestration and status path (`crates/petit-tui`)

- `crates/petit-tui/src/main.rs` runs the event loop and starts a background translation worker thread
  (`start_translation_worker`).
- The worker lazily creates `GemmaTranslator` on the first request. This means the first translation includes model
  initialization and model load cost.
- `crates/petit-tui/src/app.rs` owns `App` state and all user-visible status updates through `status_message` plus
  `is_loading`.
- `App::begin_translation` sets `is_loading = true` and `status_message` to `"Translating..."`.
- `App::apply_translation_result` clears loading and either sets `"Translation complete"` or stores the raw error
  string.
- `crates/petit-tui/src/ui.rs` renders the footer through `status_widget`, showing spinner text when loading, otherwise
  language-edit prompts, then `status_message`, else `"Ready"`.

Important observation: status is a single untyped string (`Option<String>`). The UI cannot distinguish
info/success/error states for styling or persistence.

## Existing error surfaces and message quality

### Core errors

`crates/petit-core/src/error.rs` defines a small `Error` enum:

- `Config(String)`
- `ModelLoad(String)`
- `Inference(String)`
- `UnsupportedLanguage(String)`

These are useful categories, but many messages are raw lower-level strings wrapped with prefixes like `Model load:` or
`Decode:`. They are valid for debugging but not always ideal TUI-facing guidance.

### TUI and config errors

- `crates/petit-tui/src/config.rs` validates language pairs during startup and returns errors such as
  `Invalid language pair: ...`.
- `crates/petit-tui/src/main.rs` prints fatal startup errors to stderr in `main()` (`eprintln!("Error: {err}")`) and
  exits.
- Runtime worker failures are propagated back as strings and shown in the TUI footer (`status_message`) without
  classification or recovery hints.

Important observation: there are at least two user-facing paths (startup stderr and in-TUI footer) with different
behavior and no shared message-shaping policy.

## Performance baseline and benchmarking gaps

### What exists today

- At research time, the benchmark example measured end-to-end translation elapsed time with `Instant::now()`.
- Runtime knobs already exist in `crates/petit-core/src/config.rs` and are exposed by the TUI CLI (`--gpu-layers`,
  `--context-size`, `--threads`).
- Defaults in `config/default.toml` are tuned for a GPU-offload path (`gpu_layers = 999`, `context_size = 2048`,
  `threads = 4`).

### What is missing for Phase 5 goals

- No benchmark matrix definition (short vs. medium inputs, warm vs. cold runs).
- No repeatable measurement command documented in `README.md` or `docs/BUILD.md` for collecting evidence.
- No instrumentation inside `ModelManager::infer` to attribute time to context creation, prompt tokenization, decode,
  generation loop, or detokenization.
- No captured before/after evidence file for optimization claims.

## Testing and validation gaps relevant to Phase 5

- `crates/petit-tui/src/app.rs` has no state-level tests for status transitions (loading, success, error, duplicate
  request, empty input).
- `crates/petit-tui/src/ui.rs` does not currently encode different status severities, so UI tests cannot assert error
  styling behavior yet.
- `scripts/check.sh` is the required verification command and already runs format, clippy, check, and test in one path,
  which is suitable for plan validation after implementation.

## Design implications for the replacement Phase 5 plan

1. The plan should define a benchmark matrix and evidence format first, before optimization work, so improvements can be
   proven.
2. The plan should separate cold-start cost (translator/model init) from warm translation cost because the current
   worker lazily initializes on first use.
3. UX improvements should introduce typed status/error semantics in app state so `ui.rs` can render clearer feedback
   without string matching.
4. The plan should include docs updates in both `README.md` and/or `docs/BUILD.md` only after measurement commands and
   outputs are stable.
5. Acceptance criteria must require measured before/after data and observable TUI messaging changes, not just internal
   refactors.
