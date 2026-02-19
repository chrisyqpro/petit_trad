# Architecture

This document is the bird's-eye view of how the codebase is structured.
It describes stable component boundaries and data flow.

This document is not a roadmap, task tracker, or feature backlog.

## Top-Level Shape

`petit_trad` has two production crates and one prototype area:

- `crates/petit-core`: translation engine and runtime integration
- `crates/petit-tui`: terminal application and interaction layer
- `proto/`: experiments used to validate model/prompt behavior

Runtime assets:

- `config/default.toml`: baseline settings
- `models/` (user-managed): local GGUF model files

## Component Responsibilities

### `petit-core`

Owns translation logic and model runtime concerns:

- config and error types
- language normalization/validation
- prompt construction for TranslateGemma
- model loading and token generation through `llama-cpp-2`

`petit-core` is the backend boundary. Frontends should use its public API
(`Translator`, `Config`, `Error`) instead of reimplementing inference logic.

### `petit-tui`

Owns terminal UX and request orchestration:

- terminal lifecycle and render loop
- input/state management and status feedback
- config loading and precedence application
- worker-thread execution for translation requests

`petit-tui` depends on `petit-core`; the inverse dependency is not allowed.

## Translation Flow

1. `petit-tui` gathers text/language/config from CLI, env, config file, and UI.
2. A worker thread owns a `GemmaTranslator` instance from `petit-core`.
3. `GemmaTranslator` validates language pair, builds prompt, calls inference.
4. `ModelManager` performs tokenization, decode loop, sampling, and detokenization.
5. Result text returns to `petit-tui` for display or stdout output.

## Architectural Invariants

- Inference is local-only (no cloud translation API path).
- Backend/model integration stays inside `petit-core`.
- UI code stays inside frontend crates (`petit-tui`, future frontends).
- Config precedence is `CLI > env > config file > defaults`.
- Prompt format for TranslateGemma follows `docs/design-docs/prompt-format.md`.

## Deeper Reading

- `docs/design-docs/index.md` for tech design docs
- `docs/product-specs/index.md` for product spec docs
