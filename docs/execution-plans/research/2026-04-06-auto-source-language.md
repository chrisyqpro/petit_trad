# Research: auto source language prompt enhancement

## Prompted requirement

Add an `auto` source language option and make it the default. When source is `auto`, prompt
TranslateGemma to infer the source language from the text itself and return only the translation
with no explanation, notes, quotes, or extra formatting. When source is explicit, keep the existing
source-to-target prompt behavior. Glossary injection must continue to work in both cases.

## Findings

### Current prompt design is explicit-pair based in both code and persistent docs

- `crates/petit-core/src/gemma.rs` builds two prompt shapes today:
  - plain translation:
    `<start_of_turn>user\n[{src}->{tgt}] {text}<end_of_turn>\n<start_of_turn>model\n`
  - glossary-assisted translation: same pair header plus glossary block before `Text:`
- `docs/design-docs/prompt-format.md` and `docs/design-docs/glossary-constraints.md` both document
  the same explicit `[src->tgt]` direct format and glossary extension.
- `ARCHITECTURE.md` says prompt format follows `docs/design-docs/prompt-format.md`, so prompt-shape
  changes need a persistent design-doc update rather than a code-only patch.

### `auto` is not accepted anywhere in the current language pipeline

- `crates/petit-core/src/language.rs` validates pairs by checking both source and target against the
  TranslateGemma supported-language list. `auto` is currently unsupported.
- `crates/petit-tui/src/config.rs` normalizes the configured languages, then calls
  `validate_pair(&source_lang, &target_lang)` before startup succeeds.
- `crates/petit-tui/src/app.rs` uses the same pair validation for in-TUI language edits.
- Current defaults are still explicit source:
  - `config/default.toml`: `translation.default_source = "en"`
  - `crates/petit-tui/src/app.rs`: `App::default().source_lang = "en"`

Result: changing prompt text alone is insufficient. `auto` must become a first-class accepted source
value across config loading, interactive edits, and defaults.

### Glossary lookup is coupled to a known source-target pair

- `GemmaTranslator::translate()` calls `build_prompt_with_lookup()`, which validates the pair before
  glossary lookup.
- `GlossaryStore::select_candidates(source_lang, target_lang, source_text)` retrieves candidates
  from a per-language-pair index keyed by normalized `(source_lang, target_lang)`.
- `crates/petit-core/src/glossary.rs` currently builds one HNSW index per normalized pair and
  shortlists candidates only from that exact pair bucket.

This means the requirement "glossary injection should work in both cases" is a real design change:
with `source = auto`, the current glossary store has no source pair key to query. Preserving
glossary behavior in auto mode requires a retrieval-path extension, not only a prompt update.

### TUI behavior needs a decision for `auto`

- The header and edit flow already render raw language codes, so showing `auto` is mechanically
  fine.
- `App::swap_languages()` currently swaps source and target blindly. If source defaults to `auto`,
  swapping would produce `target_lang = "auto"`, which does not match the requested semantics.
- Help text in `crates/petit-tui/src/ui.rs` currently advertises swap as a generic action, with no
  special handling for sentinel values.

The design needs an explicit rule for swap behavior once `auto` is the default source.

### Existing tests cover the current seams and can anchor the enhancement

- `crates/petit-core/src/gemma.rs` already has prompt-format tests with and without glossary terms.
- `crates/petit-core/src/glossary.rs` already tests pair partitioning and deterministic candidate
  selection.
- `crates/petit-tui/src/config.rs` covers config precedence and default loading.
- `crates/petit-tui/src/app.rs` covers invalid language edits and app state transitions.

Targeted verification against current HEAD passed:

- `cargo test -p petit-core gemma --features cpu-only`
- `cargo test -p petit-tui config --features cpu-only`

## Design implications

This should be handled as an enhancement to the existing prompt and language-selection design, not
as a localized execution-only change.

The main design questions to resolve next are:

1. How `auto` is represented in validation and API boundaries.
2. How glossary retrieval works when the source language is unknown at request time.
3. What exact prompt templates and shared "translation-only" instructions are used for:
   - explicit source without glossary
   - explicit source with glossary
   - auto source without glossary
   - auto source with glossary
4. What `swap languages` should do when source is `auto`.

## Need new design or enhance existing ones?

Yes. This is an enhancement to the existing prompt-format and language-selection design, and it also
touches glossary retrieval behavior when the source language is not known up front.
