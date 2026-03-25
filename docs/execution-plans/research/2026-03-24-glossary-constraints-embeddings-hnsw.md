# Research: glossary constraint feature with Rust embeddings and HNSW

## Prompted requirement

Implement a glossary constraint feature for `petit_trad` using a Rust embeddings library and a
lightweight HNSW vector index. The goal is not only the user feature itself, but also to gain
hands-on experience with a simple local embedding and vector-search stack inside this translator
app.

## Current project state relevant to this enhancement

`petit_trad` currently has a very small translation path:

- `petit-tui` resolves config and request state.
- `GemmaTranslator` in `crates/petit-core/src/gemma.rs` validates languages, builds one direct
  TranslateGemma prompt, and sends it to llama.cpp.
- `ModelManager` in `crates/petit-core/src/model_manager.rs` owns model loading and synchronous
  inference.

Important current constraints from persistent docs and code:

- Translation stays local-only.
- Backend/model integration stays in `petit-core`.
- Config precedence must stay deterministic: `CLI > env > config file > defaults`.
- Prompt format is deliberately simple and documented in `docs/design-docs/prompt-format.md`.
- The product does not currently have any built-in model-download flow.

There is no existing glossary system, retrieval system, secondary model runtime, or vector storage
abstraction in the repo. The new feature would therefore add:

- a new local glossary data model
- a second local model/runtime path for embeddings
- a vector index build/load/search path
- prompt augmentation rules
- new config and UX surface

## Where the feature fits in the current architecture

The cleanest boundary is inside `petit-core`, not `petit-tui`.

Reasons:

- `ARCHITECTURE.md` says backend/model integration belongs in `petit-core`.
- The glossary decision affects translation semantics, not just UI.
- stdin mode, benchmark mode, and TUI mode should all use the same glossary behavior.

The likely insertion points are:

- `crates/petit-core/src/config.rs` Add glossary-related core config.
- `crates/petit-core/src/gemma.rs` Extend prompt building to inject retrieved glossary constraints.
- `crates/petit-core/src/lib.rs` Re-export any new glossary/index types if needed.
- new `crates/petit-core/src/glossary.rs` Own glossary loading, embedding, index build, retrieval,
  and prompt-context formatting.
- `crates/petit-tui/src/config.rs` Extend file/env/CLI precedence into the new core config fields.
- `crates/petit-tui/src/cli.rs` Add user-visible flags only if the first iteration needs runtime
  overrides.
- `config/default.toml` Add disabled-by-default baseline glossary settings.

## Why glossary retrieval is not just string matching

If the goal were only strict term replacement, exact string matching would be simpler than
embeddings. That is not what this task is optimizing for. The stated goal is to learn a simple
embeddings + vector-index flow inside the app, so the glossary feature should intentionally exercise
that stack.

That said, relying on vector search alone would be weak for glossary constraints:

- a full sentence embedding may miss short source terms
- ANN similarity alone can return semantically related but non-binding entries
- glossary enforcement needs predictable behavior

The best first iteration is therefore hybrid:

1. Exact or normalized substring checks remain a high-confidence signal.
2. Embedding + HNSW retrieval generates candidate glossary entries for the current input.
3. The prompt receives a small, deterministic shortlist of terms to preserve.

This uses embeddings/vector search for learning, while keeping enough guard rails for a terminology
feature.

## Embeddings library research

### Candidate: `fastembed`

This is the strongest fit for a first implementation.

Verified from primary sources on 2026-03-24:

- `fastembed` docs.rs latest page shows crate version `5.13.0`.
- The crate exposes a synchronous `TextEmbedding` API.
- `InitOptions` includes `model_name`, `execution_providers`, `cache_dir`, `show_download_progress`,
  and `max_length`.
- The project README states there is no Tokio dependency and that it uses ONNX Runtime plus Hugging
  Face tokenizers.
- The supported model list includes:
  - `MultilingualE5Small`
  - `MultilingualE5Base`
  - `MultilingualE5Large`
  - `BGEM3`
  - `EmbeddingGemma300M`

Why it fits this repo:

- The rest of `petit_trad` is synchronous in `petit-core`; `fastembed` matches that directly.
- It already exposes model caching, which matters because this project currently has no separate
  asset manager for embedding models.
- It supports multilingual models, which matters for a translator app more than English-only
  retrieval models.
- It is much less work than wiring raw `candle` or `ort` primitives directly.

### Model choice for first iteration

Three realistic choices stand out:

1. `MultilingualE5Small`
2. `EmbeddingGemma300M`
3. `BGEM3`

Observations from primary sources:

- `fastembed` supports all three models.
- The official `intfloat/multilingual-e5-small` model card says each input should start with
  `query:` or `passage:`, even for non-English text.
- Google’s EmbeddingGemma documentation describes it as a multilingual text embedding model trained
  in over 100 languages.
- The official `BAAI/bge-m3` model card says it supports more than 100 working languages and does
  not require adding instructions to queries for dense embedding retrieval.

Corrective note:

- `EmbeddingGemma300M` should not be treated as non-multilingual. Based on the official Google
  documentation, it is explicitly multilingual.

Revised recommendation:

- `MultilingualE5Small` and `EmbeddingGemma300M` are both credible first choices.
- `BGEM3` remains a stronger but heavier follow-up option if the smaller models show poor recall.

Tradeoff framing:

- Prefer `MultilingualE5Small` if the goal is to start from a widely used, retrieval-specific
  baseline with a simple documented query/passages convention.
- Prefer `EmbeddingGemma300M` if the goal is tighter alignment with the Gemma family already used in
  this repo, plus a small on-device-oriented multilingual model from Google.

Current evidence does not justify a strong claim that `MultilingualE5Small` is categorically safer
because of multilinguality. If we keep `MultilingualE5Small` as the first pick, that should be
framed as an implementation preference, not a claim that EmbeddingGemma lacks multilingual support.

## HNSW library research

### Candidate: `hnsw_rs`

This is the strongest fit for a lightweight in-process vector index.

Verified from primary sources on 2026-03-24:

- `hnsw_rs` docs.rs latest page shows crate version `0.3.4`.
- The crate README states it implements HNSW and supports multithreaded insertion and search.
- It supports dump/reload via `hnswio`.
- It supports filtered search.
- The constructor is: `Hnsw::new(max_nb_connection, max_elements, max_layer, ef_construction, f)`.
- Search is: `search(data, knbn, ef_arg)`.

Why it fits this repo:

- It is embedded, local, and small in conceptual surface area.
- It can stay fully inside `petit-core` without adding a service dependency.
- It is enough to learn the main vector-db ideas the enhancement is targeting: vector generation,
  indexing, approximate nearest-neighbor search, metadata filtering, and persistence.

### Recommended index shape

Do not introduce a general database abstraction in the first iteration.

Use a project-local store shaped roughly like:

- glossary entries loaded from a local file
- source-text embedding per entry
- in-memory `Hnsw<f32, _>` per language pair
- optional dump/reload cache later if startup cost becomes a problem

Why per-language-pair indices are better than one global index for v1:

- the current translator request already has one source/target pair
- it avoids unnecessary metadata-filter complexity in the first design
- it keeps each index smaller
- it reduces false positives across unrelated language pairs

The global-index-plus-filter approach is possible later because `hnsw_rs` supports filtered search,
but it is not the simplest first cut.

## Recommended feature behavior for a first iteration

### Glossary data model

Keep the glossary file format intentionally small and explicit. TSV or TOML are both reasonable. TSV
is easier to author and inspect for terminology lists.

Minimal fields:

- `source_lang`
- `target_lang`
- `source_term`
- `target_term`
- `note` (optional)

Optional future fields:

- `case_sensitive`
- `domain`
- `priority`
- `alternate_source`
- `alternate_target`

### Retrieval behavior

For a given translation request:

1. Normalize the input language pair.
2. Select the index for that pair.
3. Create a query embedding from the full source text.
4. Retrieve top `k` glossary candidates from HNSW.
5. Promote any exact normalized source-term matches above ANN-only matches.
6. Apply a similarity threshold.
7. Deduplicate and cap the shortlist.
8. Inject the shortlist into the translation prompt.

This should stay deterministic:

- fixed embedding model
- fixed normalization rules
- fixed `k`
- fixed similarity threshold
- fixed result ordering rules

### Prompt strategy

Do not replace the current direct TranslateGemma format with a large instruction template. Keep the
existing structure but prepend a compact glossary block inside the user turn.

Example shape:

```text
<start_of_turn>user
[en->fr]
Use the following glossary entries exactly when applicable:
- account balance -> solde du compte
- savings account -> compte d'epargne

Text:
Your balance is available in the savings account.<end_of_turn>
<start_of_turn>model
```

This is still consistent with the current “single user turn, single model turn” prompt design, but
it means `docs/design-docs/prompt-format.md` must be updated during the design step.

## Important risks and constraints

### 1. Startup latency will increase

`petit-tui` already initializes the translator worker at startup. If glossary loading also
initializes the embedding model and builds the HNSW index during `GemmaTranslator::new`, the startup
path will become more expensive.

This is acceptable for the first design as long as it is explicit, but it should be called out:

- current footer says `Initializing translator...`
- glossary-enabled startup may need more descriptive status text
- benchmark mode startup numbers will change

### 2. First-run model download changes product behavior

`fastembed` supports model caching, but that implies a download path if the embedding model is not
already present. This is a meaningful product change because the current app expects user-managed
local assets and has no built-in model-download UX.

This needs a design decision:

- either allow lazy download into a configured cache directory
- or require a local embedding model cache to be pre-populated/documented

### 3. ANN retrieval alone is not strong enough for terminology guarantees

Glossary features imply higher precision than general semantic retrieval. This is why exact-match
promotion and a strict shortlist are important. Without those guard rails, the feature can degrade
translation quality rather than improve it.

### 4. Whole-input embeddings are imperfect for short terms

A sentence embedding over a long input may dilute a short glossary term. A future enhancement may
need per-span or per-sentence querying, but that should not block v1. For the first cut, exact-match
promotion is the right mitigation.

### 5. Prompt injection budget must stay small

The current translation path is intentionally lean. Dumping too many retrieved entries into the
prompt will consume context window and reduce output quality. The glossary shortlist should stay
small, likely in the `3-8` range.

## Verification impact

The current verification setup is not enough by itself for this feature.

Needed additions in a future plan:

- unit tests for glossary file parsing
- unit tests for language-pair partitioning
- unit tests for prompt rendering with glossary entries
- deterministic tests for candidate ordering, thresholding, and exact-match promotion
- optional integration coverage for translation behavior when a local model is available

The existing eval harness can still help later, but it should not be the only test surface because
retrieval and prompt-augmentation logic can be tested without any local GGUF translation model.

## Recommended first implementation scope

To keep the feature genuinely small and educational:

- glossary support only in `petit-core`
- one local glossary file
- one embedding model choice, configured but with a documented default
- one in-memory HNSW index per language pair
- one prompt augmentation strategy
- no background incremental updates
- no separate vector-db service
- no reranker
- no attempt to edit or post-process model output after translation

This keeps the work aligned with the stated learning goal: understand embeddings and ANN indexing in
a real app without over-building infrastructure.

## Recommendation summary

Recommended stack for v1:

- embeddings library: `fastembed`
- default embedding model: `MultilingualE5Small`
- vector index: `hnsw_rs`
- retrieval style: hybrid exact-match promotion plus ANN shortlist
- storage model: per-language-pair in-memory HNSW indices in `petit-core`
- prompt strategy: compact glossary block injected into the existing user turn

This is the lowest-friction path that still exercises the concepts the feature is supposed to teach:

- embedding generation
- local model caching
- vector indexing
- approximate nearest-neighbor search
- retrieval-to-prompt integration

## Sources

- [fastembed docs.rs crate page](https://docs.rs/fastembed/latest/fastembed/)
- [fastembed `InitOptions` docs](https://docs.rs/fastembed/latest/fastembed/type.InitOptions.html)
- [fastembed `EmbeddingModel` docs](https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html)
- [fastembed GitHub README](https://github.com/Anush008/fastembed-rs)
- [hnsw_rs docs.rs crate page](https://docs.rs/hnsw_rs/latest/hnsw_rs/)
- [hnsw_rs `Hnsw` API docs](https://docs.rs/hnsw_rs/latest/hnsw_rs/hnsw/struct.Hnsw.html)
- [hnswlib-rs GitHub README](https://github.com/jean-pierreBoth/hnswlib-rs)
- [official `intfloat/multilingual-e5-small` model card](https://huggingface.co/intfloat/multilingual-e5-small)
- [official `BAAI/bge-m3` model card](https://huggingface.co/BAAI/bge-m3)

## Need design/doc update next?

Yes.

This feature changes prompt construction, core runtime responsibilities, config surface, startup
behavior, and likely product documentation around local asset management. It should move to a design
step before planning implementation.
