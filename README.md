# petit_trad

Local translation tool using TranslateGemma models via local llama.cpp bindings.

## Goals

- Run TranslateGemma locally (4B / 12B / 27B) without cloud APIs
- Provide a terminal-first UX (TUI), with future GUI expansion
- Support WSL, Linux, macOS, and Windows

## Quick Start

1. Install Rust (stable) and Cargo.
1. Validate the workspace:

```bash
cargo check
cargo test --workspace
```

1. Run the TUI:

```bash
cargo run -p petit-tui
```

1. One-shot stdin mode:

```bash
echo "Hello, how are you?" | cargo run -p petit-tui -- --stdin --target-lang fr
```

## Performance and Startup Behavior

- The TUI initializes the translator worker in the background at startup and shows footer status updates
  (`Initializing translator...`, `Translator ready`, or an initialization error) before the first translation request.
- The first usable translation experience is now easier to reason about because model initialization is surfaced
  separately from translation completion.

For repeatable local latency measurements (startup + warm runs), use:

```bash
cargo run -p petit-tui --features cpu-only -- --benchmark \
  --model <path-to-gguf> \
  --source-lang en --target-lang fr \
  --text "Hello, how are you?" \
  --max-new-tokens 64 \
  --warmup-runs 1 \
  --runs 3
```

Record short-input and medium-input runs separately. `Startup` captures model initialization time, while `Run N` values
capture translation time after the translator is available. Latency is highly hardware- and model-dependent, so compare
numbers only when the model, feature flags, and runtime config match. Benchmark mode uses the normal config/env/CLI
precedence path, so you can omit model/language flags and rely on your configured defaults.

## CI and Releases

Pull requests and pushes to `main` run CPU-only checks on Linux and macOS via GitHub Actions.

Release artifacts are produced when pushing a tag that matches `v*`, for example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Each release uploads `tar.gz` archives containing the `petit` binary for Linux x64 and macOS arm64.

To run the same CI commands locally:

```bash
cargo check --workspace --features cpu-only
cargo test --workspace --features cpu-only
```

## GPU Backends

Enable backend features at build/run time:

- CUDA (WSL/Linux NVIDIA): `cargo run -p petit-tui --features cuda`
- Metal (macOS Apple Silicon): `cargo run -p petit-tui --features metal`
- Vulkan (Linux AMD): `cargo run -p petit-tui --features vulkan`
- CPU-only: omit GPU features or use `--features cpu-only`

WSL CUDA builds often require:

```bash
export CUDACXX=/usr/local/cuda/bin/nvcc
export CUDAToolkit_ROOT=/usr/local/cuda
```

If you hit backend or linker issues while validating CI paths locally, try a clean CPU-only pass first:

```bash
cargo clean
cargo check --workspace --features cpu-only
```

## Model Files

Models are not checked into git. By default, config expects:

- `models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf`

Override with CLI (`--model`), env vars, or config.

Config file precedence is:

- `--config <path>`
- `$XDG_CONFIG_HOME/petit_trad/config.toml` (or `$HOME/.config/petit_trad/config.toml`)
- `config/default.toml`

## Repository Organization

- `AGENTS.md`: AI agents instructions
- `ARCHITECTURE.md`: fast architectural map and invariants
- `crates/petit-core/`: translation engine library
- `crates/petit-tui/`: terminal UI binary (`petit`)
- `proto/`: Python prototype and experiments
- `docs/`: durable project documentation root
- `docs/PLANS.md`: execution-plan requirements and conventions
- `docs/BUILD.md`: project build guide
- `docs/design-docs/`: technical architecture and prompt docs
- `docs/product-specs/`: product scope and requirement docs
- `docs/execution-plans/`: active/completed execution plans and tracker

## License

GPL-3.0-or-later. See `LICENSE`.
