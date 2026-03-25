# Product Scope

## In Scope (Current)

- Text-to-text translation in a terminal-first workflow
- Interactive TUI usage and one-shot stdin usage
- User-managed GGUF model files (no built-in download flow yet)
- Language-pair selection and validation for supported languages
- Optional glossary-constrained translation using a local glossary file
- In-process embedding and vector retrieval for glossary candidate selection

## Out of Scope (Current)

- Built-in model download and first-run setup wizard
- Batch/document translation pipelines
- Full GUI implementation
- Concurrent job queueing in v1
- Glossary editing UI
- External vector database integration
- User-selectable embedding models in v1

## Open Questions

### Model Acquisition Experience

- Keep manual model setup, or add:
  - built-in download command
  - guided first-run setup

### Glossary Asset Experience

- Keep embedding-model cache management implicit when glossary support is enabled, or surface
  explicit cache-management commands later

### Language UX

- Keep language-code-first UX, or accept/display language names by default

### Optional User Features

- Translation history
- Clipboard integration
- Model hot-reloading
