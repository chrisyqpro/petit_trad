# Glossary Constraints

This document defines the durable design for glossary-constrained translation in `petit_trad`.

The feature adds a local glossary retrieval path to improve terminology consistency while also
serving as a deliberate learning vehicle for embeddings and vector search inside the existing
translator architecture.

## Goals

- Keep translation local-only.
- Keep glossary behavior shared across TUI, stdin mode, and future frontends.
- Use a Rust embeddings library and an in-process HNSW index.
- Preserve deterministic config precedence and deterministic retrieval behavior.
- Keep the first implementation small enough to understand end-to-end.

## Non-Goals

- No external vector database service.
- No reranker or multi-stage retrieval pipeline.
- No output post-processing or hard token-level constraint decoding.
- No incremental glossary editing UI in v1.
- No generic terminology platform beyond translation glossary support.

## Chosen Stack

### Embeddings

- Library: `fastembed`
- Model: `EmbeddingGemma300M`

`EmbeddingGemma300M` is the required embedding model for this feature. The design chooses it to stay
aligned with the Gemma family already used for translation while still using a compact multilingual
embedding model.

### Vector Index

- Library: `hnsw_rs`
- Shape: one in-memory HNSW index per normalized language pair

This keeps the design small and avoids building a general-purpose storage layer.

## User-Facing Behavior

When glossary support is enabled and a glossary file is configured:

1. The translator loads glossary entries for normalized language pairs.
2. The translator builds the per-pair HNSW indices during translator initialization.
3. Each translation request retrieves glossary candidates for the current input.
4. The prompt is augmented with a compact glossary block.
5. TranslateGemma is asked to use the provided target terms when applicable.

When glossary support is disabled, or when no candidates are found for the current request,
translation behavior remains the same as today.

The translator also supports a source-language sentinel value `auto`. In that mode, the model is
asked to infer the source language from the request text, but glossary retrieval must still stay
available.

## Configuration Design

Glossary configuration belongs in `petit-core::Config` and follows the same precedence as the rest
of the application:

```text
CLI args > environment variables > config file > defaults
```

V1 adds a new glossary section conceptually shaped like:

```toml
[glossary]
enabled = false
path = "config/glossary.tsv"
embedding_model_dir = "models/embeddinggemma-300m-ONNX"
max_matches = 6
```

Rules:

- `enabled = false` means no glossary initialization work is done.
- If `enabled = true`, `path` is required.
- `embedding_model_dir` points to a user-managed local EmbeddingGemma model directory under
  `models/`.
- `max_matches` caps prompt injection size and is deterministic.

V1 intentionally does not expose model selection in config. The embedding model is fixed to
`EmbeddingGemma300M`, but the local asset directory is explicit and user-managed.

## Glossary File Format

The glossary source of truth is a UTF-8 TSV file.

Required columns:

- `source_lang`
- `target_lang`
- `source_term`
- `target_term`

Optional column:

- `note`

V1 rules:

- One glossary entry maps exactly one source term to one target term.
- Terms are matched against normalized language pairs.
- Empty required fields are invalid.
- Duplicate rows are allowed in the file but are deduplicated at load time by normalized pair plus
  normalized source term plus target term.
- `note` is metadata only and is not included in the prompt in v1.

TSV is chosen over TOML for the glossary payload because it is easy to author, diff, and bulk-edit
as a terminology list.

## Runtime Architecture

### New `petit-core` Subsystem

`petit-core` gains a glossary subsystem responsible for:

- parsing and validating glossary TSV input
- normalizing language pairs and source terms
- creating embeddings with `fastembed`
- building per-pair HNSW indices
- retrieving glossary candidates for a source input
- formatting prompt-ready glossary context

Expected module boundary:

- `crates/petit-core/src/glossary.rs`

### Data Model

Conceptual internal types:

```rust
struct GlossaryConfig {
    enabled: bool,
    path: PathBuf,
    embedding_model_dir: PathBuf,
    max_matches: usize,
}

struct GlossaryEntry {
    source_lang: String,
    target_lang: String,
    source_term: String,
    target_term: String,
    note: Option<String>,
}

struct GlossaryStore {
    entries_by_pair: HashMap<LangPair, Vec<GlossaryEntry>>,
    index_by_pair: HashMap<LangPair, PairGlossaryIndex>,
}
```

`PairGlossaryIndex` owns:

- the normalized entries for that pair
- the source-term embeddings
- the HNSW index

### Initialization Flow

Glossary initialization happens during translator construction, alongside model startup, but only
when glossary support is enabled.

Initialization sequence:

1. Load glossary config.
2. Read and validate TSV rows.
3. Normalize language pairs and deduplicate entries.
4. Create a `fastembed` text embedding client pinned to `EmbeddingGemma300M`.
5. Embed each `source_term`.
6. Build one HNSW index per language pair.

Failure policy:

- Invalid glossary config or unreadable glossary files are startup errors.
- Glossary-enabled startup must fail fast rather than silently falling back to unconstrained
  translation.

This is stricter than "best effort" retrieval because terminology consistency is not a feature that
should quietly disappear.

### Embedding Model Asset Behavior

The EmbeddingGemma asset is treated like the TranslateGemma GGUF asset:

- the user manages the local files under `models/`
- glossary config points at that local directory explicitly
- glossary startup must fail fast if required files are missing
- glossary startup must not download model assets automatically

### Retrieval Flow

For each translation request:

1. Normalize `source_lang` and `target_lang`.
2. Resolve the glossary search scope:
   - explicit source: the pair-specific index for `(source_lang, target_lang)`
   - `auto` source: every pair index whose target language matches `target_lang`
3. Embed the full source input with `EmbeddingGemma300M`.
4. Query the relevant HNSW index or indices for a fixed number of nearest candidates.
5. Score exact normalized substring matches separately.
6. Merge results with exact matches ranked ahead of ANN-only matches.
7. Deduplicate by normalized source term.
8. Truncate to `max_matches`.

If no candidates survive, the translator uses the plain prompt path.

For `auto` source mode, the merge across source-language buckets must stay deterministic. Ties are
broken by the same stable ordering rules used in explicit-pair mode, with language-pair ordering
applied before final truncation when needed.

### Exact-Match Promotion

ANN retrieval alone is not precise enough for terminology constraints. V1 uses a hybrid ranking
policy:

- Exact normalized substring match is highest priority.
- ANN similarity is used to discover additional plausible candidates.
- ANN-only candidates are included only if they pass a fixed internal threshold.

The threshold is an implementation constant in `petit-core`, not a user-facing configuration knob in
v1.

### Determinism Requirements

Glossary retrieval must be deterministic for the same:

- glossary file contents
- normalized language pair
- source input
- glossary configuration
- embedding model version

To preserve determinism:

- use one fixed embedding model
- use one fixed retrieval order
- keep prompt candidate cap small
- do not randomize retrieval or prompt formatting

## Prompt Design

The base TranslateGemma direct format stays intact conceptually, but all prompt variants share the
same translation-only output instruction:

```text
Return only the translation of source text.
Do not explain the source language.
Do not add notes, quotes, or extra formatting.
```

The glossary feature extends the user turn with a compact glossary block.

### No Glossary Candidates with Explicit Source

```text
<start_of_turn>user
[en->fr]
Return only the translation of source text.
Do not explain the source language.
Do not add notes, quotes, or extra formatting.

Text:
Hello, how are you?<end_of_turn>
<start_of_turn>model
```

### No Glossary Candidates with Auto Source

```text
<start_of_turn>user
Translate the text below into fr.
Infer the source language from the text itself.
Return only the translation of source text.
Do not explain the source language.
Do not add notes, quotes, or extra formatting.

Text:
Hello, how are you?<end_of_turn>
<start_of_turn>model
```

### With Glossary Candidates and Explicit Source

```text
<start_of_turn>user
[en->fr]
Return only the translation of source text.
Do not explain the source language.
Do not add notes, quotes, or extra formatting.
Use the glossary terms exactly when they match the source text:
- account balance -> solde du compte
- savings account -> compte d'epargne

Text:
Your balance is available in the savings account.<end_of_turn>
<start_of_turn>model
```

### With Glossary Candidates and Auto Source

```text
<start_of_turn>user
Translate the text below into fr.
Infer the source language from the text itself.
Return only the translation of source text.
Do not explain the source language.
Do not add notes, quotes, or extra formatting.
Use the glossary terms exactly when they match the source text:
- account balance -> solde du compte
- savings account -> compte d'epargne

Text:
Your balance is available in the savings account.<end_of_turn>
<start_of_turn>model
```

Rules:

- The glossary block appears only when at least one candidate is selected.
- Only `source_term -> target_term` pairs are injected.
- `note` is not injected in v1.
- The glossary block is emitted before the `Text:` payload.
- The shared translation-only instruction appears in every prompt variant.
- Output format remains direct translation only.

This preserves the current one-user-turn prompt style while giving the model a clear terminology
hint.

## Error Handling

New glossary-related errors belong in `petit-core::Error`.

Expected categories:

- invalid glossary configuration
- glossary file read failure
- glossary parse failure
- embedding model initialization failure
- glossary embedding generation failure
- glossary index build failure

These errors should surface during startup with explicit messages, especially in the TUI
worker-initialization status path.

## Frontend Impact

`petit-tui` does not own glossary logic. It only:

- exposes glossary config through the existing precedence system
- reports glossary initialization failures through existing startup status surfaces

V1 does not add glossary editing controls to the TUI.

## Verification Requirements

The feature requires deterministic non-model tests in addition to the existing translation
harnesses.

V1 test surface:

- glossary TSV parsing and validation
- pair normalization and partitioning
- deduplication rules
- exact-match promotion
- prompt formatting with and without glossary candidates
- startup failure behavior for invalid glossary config

Model-backed regression tests may be added later, but the glossary subsystem must be testable
without requiring a local TranslateGemma GGUF file.

## Deferred Work

Explicitly deferred from v1:

- glossary-specific TUI screens
- note-aware prompt formatting
- domain or priority fields
- global index plus metadata filtering
- persisted HNSW dump/reload cache
- span-level source segmentation before retrieval
- user-selectable embedding models
