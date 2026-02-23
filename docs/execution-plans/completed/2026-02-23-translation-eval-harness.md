# Translation Eval Harness

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this change, contributors can run a small repeatable translation
regression check that exercises the real local translation path and compares the
results against checked-in fixtures. The visible outcome is a new
`scripts/eval.sh` command that prints pass/fail per case and exits non-zero when
translation behavior changes unexpectedly.

This does not replace `scripts/check.sh`. `scripts/check.sh` remains the
code-quality gate, while `scripts/eval.sh` becomes a behavior-regression check
that requires a local model file.

## Progress

- [x] (2026-02-23 20:25Z) Researched translation execution path, config
  precedence, and fixture/harness constraints in
  `docs/execution-plans/research/2026-02-23-translation-eval-harness.md`.
- [x] (2026-02-23 20:25Z) Authored initial ExecPlan for the translation eval
  harness.
- [x] (2026-02-23 20:48Z) Implemented `scripts/eval.sh` with fixture parsing,
  real `petit-tui --stdin` execution, exact output comparison, and summary exit
  codes.
- [x] (2026-02-23 20:48Z) Added `eval/fixtures/smoke.tsv` with documented TSV
  schema and starter smoke cases.
- [x] (2026-02-23 20:48Z) Documented translation eval usage in `docs/BUILD.md`.
- [x] (2026-02-23 20:48Z) Ran `./scripts/check.sh --fix` and a real
  `./scripts/eval.sh` attempt against the local 27B model; recorded outputs and
  local runtime failure details in this plan.
- [x] (2026-02-23 22:01Z) Replaced preservation-heavy placeholder coverage with
  a curated simple-translation smoke corpus and documented capture-first
  workflow using `eval/fixtures/smoke-inputs.tsv`.
- [x] (2026-02-23 21:40Z) Added a curated 30-case simple-translation corpus in
  `eval/fixtures/smoke-inputs.tsv` and a capture script to generate
  `eval/fixtures/smoke.tsv` from real local translations.
- [x] (2026-02-23 22:01Z) User captured and checked in 30 exact
  `eval/fixtures/smoke.tsv` expected outputs on a working Metal backend and
  confirmed `scripts/eval.sh` passed.
- [x] (2026-02-23 22:01Z) Verify phase complete: reran `./scripts/check.sh --fix`,
  recorded final evidence, and prepared this ExecPlan for move to `completed/`.

## Surprises & Discoveries

- Observation: The clean non-interactive translation path already exists in
  `petit-tui` via `--stdin`, and it prints only the translation output.
  Evidence: `crates/petit-tui/src/main.rs` `run_stdin` reads stdin, calls
  `GemmaTranslator::translate`, and prints `println!("{output}")`.

- Observation: Local configuration and environment variables can silently alter
  eval behavior if the harness does not constrain config loading.
  Evidence: `crates/petit-tui/src/config.rs` merges bundled config, user config,
  env vars, and CLI overrides in precedence order.

- Observation: The default configured model path in `config/default.toml` is not
  guaranteed to exist in each workspace clone.
  Evidence: Research-time workspace inspection showed a 27B model file under
  `models/translategemma-27b-it-GGUF/` while the default config points to a 12B
  path.

- Observation: The local 27B GGUF model in this workspace fails at runtime in
  `ModelManager::infer` context creation on this machine, even with reduced eval
  harness runtime overrides (`gpu_layers=0`, `context_size=128`, `threads=1`).
  Evidence: Real `scripts/eval.sh` runs reported
  `Inference error: Context creation: null reference from llama.cpp` for each
  fixture case.

- Observation: The starter numeric smoke fixtures were not behavior-stable
  placeholders. A successful Metal-backed run localized them (`42` ->
  `quarante-deux`, `3.14` -> `3,14`).
  Evidence: User-reported `scripts/eval.sh` output showed two exact mismatches
  with those actual values, and the fixture file was updated to match.

- Observation: Opaque-ID fixture rows are not a valid substitute for simple
  translation tests. Several rows passed, but they measured identifier
  preservation rather than translation quality; others were still translated or
  explained because the tokens contained meaningful words.
  Evidence: User-reported Metal run showed both semantic rewrites and long
  explanations for token-like inputs (`HOTEL_3A`, `LIMA_27B`, `SIGMA_72`).

- Observation: I cannot safely invent exact expected translations for a 30-case
  corpus from this environment because the local backend fails here while the
  user’s Metal backend succeeds and model outputs are exact-match fixtures.
  Evidence: `smoke-inputs.tsv` + `scripts/eval-capture.sh` were added so
  expectations are captured from a working backend rather than guessed.

- Observation: The capture-first workflow resolved the local-environment gap.
  The user’s working Metal backend successfully generated `smoke.tsv` and the
  eval harness passed with the captured expectations.
  Evidence: User confirmed "cases are successfully captured and eval passed".

## Decision Log

- Decision: The harness will invoke the real translation path through
  `cargo run -p petit-tui -- --stdin` instead of calling `proto/translate_test.py`
  or adding a second translation entry point.
  Rationale: This validates the production CLI config loader and `petit-core`
  translation stack end-to-end with minimal new code.
  Date/Author: 2026-02-23 / codex.

- Decision: The harness will pass `--no-config` and explicit language/model
  flags for every case.
  Rationale: Prevents user config and environment drift from making the eval
  non-repeatable.
  Date/Author: 2026-02-23 / codex.

- Decision: The first fixture format will be line-oriented, tab-separated, and
  limited to single-line input/output text.
  Rationale: This keeps `scripts/eval.sh` small and dependency-free while still
  covering common regression-smoke scenarios.
  Date/Author: 2026-02-23 / codex.

- Decision: Fixture comparison will be exact string match for the captured stdout
  value (after shell command-substitution removes the final newline).
  Rationale: Exact matching is the clearest regression signal for a small
  harness and avoids hidden normalization rules.
  Date/Author: 2026-02-23 / codex.

- Decision: Add runtime override flags to `scripts/eval.sh` (`--gpu-layers`,
  `--context-size`, `--threads`) with conservative defaults (`0`, `256`, `1`).
  Rationale: The repository config defaults are performance-oriented and can
  fail on larger local models; safer defaults improve harness portability and
  repeatability for short fixture cases.
  Date/Author: 2026-02-23 / codex.

- Decision: Keep `eval/fixtures/smoke.tsv` focused on translation behavior and
  do not count opaque-ID placeholder rows as translation coverage.
  Rationale: Opaque-ID rows can be useful for copy/preserve behavior checks, but
  they hide gaps in the actual goal ("simple translation" regression testing)
  when presented as translation coverage.
  Date/Author: 2026-02-23 / codex.

- Decision: Add `scripts/eval-capture.sh` and `eval/fixtures/smoke-inputs.tsv`
  to support a curated 30-case simple-translation corpus whose exact expected
  outputs are captured from a working local backend.
  Rationale: Exact-match translation fixtures are model/backend-dependent, and
  this environment cannot reliably run the available local 27B model.
  Date/Author: 2026-02-23 / codex.

## Outcomes & Retrospective

Implemented the eval harness, fixture format, and build documentation update.
The harness correctly exercises the real `petit-tui` stdin translation path and
reports per-case pass/fail/error plus summary counts.

On this machine, the available local 27B model fails during llama.cpp context
creation, so a successful translation transcript could not be recorded yet. The
harness still handled the real translation-command failures as designed and
surfaced the stderr details for diagnosis.

Follow-up correction: the expanded opaque-ID fixture set was not appropriate as
translation coverage. A curated 30-case translation corpus is now defined in
`eval/fixtures/smoke-inputs.tsv`. The remaining work is to capture exact
`smoke.tsv` expectations on a working backend and verify them.

Completed in Verify: the user captured the 30-case `smoke.tsv` expectations on
a working Metal backend and confirmed the eval harness passed. The project now
has a repeatable translation regression workflow with a curated corpus and a
capture step for exact expectations.

## Context and Orientation

This repository has two production crates:

- `crates/petit-core` contains the translation engine and local llama.cpp model
  integration.
- `crates/petit-tui` contains the terminal application and CLI entry point.

The real translation behavior requested by this enhancement is already reachable
without the interactive TUI via `petit-tui` stdin mode. In
`crates/petit-tui/src/main.rs`, `run_stdin` reads stdin text, constructs
`GemmaTranslator`, runs translation, and prints the translated text to stdout.

`crates/petit-tui/src/config.rs` loads config from `config/default.toml`, then
optional user config, then environment variables, then CLI flags. This matters
because a regression harness must be repeatable. If it inherits local config, it
can accidentally compare different models or runtime settings across runs.

The current repository has `scripts/check.sh` for fmt/lint/build/test, but no
translation behavior harness. The requested addition is a new `scripts/eval.sh`
and fixture sets under `eval/fixtures/`. Short single-line examples keep the
shell implementation simple, but fixture intent must be explicit:

- `eval/fixtures/smoke.tsv`: actual translation behavior (simple phrases and
  localized literals)
- `eval/fixtures/smoke-inputs.tsv`: curated simple-translation corpus to capture
  exact expectations from a working local backend

This plan builds on the research captured in
`docs/execution-plans/research/2026-02-23-translation-eval-harness.md`, but it
repeats all implementation-critical details here so a new contributor can work
from this file alone.

## Plan of Work

Milestone 1 adds the harness shell script at `scripts/eval.sh`. The script will
parse a minimal CLI, validate required inputs (especially model path and fixture
file path), iterate fixture rows, invoke the real translator via `cargo run`,
compare outputs, and print a concise summary. At the end of this milestone,
contributors can run a command and see pass/fail results on known fixtures.

Milestone 2 adds fixture files under a new `eval/fixtures/` directory. The
translation smoke fixture (`smoke.tsv`) must contain simple, meaningful
translation examples (short phrases and literals) and must not be padded with
opaque-ID placeholder rows. Fixture files must include comments that document
schema and recording assumptions.

Milestone 3 updates `docs/BUILD.md` with a translation eval section and records
evidence in this plan. The documentation must clearly state that the command
requires a local GGUF model and show an example that points at an explicit model
path. This milestone also runs the canonical check script and the eval script
and captures short transcripts in `Artifacts and Notes`.

## Concrete Steps

Run all commands from the repository root:

    cd /Users/dzr/src/repo/petit_trad

Implement the harness script and make it executable:

    edit scripts/eval.sh
    chmod +x scripts/eval.sh

Create the fixture directory and initial fixture file:

    mkdir -p eval/fixtures
    edit eval/fixtures/smoke.tsv

Add docs usage instructions:

    edit docs/BUILD.md

Create or revise the curated simple-translation corpus:

    edit eval/fixtures/smoke-inputs.tsv

Capture exact translation expectations from a working backend:

    ./scripts/eval-capture.sh \
      --model /absolute/path/to/model.gguf \
      --features metal \
      --gpu-layers 0 \
      --context-size 256 \
      --threads 1

Generate or verify fixture outputs using the explicit model path. The script
must call the real translator path in this shape (the harness wraps this):

    printf '%s' "Hello, how are you?" | cargo run -p petit-tui --features cpu-only -- \
      --stdin --no-config \
      --model /absolute/path/to/model.gguf \
      --source-lang en \
      --target-lang fr

Run the harness against the fixture set:

    ./scripts/eval.sh --model /absolute/path/to/model.gguf

Expected success transcript shape (exact timings are not relevant):

    [PASS] en-fr-hello
    [PASS] en-de-weather
    Summary: 2 passed, 0 failed, 0 errors

Run the canonical verification script before commit:

    ./scripts/check.sh --fix

## Validation and Acceptance

The change is accepted when all of the following are true:

- `scripts/eval.sh` exists and runs a real translation for each fixture by
  invoking `petit-tui` stdin mode (`--stdin`) with explicit `--model`,
  `--source-lang`, and `--target-lang` flags.
- The harness passes `--no-config` so local user config and environment files do
  not change behavior silently.
- The harness prints per-case pass/fail status and a summary count, and exits
  non-zero when any fixture fails or when a required input (such as the model
  path) is missing.
- A checked-in fixture file exists under `eval/fixtures/` and is documented with
  its tab-separated schema and single-line limitation.
- Translation coverage claims are based on natural-language or localized-literal
  cases in `eval/fixtures/smoke.tsv`, not opaque-ID preservation rows.
- `eval/fixtures/smoke.tsv` expectations are generated from
  `eval/fixtures/smoke-inputs.tsv` on a working backend and then checked in.
- `docs/BUILD.md` documents how to run the translation eval, including the need
  for a local GGUF model path.
- `./scripts/check.sh --fix` passes after the new script and docs are added.
- A real `./scripts/eval.sh` run is executed against a local model and its
  output is recorded in this plan’s `Artifacts and Notes` section.

Optional negative-path verification (recommended during implementation): change a
fixture expected output temporarily and confirm the harness reports `[FAIL]` and
returns a non-zero exit code, then restore the fixture.

## Idempotence and Recovery

The planned changes are additive and safe to rerun:

- Re-running `scripts/eval.sh` does not modify source files or fixtures.
- Re-running `./scripts/check.sh --fix` is safe and already part of the project
  workflow.
- Recreating `eval/fixtures/` with `mkdir -p` is idempotent.

If a run fails:

- Missing model path: provide `--model /absolute/path/to/model.gguf` and rerun.
- Fixture mismatch: inspect the reported expected vs actual values, determine
  whether the translation behavior regressed or the fixture should be updated,
  then rerun after a deliberate decision.
- Build failure in `cargo run`: run `./scripts/check.sh --fix` first to restore a
  clean compile/test baseline.

## Artifacts and Notes

Record evidence here as work proceeds.

Planned artifacts to capture:

- `scripts/eval.sh --help` or usage output (if implemented)
- Successful `./scripts/eval.sh --model ...` transcript
- One failure transcript showing mismatch formatting (optional but useful)
- `./scripts/check.sh --fix` completion line

Recorded artifacts:

- `scripts/eval.sh --help` output (2026-02-23):

    Usage: ./scripts/eval.sh --model <path> [options]
    ...
    --gpu-layers <n>    GPU layers override (default: 0)
    --context-size <n>  Context size override (default: 256)
    --threads <n>       CPU threads override (default: 1)

- `./scripts/check.sh --fix` completion (2026-02-23):

    ==> all checks passed

- Final `./scripts/check.sh --fix` completion during Verify (2026-02-23):

    ==> all checks passed

- Real eval harness attempt against local 27B model (2026-02-23):

    ./scripts/eval.sh --model models/translategemma-27b-it-GGUF/translategemma-27b-it.Q8_0.gguf \
      --features metal --gpu-layers 0 --context-size 128 --threads 1

    [ERROR] digits-en-fr (translator command failed, exit 1)
      stderr: ... Running `target/debug/petit --stdin --no-config --model ...`
      stderr: ... `--source-lang en --target-lang fr --gpu-layers 0`
      stderr: ... `--context-size 128 --threads 1`
      stderr: Error: Inference error: Context creation: null reference from llama.cpp
    [ERROR] decimal-en-de (translator command failed, exit 1)
      stderr: ... Running `target/debug/petit --stdin --no-config --model ...`
      stderr: ... `--source-lang en --target-lang de --gpu-layers 0`
      stderr: ... `--context-size 128 --threads 1`
      stderr: Error: Inference error: Context creation: null reference from llama.cpp
    Summary: 0 passed, 0 failed, 2 errors

- User-verified successful capture + eval on Metal backend (2026-02-23):

    ./scripts/eval-capture.sh ... --features metal ...
    ./scripts/eval.sh ... --features metal ...
    Result: capture succeeded and eval passed (user confirmation)

Fixture file schema (implemented in `eval/fixtures/smoke.tsv`):

- UTF-8 text file
- Lines beginning with `#` are comments
- Blank lines are ignored
- Data lines are tab-separated with five columns:
  `case_id<TAB>source_lang<TAB>target_lang<TAB>input_text<TAB>expected_output`
- Version 1 limitation: `input_text` and `expected_output` must be single-line
  and must not contain tab characters

Fixture corpus files (implemented):

- `eval/fixtures/smoke.tsv`: translation behavior smoke checks
- `eval/fixtures/smoke-inputs.tsv`: curated translation corpus inputs (30 cases)

## Interfaces and Dependencies

Implement the following script interface in `scripts/eval.sh` (POSIX `sh`):

- Required:
  - `--model <path>`: absolute or repo-relative path to the GGUF model file to
    use for all fixture cases
- Optional:
  - `--fixtures <path>`: defaults to `eval/fixtures/smoke.tsv`
  - `--features <value>`: Cargo feature set passed to `cargo run`; defaults to
    `cpu-only`
  - `--gpu-layers <n>`: defaults to `0`
  - `--context-size <n>`: defaults to `256`
  - `--threads <n>`: defaults to `1`
  - `--help`: prints usage and exits 0

Script behavior requirements:

- Exit `0` when all cases pass.
- Exit `1` when one or more cases fail or a translation command fails.
- Exit `2` for usage errors (unknown flags, missing flag values, missing fixture
  file, missing model file).
- Use `cargo run -p petit-tui --features "$FEATURES" -- --stdin --no-config ...`
  for each case so the real production translation path is exercised.
- Pass explicit runtime knobs (`--gpu-layers`, `--context-size`, `--threads`)
  for repeatability and to avoid relying on bundled config defaults.
- Capture stdout for comparison and compare exact strings (command-substitution
  semantics remove the final trailing newline).
- Print a deterministic summary line with counts.

Dependencies and constraints:

- No new runtime dependency such as `jq` or Python is required for the harness.
- The harness depends on an already-available local GGUF model file and a
  working Rust toolchain.
- The harness intentionally does not run inside `scripts/check.sh` because it is
  model-dependent and expensive.
- `scripts/eval-capture.sh` is the supported way to regenerate
  `eval/fixtures/smoke.tsv` from `eval/fixtures/smoke-inputs.tsv` on a working
  local backend.

## Revision Note

- 2026-02-23: Initial ExecPlan created after research approval to define a small
  fixture-based translation regression harness with `scripts/eval.sh`.
- 2026-02-23: Updated after implementation with runtime override flags, local
  verification results, and artifact transcripts; local 27B eval runs fail in
  llama.cpp context creation on this machine.
- 2026-02-23: Updated smoke fixture expectations after a successful user Metal
  run showed localized numeric outputs (`quarante-deux`, `3,14`).
- 2026-02-23: Corrected fixture strategy after user review; removed opaque-ID
  rows from translation coverage claims and reopened work for a curated
  simple-translation corpus.
- 2026-02-23: Added `smoke-inputs.tsv` (30 curated translation cases) and
  `scripts/eval-capture.sh` so exact expectations are captured from a working
  backend instead of guessed.
- 2026-02-23: Verify complete; user-confirmed Metal capture/eval success
  recorded, final `check.sh` rerun passed, and plan marked ready to move to
  `completed/`.
