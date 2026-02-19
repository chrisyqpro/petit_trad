# Product Requirements

This document describes only user-visible requirements.

## Functional Requirements

- Users can translate text between supported language pairs.
- Users can run translation in:
  - interactive TUI mode
  - one-shot stdin mode for scripts
- Users can select source and target language.
- The application validates unsupported language codes and returns clear errors.

## Configuration Requirements

- Users can configure runtime behavior by CLI flags, environment variables, and
  config file values.
- Precedence must be deterministic:
  `CLI args > environment variables > config file > defaults`.
- Users can override the model path from defaults.

## Operational Requirements

- Inference must remain local-only (no cloud API dependency).
- Product must support WSL, Linux, macOS, and Windows.
- Product must support CPU operation and optional GPU acceleration where
  available on the host platform.
