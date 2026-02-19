# petit_trad

Local translation tool using TranslateGemma models via local llama.cpp bindings.

## Goals

- Run TranslateGemma locally (4B / 12B / 27B) without cloud APIs
- Provide a terminal-first UX (TUI), with future GUI expansion
- Support WSL, Linux, macOS, and Windows

## Quick Start

1. Install Rust (stable) and Cargo.
2. Validate the workspace:

```bash
cargo check
cargo test --workspace
```

3. Run the TUI:

```bash
cargo run -p petit-tui
```

4. One-shot stdin mode:

```bash
echo "Hello, how are you?" | cargo run -p petit-tui -- --stdin --target-lang fr
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

## Model Files

Models are not checked into git. By default, config expects:

- `models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf`

Override with CLI (`--model`), env vars, or config.

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
