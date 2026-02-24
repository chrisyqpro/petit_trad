# Build Guide

This guide covers local development setup for `petit_trad`.

## Prerequisites

- Rust toolchain (stable)
- C/C++ build tools
- Git

Optional by backend:

- CUDA toolkit for `--features cuda`
- Xcode Command Line Tools for `--features metal` on macOS

## Verification

### Code Check Script

The canonical command -- run this before every commit. It auto-formats, then lints, then check and tests:

```bash
./scripts/check.sh --fix
```

Check-only mode (matches CI exactly, no auto-format):

```bash
./scripts/check.sh
```

Override features for a local GPU build:

```bash
./scripts/check.sh --fix --features metal   # macOS Metal
./scripts/check.sh --fix --features cuda    # CUDA
```

To run individual stages without the script:

```bash
cargo fmt --check
cargo clippy --workspace --features cpu-only -- -D warnings
cargo check --workspace --features "$FEATURES"
cargo test --workspace
```

### Translation Regression Eval

Use the eval harness to run real translations and compare stdout against
checked-in fixtures. This is separate from `scripts/check.sh` because it needs a
local GGUF model and can be slow.

Example (run smoke eval against an existing captured `smoke.tsv`):

```bash
./scripts/eval.sh \
  --model models/translategemma-27b-it-GGUF/translategemma-27b-it.Q8_0.gguf
```

If your local model or machine needs different runtime settings, override them:

```bash
./scripts/eval.sh \
  --model /absolute/path/to/model.gguf \
  --features metal \
  --gpu-layers 0 \
  --context-size 256 \
  --threads 1
```

Notes:

- Fixtures live under `eval/fixtures/`.
- `eval/fixtures/smoke-inputs.tsv` is the curated simple-translation corpus
  used to generate `eval/fixtures/smoke.tsv`.
- `eval/fixtures/smoke.tsv` is for actual translation behavior checks.
- The harness uses `petit-tui` `--stdin` mode and passes `--no-config` plus
  explicit model/language flags for each case.
- The harness prints per-case pass/fail and exits non-zero on mismatches or
  translation command errors.

Capture or refresh translation expectations on a working local backend (for
example Metal) before running the smoke eval:

```bash
./scripts/eval-capture.sh \
  --model /absolute/path/to/model.gguf \
  --features metal \
  --gpu-layers 0 \
  --context-size 256 \
  --threads 1
```

### Runtime Smoke Harness

Use the smoke harness for quick runtime sanity checks with readable per-check
status output:

```bash
./scripts/smoke.sh
```

The harness prints one line per check with `PASS`, `FAIL`, or `SKIP`, then a
final summary. It includes model-free checks (help/version and validation
errors) plus an optional model-backed stdin translation smoke.

If no local model file is available, the model-backed smoke check reports
`SKIP`. This is expected on fresh clones and does not replace the canonical
verification script.

To run the model-backed smoke check with a specific local GGUF file:

```bash
SMOKE_MODEL_PATH=/absolute/path/to/model.gguf ./scripts/smoke.sh
```

## Run TUI

```bash
cargo run -p petit-tui
```

GPU examples:

```bash
cargo run -p petit-tui --features cuda
cargo run -p petit-tui --features metal
cargo run -p petit-tui --features vulkan
```

### Startup Feedback

The TUI now starts a background translator worker immediately and reports translator initialization status in the
footer. This makes first-use behavior more predictable because model initialization is surfaced before the first
translation request.

## Stdin Mode

```bash
echo "Hello, how are you?" | cargo run -p petit-tui -- --stdin --target-lang fr
```

## Benchmarking Translation Latency

Use the `petit` benchmark mode for repeatable local measurements. It reports:

- `Startup`: translator/model initialization time (`GemmaTranslator::new`)
- `Warmup` runs (optional, excluded from summary)
- measured `Run N` timings
- `Average`, `Min`, and `Max` across measured runs

Example (CPU-only):

```bash
cargo run -p petit-tui --features cpu-only -- --benchmark \
  --model models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf \
  --source-lang en --target-lang fr \
  --text "Hello, how are you?" \
  --max-new-tokens 64 \
  --warmup-runs 1 \
  --runs 3
```

Benchmark matrix to record in local notes / ExecPlans:

- Short input: one sentence (cold + warm runs)
- Medium input: 3-5 sentence paragraph (cold + warm runs)
- Cold result: `Startup` plus the first measured `Run`
- Warm result: subsequent measured `Run` values after warmup

Interpretation notes:

- `Startup` is usually the largest component on the first use because it includes model load and backend initialization.
- `Run N` times are hardware-, model-, and feature-dependent (`cpu-only`, `metal`, `cuda`, `vulkan`), so do not compare
  numbers across machines without recording the exact config.
- Use a lower `--max-new-tokens` value for quick sanity checks, and a stable value when comparing before/after results.
- Benchmark mode runs through the normal `petit-tui` config loader, so config file, env vars, and CLI overrides all
  apply.

## Config and Model

- Default config file: `config/default.toml`
- User config path: `$XDG_CONFIG_HOME/petit_trad/config.toml` (fallback: `$HOME/.config/petit_trad/config.toml`)
- Config file precedence: `--config <path>` > XDG user config > bundled default
- Model files are expected under `models/` unless overridden by CLI or env vars

## WSL CUDA Notes

When building with CUDA in WSL, these vars are often required:

```bash
export CUDACXX=/usr/local/cuda/bin/nvcc
export CUDAToolkit_ROOT=/usr/local/cuda
```

## GitHub Actions Workflows

The repository includes:

- `.github/workflows/ci.yml`: runs CPU-only checks on `ubuntu-latest` and `macos-latest` for push and pull request
  events to `main`
- `.github/workflows/release.yml`: runs on pushed tags matching `v*`, builds release binaries, and publishes `tar.gz`
  assets to a GitHub release

## Release Commands

Create and push a semantic tag from the repository root:

```bash
git tag v0.1.0
git push origin v0.1.0
```

After the workflow completes, the release should include:

- `petit-v0.1.0-linux-x64.tar.gz`
- `petit-v0.1.0-macos-arm64.tar.gz`

## Troubleshooting

- If CI fails only on one runner, compare local output with `cargo test --workspace --features cpu-only`.
- If a release is missing artifacts, verify the tag matches `v*` and that the `build` job succeeded before `publish`.
- If stale build output causes local confusion, run `cargo clean` and rerun the target command.
