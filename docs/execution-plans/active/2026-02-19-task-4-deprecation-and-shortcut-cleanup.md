# Task 4: Deprecation and Shortcut Cleanup

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this work, `petit-core` no longer depends on a deprecated llama.cpp token
conversion API, and `petit-tui` has an explicit, test-backed translation
shortcut policy that can be validated across terminals. The visible effect is a
cleaner build signal and predictable translation triggering when users press
`Ctrl+Enter` or fallback keys. This task also aligns workspace manifests with
Rust edition 2024 where project metadata still targets 2021.

## Progress

- [x] (2026-02-19 10:35Z) Created ExecPlan with scope, milestones, and
      validation strategy.
- [ ] (2026-02-19 10:35Z) Replace deprecated token detokenization call in
      `crates/petit-core/src/model_manager.rs` and confirm clean compile.
- [ ] (2026-02-19 10:35Z) Refactor TUI translation shortcut matching into a
      dedicated helper in `crates/petit-tui/src/main.rs`.
- [ ] (2026-02-19 10:35Z) Add focused tests for shortcut matching behavior and
      remove the unresolved TODO comment.
- [ ] (2026-02-19 10:35Z) Run full validation commands and capture evidence in
      this document.
- [ ] (2026-02-19 23:20Z) Align Cargo manifest edition settings from 2021 to
  2024 and validate workspace build/tests after the change.

## Surprises & Discoveries

- Observation: `cargo check` and `cargo test --workspace` both pass now, but
  deprecation warnings are emitted from
  `crates/petit-core/src/model_manager.rs:126`.
  Evidence: Warning text points to `LlamaModel::token_to_str` and recommends
  `token_to_piece`.

- Observation: The active TODO in `crates/petit-tui/src/main.rs:140` indicates
  key handling uncertainty for `Ctrl+Enter` across terminals.
  Evidence: Comment says to keep fallbacks until behavior is confirmed.

## Decision Log

- Decision: Keep fallback translation triggers (`Ctrl+M` and `F5`) while making
  matching logic explicit and testable.
  Rationale: Terminal key encodings differ across platforms and emulators, so
  conservative compatibility is safer than narrowing behavior.
  Date/Author: 2026-02-19 / codex.

- Decision: Require one warning-strict check (`RUSTFLAGS="-D warnings"`) as
  part of acceptance.
  Rationale: It proves the deprecation cleanup is real and prevents regression.
  Date/Author: 2026-02-19 / codex.

## Outcomes & Retrospective

In progress. On completion, summarize:

- Which APIs were changed and why.
- Whether shortcut behavior differed on tested terminals.
- Any residual risk that should be tracked in a future plan.

## Context and Orientation

This task is a maintenance cleanup with two independent changes.

An additional repository consistency item is now in scope: Rust edition
alignment. Current project context indicates edition 2024 should be used, while
some manifests may still specify or inherit 2021. This work must update the
edition settings in relevant Cargo manifests and verify no behavioral regression
in build or tests.

The first change is in `crates/petit-core/src/model_manager.rs`, inside
`ModelManager::infer`. The file currently converts generated tokens to text with
`LlamaModel::token_to_str`, which is deprecated in the installed `llama_cpp_2`
crate. "Detokenization" in this repository means converting model token IDs
back into displayable UTF-8 text.

The second change is in `crates/petit-tui/src/main.rs`, inside
`handle_key_event`. Translation is currently triggered by a compound condition
that includes `Ctrl+Enter`, `Ctrl+M`, and `F5`, accompanied by a TODO comment
about terminal behavior. The footer text in `crates/petit-tui/src/ui.rs` should
stay aligned with actual shortcuts so user guidance remains correct.

## Plan of Work

Milestone 1 updates the detokenization path in
`crates/petit-core/src/model_manager.rs` from the deprecated conversion API to
the recommended `token_to_piece` API. Keep behavior equivalent: continue
collecting generated pieces in order and return `output.trim().to_string()`.
Compile immediately after this edit to catch type and encoding mismatches.

Milestone 2 hardens shortcut handling in `crates/petit-tui/src/main.rs` by
extracting translation trigger matching into a small helper function, for
example `fn is_translate_shortcut(key: &KeyEvent) -> bool`. Replace the inline
compound condition with this helper and remove the TODO. Keep fallback keys.

Milestone 3 adds focused tests in `crates/petit-tui/src/main.rs` under
`#[cfg(test)]` that verify `is_translate_shortcut` returns true for
`Ctrl+Enter`, `Ctrl+M`, and `F5`, and false for unrelated keys. If footer copy
in `crates/petit-tui/src/ui.rs` is changed to mention fallbacks, verify text
still fits existing layout.

Milestone 4 runs validation commands, records concise output snippets in this
plan, and updates `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` so the plan remains restartable for a novice.

Milestone 5 aligns Rust edition metadata in Cargo manifests to 2024. Check
workspace root and crate manifests (`Cargo.toml` at repository root,
`crates/petit-core/Cargo.toml`, and `crates/petit-tui/Cargo.toml`) for direct
or workspace-inherited edition values that still resolve to 2021. Apply the
smallest consistent update so all crates build under edition 2024, then rerun
validation commands.

## Concrete Steps

Run from repository root:

    cd /Users/dzr/src/repo/petit_trad

Baseline:

    cargo check
    cargo test --workspace

After Milestone 1 and 2 edits:

    RUSTFLAGS="-D warnings" cargo check
    cargo test --workspace

  Edition alignment checks:

    rg -n "edition" Cargo.toml crates/*/Cargo.toml
    cargo metadata --format-version 1 > /tmp/petit-cargo-metadata.json
    rg -n '"edition":"2024"' /tmp/petit-cargo-metadata.json

Plan-completion sanity checks:

    rg -n "TODO: Confirm Ctrl\\+Enter behavior" crates/petit-tui/src/main.rs
    rg -n "token_to_str\\(" crates/petit-core/src/model_manager.rs

Manual shortcut confirmation:

    cargo run -p petit-tui

In the running UI, enter sample text and verify translation request starts for
`Ctrl+Enter`, `Ctrl+M`, and `F5`.

## Validation and Acceptance

Accept this task as complete when all conditions below are true:

- `RUSTFLAGS="-D warnings" cargo check` succeeds.
- `cargo test --workspace` succeeds.
- `crates/petit-core/src/model_manager.rs` no longer calls
  `LlamaModel::token_to_str`.
- The TODO at `crates/petit-tui/src/main.rs:140` is removed and replaced by
  explicit helper-backed shortcut logic.
- Automated tests cover shortcut matching for positive and negative cases.
- Manual TUI run confirms translation can be triggered with primary and fallback
  shortcuts.
- Cargo manifests and resolved workspace metadata reflect Rust edition 2024.

## Idempotence and Recovery

All edits are safe to reapply and can be validated repeatedly with the same
commands. If detokenization output degrades (garbled or empty text), revert only
the detokenization hunk in `crates/petit-core/src/model_manager.rs`, rerun
tests, and re-implement with explicit UTF-8-safe concatenation. If shortcut
changes misfire, keep only helper extraction, restore previous key conditions,
and add tests before reattempting behavior changes.

## Artifacts and Notes

Capture implementation evidence here during execution:

- Warning-strict check output snippet:
- Test output snippet:
- Manual shortcut validation notes (terminal emulator, OS, observed keys):

## Interfaces and Dependencies

This task depends on:

- `llama_cpp_2` model token conversion methods used by
  `crates/petit-core/src/model_manager.rs`.
- `crossterm::event::KeyEvent`, `KeyCode`, and `KeyModifiers` in
  `crates/petit-tui/src/main.rs`.
- Existing translation request flow through `request_translation` and worker
  channel plumbing, which must remain unchanged.

No public crate interface changes are expected; changes are internal to current
module boundaries.

## Revision Note

- 2026-02-19: Created this plan to execute the previously identified "task 4"
  cleanup (deprecation removal and terminal shortcut confirmation).
- 2026-02-19: Expanded task scope to include Rust edition alignment from 2021
  to 2024, with manifest and validation requirements.
