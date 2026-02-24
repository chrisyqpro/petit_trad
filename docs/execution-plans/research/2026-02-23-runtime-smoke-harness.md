# Runtime Smoke Harness Research

## Why this research exists

The request is to add a `smoke.sh` script that runs runtime smoke checks and prints output that clearly shows which
checks passed or failed.

This research captures the current verification and runtime entrypoints so the plan can define a smoke harness that is
useful on developer machines without requiring a local model file.

## Current verification surface

### Canonical check script (`scripts/check.sh`)

`scripts/check.sh` is the repository's canonical verification script and is documented in `docs/BUILD.md` as the command
to run before commits.

It runs these stages in order:

- `cargo fmt` (or `cargo fmt --check`)
- `cargo clippy --workspace --features "$FEATURES" -- -D warnings`
- `cargo check --workspace --features "$FEATURES"`
- `cargo test --workspace --features "$FEATURES"`

Important behavior:

- The script uses `set -eu`.
- It exits on the first failing command.
- It prints stage headers as it goes, but it does not print a final per-stage pass/fail summary.

Implication for smoke output: if one command fails, the user can only see the first failure and cannot immediately see
which later smoke checks were not run.

### CI usage (`.github/workflows/ci.yml`)

CI currently runs only `./scripts/check.sh` on macOS and Linux. There is no separate smoke harness in CI today.

Implication: adding `smoke.sh` can be an additive developer tool first. It does not need CI integration to be useful.

## Runtime entrypoints relevant to a smoke harness

### TUI executable entry flow (`crates/petit-tui/src/main.rs`)

`run()` processes CLI arguments before loading config only for:

- `--help`
- `--version`

These are reliable positive smoke checks because they avoid config file parsing, model loading, and terminal setup.

After that, `run()` calls `load_config(&cli)` and may enter:

- benchmark mode (`run_benchmark`)
- stdin mode (`run_stdin`)
- TUI mode (terminal setup + worker thread)

### CLI parser behavior (`crates/petit-tui/src/cli.rs`)

`CliArgs::parse()` returns descriptive errors before config loading for invalid or conflicting arguments (for example
unknown flags, missing values, or `--no-config` combined with `--config`).

Implication: the smoke harness can include negative checks (expected failures) and still classify them as PASS when the
expected error text appears.

### Config loading behavior (`crates/petit-tui/src/config.rs`)

`load_config()` always reads `config/default.toml` first unless `--help`/`--version` short-circuits earlier in
`run()`.

Important behavior discovered from static reading:

- `load_config()` validates required config values and language pair.
- It does not verify that the model file exists during config load.
- Model file existence is checked later when `GemmaTranslator::new(config.core)` is called.

Implication: smoke checks can exercise config loading and some runtime argument validation without a local model file, as
long as they fail before translator initialization.

## Default config and local model availability

`config/default.toml` points to:

- `models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf`

In this worktree, `models/` does not exist.

Implication:

- Any smoke check that requires translator initialization will fail on a fresh clone unless it supports skip behavior or
  an injected model path.
- A robust smoke harness should clearly distinguish `PASS`, `FAIL`, and `SKIP` states.

## Candidate smoke checks that do not require a local model

Based on code flow, these are strong candidates for deterministic smoke checks:

- `cargo run -p petit-tui -- --help` (expected success)
- `cargo run -p petit-tui -- --version` (expected success)
- `cargo run -p petit-tui -- --definitely-invalid-flag` (expected failure with `Unknown argument`)
- `cargo run -p petit-tui -- --no-config --config config/default.toml` (expected failure with conflict message)
- `printf '' | cargo run -p petit-tui -- --stdin` (expected failure with `stdin is empty`; reaches config load and
  runtime stdin path without model init)
- `cargo run -p petit-tui -- --benchmark --runs 0` (expected failure with `--runs must be at least 1`; reaches config
  load and benchmark path without model init)

These checks cover:

- process startup and binary execution
- CLI parsing
- config load and validation path
- runtime mode dispatch and early validation errors

## Design implications for the ExecPlan

1. `smoke.sh` should be independent from `scripts/check.sh`, not a replacement.
2. The script should aggregate results instead of using `set -e` fail-fast semantics.
3. The script output should print one clear line per check with a stable status label (`PASS` / `FAIL` / `SKIP`) and a
   final summary/count.
4. The script should support expected-failure checks (non-zero exit can still be PASS when failure is intentional and
   message matches).
5. Optional model-dependent smoke should be guarded by a model-path existence check and reported as `SKIP` when absent.
6. `docs/BUILD.md` should be updated to document when to use `smoke.sh` versus `scripts/check.sh`, and what a skipped
   model-dependent check means.
