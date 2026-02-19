# Product Scope

## In Scope (Current)

- Text-to-text translation in a terminal-first workflow
- Interactive TUI usage and one-shot stdin usage
- User-managed GGUF model files (no built-in download flow yet)
- Language-pair selection and validation for supported languages

## Out of Scope (Current)

- Built-in model download and first-run setup wizard
- Batch/document translation pipelines
- Full GUI implementation
- Concurrent job queueing in v1

## Open Questions

### Model Acquisition Experience

- Keep manual model setup, or add:
  - built-in download command
  - guided first-run setup

### Language UX

- Keep language-code-first UX, or accept/display language names by default

### Optional User Features

- Translation history
- Clipboard integration
- Model hot-reloading
