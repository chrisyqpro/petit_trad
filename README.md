petit_trad
===========

A local translation tool that runs Google's TranslateGemma models locally.

Goals
- Run TranslateGemma locally (4B / 12B / 27B) without cloud APIs.
- Terminal-first UX (TUI) with future GUI planned.
- Cross-platform support: WSL (primary dev), Linux, macOS, Windows.

Quick start (development)

1. Install Rust and Cargo.
2. Build workspace:

```bash
cargo check
```

3. Run the TUI (placeholder binary until UI is implemented):

```bash
cargo run -p petit-tui --release
```

GPU backends

GPU support is enabled at build time via Cargo features:

- CUDA (WSL/Linux NVIDIA): `cargo run -p petit-tui --features cuda`
- Metal (macOS Apple Silicon): `cargo run -p petit-tui --features metal`
- Vulkan (Linux AMD): `cargo run -p petit-tui --features vulkan`
- CPU-only: omit GPU features or use `--features cpu-only`

Python prototype

A Python prototype is provided under `proto/` with dependencies listed in
`proto/requirements.txt`. Use it to validate TranslateGemma prompt format and
translation quality before implementing the Rust inference backend.

Models

By default the project expects a GGUF model in `models/`.
Default (local): `translategemma-12b-it.Q8_0.gguf`.
Alternative sizes: `translategemma-4b-it.*` and `translategemma-27b-it.*`.

Docs and agent files

- Permanent docs: `doc/`
- Agent runtime files and session data (git-ignored): `.agent/`

Contributing

See `doc/architecture.md` and `.agent/plan.md` for design and current plan.

License

This project is licensed under GPLv3. See `LICENSE` for details.
