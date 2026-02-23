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

The canonical command -- run this before every commit. It auto-formats, then
lints, then check and tests:

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

### Ad-hoc Commands

For running individual stages without the script:

```bash
cargo fmt --check
cargo clippy --workspace --features cpu-only -- -D warnings
cargo check --workspace --features "$FEATURES"
cargo test --workspace
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

## Stdin Mode

```bash
echo "Hello, how are you?" | cargo run -p petit-tui -- --stdin --target-lang fr
```

## Config and Model

- Default config file: `config/default.toml`
- User config path: `$XDG_CONFIG_HOME/petit_trad/config.toml`
  (fallback: `$HOME/.config/petit_trad/config.toml`)
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

- `.github/workflows/ci.yml`: runs CPU-only checks on `ubuntu-latest` and
  `macos-latest` for push and pull request events to `main`
- `.github/workflows/release.yml`: runs on pushed tags matching `v*`, builds
  release binaries, and publishes `tar.gz` assets to a GitHub release

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

- If CI fails only on one runner, compare local output with
  `cargo test --workspace --features cpu-only`.
- If a release is missing artifacts, verify the tag matches `v*` and that the
  `build` job succeeded before `publish`.
- If stale build output causes local confusion, run `cargo clean` and rerun the
  target command.
