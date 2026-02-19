# Build Guide

This guide covers local development setup for `petit_trad`.

## Prerequisites

- Rust toolchain (stable)
- C/C++ build tools
- Git

Optional by backend:

- CUDA toolkit for `--features cuda`
- Xcode Command Line Tools for `--features metal` on macOS

## Quick Checks

```bash
cargo check
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
- User config path is determined by `directories::ProjectDirs` in `petit-tui`
- Model files are expected under `models/` unless overridden by CLI or env vars

## WSL CUDA Notes

When building with CUDA in WSL, these vars are often required:

```bash
export CUDACXX=/usr/local/cuda/bin/nvcc
export CUDAToolkit_ROOT=/usr/local/cuda
```
