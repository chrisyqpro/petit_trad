# Implement auto source language translation

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with [docs/PLANS.md](docs/PLANS.md).

## Purpose / Big Picture

After this change, a user can leave the source language at `auto` and let TranslateGemma infer the
source language from the text while still translating into an explicit target language. The prompt
must always tell the model to return only the translation, and glossary-constrained translation must
still work even when the source language is unknown up front. A human can see the feature working by
running stdin mode with `--source-lang auto`, by observing that the TUI starts with `auto` as the
source, and by confirming that glossary prompts still inject selected terms for auto mode.

## Progress

- [x] (2026-04-06 20:15Z) Researched the current prompt, language validation, glossary, and TUI
      surfaces; recorded findings in
      `docs/execution-plans/research/2026-04-06-auto-source-language.md`.
- [x] (2026-04-06 20:15Z) Updated persistent design and product docs for `auto` source mode and the
      shared translation-only instruction; committed as `1dbcfbe`.
- [x] (2026-04-06 20:33Z) Implement `petit-core` support for source `auto`, including prompt
      generation and glossary candidate lookup.
- [x] (2026-04-06 20:33Z) Implement `petit-tui` defaults and UI behavior for source `auto`,
      including safe swap behavior.
- [x] (2026-04-06 20:33Z) Expand tests to cover explicit-source prompts, auto-source prompts,
      glossary behavior, config loading, and TUI edits.
- [x] (2026-04-06 20:42Z) Fix global auto-source glossary ordering so exact matches always outrank
      ANN-only candidates before truncation.
- [x] (2026-04-06 20:44Z) Re-run focused glossary regression tests and full repo verification after
      the auto-source ordering fix. `cargo test -p petit-core glossary --features cpu-only` and
      `./scripts/check.sh --fix` passed after the follow-up fix. CPU-only and Metal checks had
      already passed before this follow-up; model-backed smoke remained blocked by a llama.cpp
      context-creation failure in this environment.
- [x] (2026-04-06 21:05Z) Update `scripts/smoke.sh` so the model-backed stdin smoke uses
      `--src auto` and exercises the shipped auto-source path instead of the old explicit-source
      fallback.
- [x] (2026-04-06 21:40Z) Revert the temporary XDG compatibility workaround after confirming the
      remaining TUI `en` display came from stale user config, not repo defaults. Repo config loading
      is back to normal precedence and `config/default.toml` remains the source of the shipped
      `default_source = "auto"` behavior.

## Surprises & Discoveries

- Observation: The committed code still uses the old `[src->tgt]` prompt shape in
  `crates/petit-core/src/gemma.rs` and still validates both source and target as explicit model
  languages. Evidence: `build_prompt()` and `validate_pair()` in the current tree.
- Observation: Glossary retrieval is coupled to one normalized `(source_lang, target_lang)` key, so
  `auto` mode cannot be implemented by prompt text alone. Evidence:
  `GlossaryStore::select_candidates()` resolves exactly one `lang_pair_key`.
- Observation: TUI swap currently performs a raw `std::mem::swap`, which would incorrectly allow
  `target_lang = "auto"` once `auto` becomes the default source. Evidence:
  `crates/petit-tui/src/app.rs`.
- Observation: Model-backed smoke with the local 27B GGUF failed in this environment with
  `Inference error: Context creation: null reference from llama.cpp`. Evidence: `./scripts/smoke.sh`
  using `SMOKE_MODEL_PATH=models/translategemma-27b-it-GGUF/translategemma-27b-it.Q8_0.gguf`.
- Observation: Auto-source glossary fan-out originally concatenated ranked candidates per language
  pair in source-language sort order, which could let an earlier pair's ANN-only hit outrank a later
  pair's exact hit when `max_matches` truncated the merged list. Evidence:
  `crates/petit-core/src/glossary.rs`. The follow-up fix applies one global ordering pass before
  truncation and now has a regression test covering a later exact hit versus an earlier ANN-only
  hit.
- Observation: The later TUI report showing `Source: en` was caused by a stale user XDG config
  override, not by the repo default. Evidence: the issue disappeared after the stale
  `default_source = "en"` override was removed from the local user config, and the repo-side
  compatibility workaround was then reverted.

## Decision Log

- Decision: Treat `auto` as a reserved source-only sentinel instead of a normal model language code.
  Rationale: The design docs and product requirements say the target must remain explicit.
  Date/Author: 2026-04-06 / Codex
- Decision: Keep one shared translation-only instruction across explicit-source, auto-source, and
  glossary prompt variants, each with a `Text:` section before the input payload. Rationale: This
  matches the approved design docs and keeps prompt behavior consistent. Date/Author: 2026-04-06 /
  Codex
- Decision: For glossary lookup in `auto` mode, search every glossary index that matches the
  requested target language and merge candidates deterministically before prompt injection.
  Rationale: This preserves glossary support without guessing the source language outside the model.
  Date/Author: 2026-04-06 / Codex
- Decision: Block swap from producing `target_lang = "auto"` by keeping the explicit target and
  reporting a clear status instead of silently creating an invalid state. Rationale: Product docs
  require explicit target preservation in swap-style actions. Date/Author: 2026-04-06 / Codex
- Decision: Treat the failed model smoke as an environment blocker rather than a code regression for
  this pass. Rationale: The failure surfaced from llama.cpp context creation after the unit and
  Metal checks passed, and the same script is known to depend on host/runtime support outside the
  pure Rust test matrix. Date/Author: 2026-04-06 / Codex
- Decision: Apply a single global ordering pass for auto-source glossary candidates before
  truncation, with exact candidates sorted ahead of ANN-only candidates across all matching pairs.
  Rationale: This is the only way to make the acceptance rule hold when multiple pairs contribute
  candidates to the same request. Date/Author: 2026-04-06 / Codex
- Decision: Add a regression test that forces `max_matches = 1` with an earlier ANN-only hit and a
  later exact hit, so the global ordering rule is protected against future regressions. Rationale:
  The bug only shows up when truncation happens after merging. Date/Author: 2026-04-06 / Codex
- Decision: Keep config precedence unchanged and revert the temporary XDG-specific workaround once
  the stale user override was identified. Rationale: the shipped behavior should come from
  `config/default.toml` and normal overlays, not from hardcoded special cases for old local config
  values. Date/Author: 2026-04-06 / Codex
- Decision: Make the default smoke harness use `--src auto` for the model-backed stdin check.
  Rationale: the smoke harness should exercise the new default acceptance path instead of silently
  proving only the older explicit-source path. Date/Author: 2026-04-06 / Codex

## Outcomes & Retrospective

The research, design, plan, implementation, review, and follow-up fixes are complete. `petit-core`
now accepts `auto` as a source-only sentinel, uses the approved shared translation-only instruction
with `Text:` markers in every prompt variant, and preserves glossary injection in both explicit and
auto-source modes. `petit-tui` ships with `default_source = "auto"`, keeps `auto` source-only in
config and interactive edits, and preserves explicit-target swap behavior.

The review follow-ups are also complete. Auto-source glossary ranking now applies one global
ordering pass before truncation, the default smoke harness now exercises `--src auto`, and the
temporary repo-side XDG compatibility workaround was reverted once the stale user override was
confirmed as the real cause of the transient `Source: en` display. The repo verification path is
green again after the revert: focused config tests and `./scripts/check.sh --fix` passed. The only
residual limitation remains environmental model-smoke reliability with the local llama.cpp backend;
that is not treated as an open product bug for this feature.

## Context and Orientation

`petit_trad` has two production crates. `crates/petit-core` owns translation behavior, prompt
construction, glossary retrieval, and llama.cpp inference. `crates/petit-tui` owns config loading,
interactive language editing, swap behavior, and stdin mode orchestration. The baseline config lives
in `config/default.toml`.

The key implementation files are:

- `crates/petit-core/src/language.rs`: normalizes language codes, exposes the supported language
  list, and currently rejects any source or target that is not a model language.
- `crates/petit-core/src/gemma.rs`: builds the final TranslateGemma prompt, validates the request,
  asks the glossary store for candidate terms, and runs inference.
- `crates/petit-core/src/glossary.rs`: stores glossary entries by normalized source-target pair,
  ranks exact matches and approximate nearest-neighbor matches, and returns prompt-ready
  `source_term -> target_term` pairs.
- `crates/petit-tui/src/config.rs`: reads config file, environment, and CLI values, applies
  precedence, normalizes languages, and validates the translation request before startup succeeds.
- `crates/petit-tui/src/app.rs`: defines default source and target languages, validates interactive
  language edits, and handles the swap action.
- `crates/petit-tui/src/ui.rs`: shows the footer hints and the current language pair.
- `config/default.toml`: defines the default source and target language values.

The approved persistent docs already define the target behavior in:

- `ARCHITECTURE.md`
- `docs/design-docs/architecture.md`
- `docs/design-docs/prompt-format.md`
- `docs/design-docs/glossary-constraints.md`
- `docs/product-specs/requirements.md`

## Plan of Work

Begin in `crates/petit-core/src/language.rs` by introducing a small, explicit API for the source
sentinel. Add a public constant such as `AUTO_SOURCE_LANG`, a helper that identifies whether a
source value is `auto`, and a validation path that accepts either `auto` or a supported source code
while still requiring `target_lang` to be a supported model language. Keep the existing helpers for
explicit language codes so current callers and tests remain readable.

Next, rework `crates/petit-core/src/gemma.rs` so prompt building is split by two axes: explicit
versus auto source, and glossary versus no glossary. Extract the shared translation-only instruction
into one constant and keep `Text:` in every prompt variant. For explicit source, keep the
`[src->tgt]` header. For auto source, switch to the natural-language header that asks the model to
infer the source language from the text. The `build_prompt_with_lookup()` path must validate the new
request shape before running glossary lookup, and its tests must clearly distinguish the four prompt
variants.

Then extend `crates/petit-core/src/glossary.rs` so candidate selection can operate in two modes.
Explicit-source mode should keep the existing single-pair behavior. Auto-source mode should search
every stored pair whose target language matches the requested target, collect exact and ANN hits per
pair, and merge them with stable ordering before de-duplicating by normalized source term and
truncating to `max_matches`. Reuse the existing ranking logic instead of cloning it in multiple
places. Add tests that prove target-scoped fan-out works, that target mismatches return no
candidate, and that deterministic truncation survives multi-pair merges.

After the core changes are stable, update `crates/petit-tui/src/config.rs`, `config/default.toml`,
and `crates/petit-tui/src/app.rs`. The default source must become `auto`. Config loading and
interactive language edits must accept `auto` only for the source side and continue rejecting it on
the target side. `App::swap_languages()` must not create `target_lang = "auto"`; either keep the
target unchanged and surface a status such as “Cannot swap when source is auto”, or implement the
equivalent explicit-target-preserving behavior chosen during coding, but the final behavior must
match the design docs and tests. If the footer hint text in `crates/petit-tui/src/ui.rs` needs to
change for clarity, update it in the same milestone.

Finally, align tests and verification commands. Keep unit tests narrow and behavior-focused: the new
tests in `petit-core` should fail before the implementation and pass afterward, and the same should
be true for the new config and TUI tests. Once the unit and integration tests pass, run the repo
check scripts and then a model-backed smoke or eval command if a local GGUF model exists.

## Concrete Steps

Work from the repository root:

    cd /Users/dzr/src/repo/petit_trad

Update source-language validation and helpers in `crates/petit-core/src/language.rs`. Then update
prompt construction and prompt tests in `crates/petit-core/src/gemma.rs`, followed by glossary
selection logic and glossary tests in `crates/petit-core/src/glossary.rs`.

After the core work, update defaults and config validation in `config/default.toml` and
`crates/petit-tui/src/config.rs`. Then update TUI editing and swap behavior in
`crates/petit-tui/src/app.rs`, and update footer hints in `crates/petit-tui/src/ui.rs` if needed.

Run focused tests while iterating:

    cargo test -p petit-core gemma --features cpu-only
    cargo test -p petit-core glossary --features cpu-only
    cargo test -p petit-core language --features cpu-only
    cargo test -p petit-tui app --features cpu-only
    cargo test -p petit-tui config --features cpu-only

Run the full repo verification commands:

    ./scripts/check.sh --fix
    ./scripts/check.sh --fix --features metal

If a local GGUF model exists, run a model-backed smoke or eval command to prove runtime behavior:

         SMOKE_MODEL_PATH=/absolute/path/to/model.gguf ./scripts/smoke.sh

    or

         ./scripts/eval.sh --model /absolute/path/to/model.gguf --features metal --gpu-layers 0 \
           --context-size 256 --threads 1

Expected success signals:

- The focused tests fail before the implementation and pass afterward.
- `./scripts/check.sh --fix` and `./scripts/check.sh --fix --features metal` both end with
  `all checks passed`.
- In a model-backed smoke or eval run, the request using `--source-lang auto` completes without a
  language-validation error and produces translated output only.

## Validation and Acceptance

Acceptance is behavioral, not structural.

For prompt formatting, add unit tests in `crates/petit-core/src/gemma.rs` that assert the full
string for:

- explicit source without glossary
- auto source without glossary
- explicit source with glossary
- auto source with glossary

Each prompt must include the shared translation-only instruction and a `Text:` marker before the
payload.

For validation, add tests in `crates/petit-core/src/language.rs`, `crates/petit-tui/src/config.rs`,
and `crates/petit-tui/src/app.rs` proving:

- `auto` is accepted as a source
- `auto` is rejected as a target
- the default config source is `auto`
- TUI source edits can set `auto`
- TUI target edits reject `auto`
- swap does not create `target_lang = "auto"`

For glossary behavior, add tests in `crates/petit-core/src/glossary.rs` and `gemma.rs` proving:

- explicit source still uses the single-pair lookup
- auto source fans out across target-matching source buckets
- exact matches still outrank ANN-only matches in both modes
- de-duplication and `max_matches` truncation remain deterministic

For end-to-end proof, run stdin mode after implementation with an explicit target and auto source:

    echo "Bonjour tout le monde" | cargo run -p petit-tui -- --stdin \
      --source-lang auto --target-lang en

Expect one translated line on stdout and no extra explanation text.

## Idempotence and Recovery

All edits in this plan are source-controlled and additive. Re-running the test commands is safe. If
a focused test fails mid-implementation, keep the failing test in place, fix the implementation, and
rerun the same command until it passes. If a model-backed smoke or eval command is unavailable
because no GGUF file exists locally, record that as a verification gap instead of inventing a path.

If glossary tests become flaky during the auto-source merge work, stop and refactor the ranking code
into one shared helper before continuing. Determinism is a product requirement, not an optimization.

## Artifacts and Notes

Useful current-state evidence:

    git commit --oneline -1
    1dbcfbe docs(prompt): design auto source mode

    ./scripts/check.sh --fix
    ==> all checks passed

The implementation step should update this section with the most relevant passing test transcripts
and any model-backed smoke or eval evidence. Current evidence:

    ./scripts/check.sh --fix
    ==> all checks passed

    ./scripts/check.sh --fix --features metal
    ==> all checks passed

    SMOKE_MODEL_PATH=models/translategemma-27b-it-GGUF/translategemma-27b-it.Q8_0.gguf ./scripts/smoke.sh
    FAIL  stdin translation smoke (exit 1)
    Error: Inference error: Context creation: null reference from llama.cpp

## Interfaces and Dependencies

Keep the implementation inside the existing crates and reuse the current libraries:

- `fastembed` and `hnsw_rs` stay the glossary retrieval mechanism in
  `crates/petit-core/src/glossary.rs`.
- `llama-cpp-2` remains behind `ModelManager`; do not add a second inference path.
- The public `Translator` trait in `crates/petit-core/src/lib.rs` keeps the same method signature.

At the end of implementation, the code should expose a clear source-sentinel interface from
`crates/petit-core/src/language.rs`, prompt builders in `crates/petit-core/src/gemma.rs` that cover
all four prompt variants, glossary selection in `crates/petit-core/src/glossary.rs` that supports
explicit and auto source modes, and `petit-tui` config and app behavior that preserve an explicit
target language at all times.
