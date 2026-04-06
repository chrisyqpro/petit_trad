# Product Requirements

This document describes only user-visible requirements.

## Functional Requirements

- Users can translate text between supported language pairs.
- Users can run translation in:
  - interactive TUI mode
  - one-shot stdin mode for scripts
- Users can select source and target language.
- Users can set the source language to `auto` so the model infers the source from the input text.
- The application validates unsupported language codes and returns clear errors.
- Users can optionally enable glossary-constrained translation.
- When glossary-constrained translation is enabled, the product uses configured glossary terms to
  improve target-term consistency when the source text matches glossary entries.
- Glossary-constrained translation remains available when source language is `auto`.
- Invalid glossary configuration must produce a clear startup error instead of a silent fallback to
  unconstrained translation.
- If the source language is `auto`, the product must not turn the target language into `auto` via
  swap-style UI actions; it must preserve an explicit target or report a clear status instead.

## Configuration Requirements

- Users can configure runtime behavior by CLI flags, environment variables, and config file values.
- Precedence must be deterministic: `CLI args > environment variables > config file > defaults`.
- Users can override the model path from defaults.
- The default source language is `auto`.
- Users can enable or disable glossary retrieval through the same deterministic precedence system.
- Users can configure the glossary file path and glossary embedding model directory.

## Operational Requirements

- Inference must remain local-only (no cloud API dependency).
- Product must support WSL, Linux, macOS, and Windows.
- Product must support CPU operation and optional GPU acceleration where available on the host
  platform.
- Glossary retrieval must remain local-only and run in-process with the translator.
