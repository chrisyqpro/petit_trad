# Translation Eval Harness Research

## Why this research exists

The repository has a strong code-quality and performance verification path (`scripts/check.sh` and benchmark mode), but
it does not have a repeatable regression harness that runs real translations and compares the output against checked-in
fixtures. The requested enhancement is a small `eval.sh` flow that can be run repeatedly to detect translation behavior
regressions.

This research documents the current translation execution path, the configuration and determinism constraints that
affect reproducibility, and a recommended implementation shape for a first harness.

## Current execution path for real translations

### CLI/TUI crate is the runnable entry point

- `crates/petit-tui/src/main.rs` is the primary binary entry point.
- `crates/petit-tui/src/cli.rs` parses CLI flags including `--stdin`, `--benchmark`, `--model`, `--source-lang`, and
  `--target-lang`.

There are two non-interactive paths that run real translations:

1. `--stdin` mode (`run_stdin` in `crates/petit-tui/src/main.rs`)
   - Reads text from stdin
   - Constructs `GemmaTranslator`
   - Calls `translate`
   - Prints only the translated output to stdout

2. `--benchmark` mode (`run_benchmark` in `crates/petit-tui/src/main.rs`)
   - Constructs `GemmaTranslator`
   - Runs one or more translations
   - Prints timing summaries and a `Target: ...` line
   - Supports `--max-new-tokens`, `--runs`, and `--warmup-runs`

For a fixture-comparison harness, `--stdin` mode is the cleanest output surface because it prints only the translation.

### Core translation behavior lives in `petit-core`

- `crates/petit-core/src/gemma.rs` implements `GemmaTranslator`.
- `GemmaTranslator::translate` validates the language pair, builds the TranslateGemma prompt, calls
  `ModelManager::infer`, and strips whitespace / trailing `<end_of_turn>` artifacts.
- `crates/petit-core/src/model_manager.rs` performs real llama.cpp inference with greedy sampling
  (`sample_token_greedy`), which is helpful for repeatability.

Important implication: a harness that shells out to `cargo run -p petit-tui -- --stdin ...` will exercise the real
translation path end-to-end (CLI config loading + `GemmaTranslator` + `ModelManager` inference).

## Existing testing and script coverage (gap analysis)

### What already exists

- `scripts/check.sh` is the canonical verification script and runs fmt, clippy, check, and tests.
- Unit tests in `crates/petit-core/src/gemma.rs` cover prompt formatting and output cleaning.
- Unit tests in `crates/petit-tui/src/app.rs` cover app-state transitions around translation requests/results.
- `proto/translate_test.py` runs real translations with a local GGUF model, but it is a prototype/manual script and
  does not compare outputs to checked-in fixtures.

### What is missing

- No script for repeatable translation regression checks against fixtures.
- No checked-in translation output fixtures (expected outputs).
- No documented workflow for re-running a stable translation behavior check before/after changes.

## Reproducibility constraints that affect a translation eval harness

### Config precedence can change behavior unexpectedly

`crates/petit-tui/src/config.rs` loads configuration in this order:

- bundled `config/default.toml`
- optional user config (`$XDG_CONFIG_HOME/...` or `$HOME/.config/...`)
- environment (`PETIT_TRAD_*`)
- CLI overrides

Implication: if the harness does not explicitly constrain config sources, local user config or environment values can
silently change model path, language defaults, thread count, GPU offload, and log behavior.

Recommendation: the harness should pass `--no-config` and explicitly set the required CLI flags (`--model`,
`--source-lang`, `--target-lang`, plus runtime knobs if needed).

### Model choice is part of the regression surface

- `config/default.toml` points to a 12B TranslateGemma GGUF model path.
- In the current workspace, `models/` contains a 27B GGUF model path and not the default 12B path.

Implication: a harness must not assume the default model path exists. It should require an explicit model path (flag or
environment variable) and report a clear error when it is missing.

### Determinism is good but not absolute across environments

The inference loop uses greedy sampling, which improves repeatability on the same hardware/config. However, outputs can
still drift when changing:

- model size/quantization file
- backend feature set (`cpu-only` vs `metal` / `cuda` / `vulkan`)
- runtime settings (context, threads, GPU layers)
- upstream model/runtime behavior over time

Implication: the first harness should optimize for local repeatability on a given machine/config, and its fixtures
should be treated as tied to a specific model/runtime configuration.

## Fixture format considerations for a shell-based harness

The requested deliverable is `eval.sh`, so the harness parser should avoid external dependencies (`jq`, Python, etc.)
unless necessary.

Observed constraints:

- Translation inputs and outputs may contain spaces and punctuation.
- Multi-line fixtures are possible but would complicate a small POSIX shell parser.
- The harness should be easy to review in git and update intentionally.

Practical first-version options:

1. TSV/pipe-delimited file (single-line inputs/outputs only)
   - Simple to parse in shell
   - Easy to diff
   - Limits fixture scope to short single-line cases (acceptable for a small regression harness)

2. Directory-per-case fixtures (`input.txt`, `expected.txt`, metadata file)
   - More robust for multi-line text
   - Slightly more verbose, but still shell-friendly

Recommendation for first version: start with a small, line-oriented fixture file for simple smoke/regression cases, and
document that fixtures are intentionally short single-line examples. This keeps the implementation small and repeatable.

## Where `eval.sh` should fit in the repository

Current script placement and conventions:

- `scripts/check.sh` is a simple POSIX shell script and is referenced from `docs/BUILD.md`.
- There is no existing `scripts/eval.sh`.

Recommended placement:

- `scripts/eval.sh` as a sibling to `scripts/check.sh`
- fixture files under a dedicated directory such as `eval/fixtures/` (new)

This keeps the harness discoverable and consistent with the repo’s script-oriented workflows.

## Recommended direction for the ExecPlan

The implementation should add a small, real-translation regression harness with the following behavior:

- `scripts/eval.sh` runs a set of fixture cases by invoking `cargo run -p petit-tui -- --stdin ...` for each case.
- The harness requires an explicit model path (CLI flag or env var) and fails fast with a clear message if not found.
- The harness compares actual stdout translation output against expected fixture output and prints pass/fail per case.
- The harness exits non-zero if any case fails.
- The harness supports selecting a fixture file (or fixture set) so it can stay small and be extended later.
- Documentation (`docs/BUILD.md`) should include a short "translation eval" section with an example command.

This approach adds meaningful regression coverage without changing the core translation behavior or introducing new test
framework dependencies.

## Post-implementation correction: fixture quality matters

The first implementation iteration revealed an important mistake: opaque token
or identifier cases (for example random-looking IDs) primarily test verbatim
preservation behavior, not translation behavior. They are useful for detecting
rewrites of codes/IDs, but they do not verify that simple phrases are
translated correctly.

Two concrete corrections are required for a trustworthy eval suite:

1. Split fixture suites by purpose.
   - `translation` fixtures: simple natural-language phrases and short literals
     that exercise real translation choices (wording, punctuation,
     localization).
   - `preserve` fixtures: opaque identifiers that should remain unchanged.

2. Treat translation coverage as a semantic test corpus, not a row-count game.
   A large count of opaque-ID rows can inflate the suite size while providing
   little evidence that translation quality is preserved.

Security/privacy correction learned during implementation: fixture files must not
contain local paths, hostnames, emails, secrets, or command-like strings. Even
when the harness does not execute fixture text, committing such data is unsafe
and unnecessary.

## Risks and open questions to settle in planning

1. Fixture strictness: exact string match vs normalized match (trim only). Exact match is stronger but more brittle.
2. Runtime defaults: whether the harness should force stable runtime knobs (e.g., `--gpu-layers 0`) or inherit local
   performance-oriented settings.
3. Fixture corpus design: how to curate 30 meaningful simple-translation cases
   across language pairs and capture exact expected outputs on a machine that
   can run the chosen model/backend reliably.
4. Fixture metadata: whether to encode model/runtime expectations in the fixture file now or defer to documentation for
   the first version.
5. Command shape: whether `scripts/eval.sh` should call `cargo run` every case (simple but slower) or build once and
   reuse the binary (faster but more script complexity).
