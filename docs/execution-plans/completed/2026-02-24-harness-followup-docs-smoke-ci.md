# Harness Follow-up: Align Docs, Expand Smoke, and Wire CI

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this change, contributors will have a cleaner harness workflow with
accurate docs, fuller runtime smoke coverage, and CI that runs both the
code-quality gate (`./scripts/check.sh`) and the runtime smoke harness
(`./scripts/smoke.sh`). The visible outcome is that local and CI instructions
match each other, the smoke harness catches invalid-language regressions, and
release automation remains focused on packaging instead of duplicating runtime
checks.

This plan does not change `scripts/eval.sh` behavior or fixture schema. Eval
robustness improvements were reviewed and intentionally deferred to a separate
future task.

## Progress

- [x] (2026-02-24 00:00Z) Completed research and recorded follow-up scope,
  findings, and CI/release recommendations in
  `docs/execution-plans/research/2026-02-24-harness-followup-alignment-smoke-eval-ci.md`.
- [x] (2026-02-24 00:00Z) Authored this ExecPlan scoped to docs alignment,
  smoke invalid-language coverage, and CI workflow updates only.
- [x] (2026-02-24 00:20Z) Fixed harness/CI documentation drift in `README.md`
  and `docs/BUILD.md` (CI-local wording, undefined `$FEATURES`, and Metal
  example `--gpu-layers 0` clarification).
- [x] (2026-02-24 00:21Z) Added invalid-language expected-failure coverage to
  `scripts/smoke.sh` and confirmed it remains model-free.
- [x] (2026-02-24 00:22Z) Updated `.github/workflows/ci.yml` to run
  `./scripts/smoke.sh` and added lightweight eval script syntax/help checks.
- [x] (2026-02-24 00:23Z) Ran local verification (`./scripts/check.sh`,
  `./scripts/smoke.sh`, eval script syntax/help checks) and recorded evidence
  in `Artifacts and Notes`.
- [x] (2026-02-24 00:35Z) Verify phase completed locally: reran final local
  checks, finalized this plan's evidence/outcomes, and prepared the file for
  move to `completed/`.
- [ ] (2026-02-24 00:35Z) Deferred outside this local session: capture a remote
  CI run URL showing the new smoke step after a branch push.

## Surprises & Discoveries

- Observation: `crates/petit-tui/src/config.rs` validates source/target
  language codes during config loading before any model initialization.
  Evidence: `validate_pair(...)` runs in `load_config()` before `run_stdin` or
  benchmark code creates `GemmaTranslator`.

- Observation: `scripts/smoke.sh` already provides model-free expected-failure
  checks and aggregates `PASS` / `FAIL` / `SKIP`, so adding one more validation
  check fits the existing script pattern without structural changes.
  Evidence: Current smoke harness checks include unknown-flag, conflicting-flag,
  empty-stdin, and benchmark validation failures.

- Observation: CI currently runs only `./scripts/check.sh`, while README wording
  can be read as if raw `cargo check` + `cargo test` is the complete CI local
  equivalent.
  Evidence: `.github/workflows/ci.yml` runs `./scripts/check.sh`, but `README.md`
  lists only `cargo check` and `cargo test` under the “same CI commands” text.

- Observation: The invalid-language runtime path is stable and remains model-free
  in practice when invoked through `petit-tui`.
  Evidence: `cargo run --quiet -p petit-tui -- --no-config --src xx --tgt fr`
  returns `Error: Invalid language pair: Unsupported language: xx` before any
  model-backed smoke check runs.

## Decision Log

- Decision: Add `./scripts/smoke.sh` to CI and keep `./scripts/check.sh` as the
  canonical code-quality gate.
  Rationale: `scripts/check.sh` covers format/lint/build/test while `smoke.sh`
  adds user-visible runtime validation paths without requiring model assets.
  Date/Author: 2026-02-24 / codex.

- Decision: Do not add smoke or eval execution to `.github/workflows/release.yml`
  in this plan.
  Rationale: Release workflow should remain focused on build/package/publish;
  runtime checks belong in CI and eval requires local model assets unavailable on
  hosted runners.
  Date/Author: 2026-02-24 / codex.

- Decision: Treat eval-harness robustness upgrades as deferred and out of scope
  for this plan.
  Rationale: The user explicitly narrowed the next plan to docs alignment,
  smoke coverage, and CI/release handling after reviewing the research.
  Date/Author: 2026-02-24 / codex.

- Decision: Add lightweight eval script CI checks (`sh -n` and `--help`) in the
  same CI workflow update.
  Rationale: These checks are model-free, fast, and catch shell syntax or CLI
  interface regressions in `scripts/eval.sh` and `scripts/eval-capture.sh`
  without introducing CI model/runtime dependencies.
  Date/Author: 2026-02-24 / codex.

## Outcomes & Retrospective

Completed local outcomes:

- `README.md` and `docs/BUILD.md` now align with the actual verification
  workflow by pointing local CI-equivalent guidance at `./scripts/check.sh` and
  `./scripts/smoke.sh`, removing an undefined `$FEATURES` example, and
  clarifying the Metal eval/capture examples that use `--gpu-layers 0`.
- `scripts/smoke.sh` now covers invalid-language validation as a model-free
  expected-failure check, increasing runtime smoke coverage without depending on
  local model assets.
- `.github/workflows/ci.yml` now runs the runtime smoke harness after
  `./scripts/check.sh` and adds lightweight model-free eval script syntax/help
  checks.
- `.github/workflows/release.yml` was intentionally left unchanged.

Local verification status:

- `./scripts/check.sh` passed.
- `./scripts/smoke.sh` passed with the new invalid-language check (`PASS=7`,
  `FAIL=0`, `SKIP=1`).
- `sh -n scripts/eval.sh scripts/eval-capture.sh` and both `--help` commands
  passed.

Deferred follow-up:

- Remote CI evidence (run URL and smoke-step outcome) could not be recorded in
  this local-only session because no branch push was performed.
- Eval robustness upgrades and eval/eval-capture refactor remain deferred by
  scope decision.

## Context and Orientation

The repository currently uses three shell scripts for local verification and
runtime checks:

- `scripts/check.sh` is the canonical code-quality verification script. It runs
  format, clippy, check, and tests. CI already uses this script.
- `scripts/smoke.sh` is a runtime smoke harness. It runs the real `petit-tui`
  binary through `cargo run`, records `PASS` / `FAIL` / `SKIP` per check, and
  skips the optional model-backed translation smoke if no model file exists.
- `scripts/eval.sh` is a translation regression harness that requires a local
  model and compares real translation output against checked-in fixtures.

This plan changes only the first two workflows and the docs that describe them.
It does not alter the eval fixture schema or matching behavior.

Relevant files and responsibilities:

- `README.md`: top-level contributor instructions, including local CI-equivalent
  commands and release workflow summary.
- `docs/BUILD.md`: detailed developer build/verification docs, including check,
  eval, and smoke harness usage.
- `scripts/smoke.sh`: runtime smoke checks to extend with an invalid-language
  expected-failure check.
- `.github/workflows/ci.yml`: CI workflow to update so smoke runs in CI.
- `.github/workflows/release.yml`: release workflow that remains unchanged.
- `crates/petit-tui/src/config.rs`: config loader that validates language pairs
  before model creation, enabling a model-free invalid-language smoke check.

Definitions used in this plan:

- “Model-free smoke check” means a runtime check that can run on a fresh clone
  without a local GGUF model file.
- “Expected-failure check” means a smoke check that passes only when the command
  exits non-zero and prints a specific error substring.
- “Lightweight eval script CI checks” means syntax/help checks for
  `scripts/eval.sh` and `scripts/eval-capture.sh` that do not run real
  translations and therefore do not require model assets.

This plan references research in
`docs/execution-plans/research/2026-02-24-harness-followup-alignment-smoke-eval-ci.md`,
but all implementation-critical details are repeated here so a novice can
execute the work from this file alone.

## Plan of Work

Milestone 1 aligns the docs with the implemented workflows. Update `README.md`
and `docs/BUILD.md` so the text and command examples reflect the actual CI and
harness behavior. The main corrections are: point CI-equivalent local guidance to
`./scripts/check.sh`, remove the undefined `$FEATURES` placeholder in BUILD, and
clarify whether `--features metal` examples with `--gpu-layers 0` are intended
CPU-run examples or true GPU offload examples.

Milestone 2 expands `scripts/smoke.sh` with one new invalid-language
expected-failure check. The implementation should follow the existing helper
pattern (`run_expect_failure`) and must remain model-free by exercising language
validation in config loading rather than translation execution. After this
milestone, `./scripts/smoke.sh` output should include a new `PASS` line for the
invalid-language validation path on machines without models.

Milestone 3 updates CI to run the smoke harness in addition to the check script.
Add a new CI step after `./scripts/check.sh` that runs `./scripts/smoke.sh`.
Optionally add lightweight eval script syntax/help checks if they remain fast and
portable. Record the decision in this plan (whether optional checks were added or
not) and do not modify `.github/workflows/release.yml`.

Milestone 4 performs verification and captures evidence. Run local check and
smoke commands, verify docs changes, and record concise outputs in `Artifacts and
Notes`. After pushing, capture the CI run URL showing both `scripts/check.sh` and
`scripts/smoke.sh` execution.

## Concrete Steps

Run all commands from the repository root:

    cd /Users/dzr/src/repo/petit_trad

Baseline before edits:

    ./scripts/check.sh
    ./scripts/smoke.sh
    ./scripts/eval.sh --help
    ./scripts/eval-capture.sh --help

Milestone 1 (docs alignment):

    edit README.md
    edit docs/BUILD.md

Required doc corrections:

- In `README.md`, replace the wording “same CI commands locally” with guidance
  that points to `./scripts/check.sh` (and optionally mentions `./scripts/smoke.sh`
  as an additional runtime sanity command).
- In `docs/BUILD.md`, replace the undefined `$FEATURES` ad-hoc example with
  explicit commands or clearly introduce the variable before use.
- In `docs/BUILD.md`, clarify the intent of Metal eval/capture examples using
  `--gpu-layers 0`; either explain they are CPU-run examples on a Metal build or
  change them to a real offload example.

Milestone 2 (smoke invalid-language check):

    edit scripts/smoke.sh
    ./scripts/smoke.sh

Add one expected-failure check that uses an invalid language code and verifies
an actionable error substring (for example `Invalid language pair` or
`Unsupported language`). The check must pass without a local model file and must
appear in the final summary counts.

Milestone 3 (CI workflow):

    edit .github/workflows/ci.yml

Required CI change:

- Add a step to run `./scripts/smoke.sh` after the existing `./scripts/check.sh`
  step.

Optional CI hardening (only if kept model-free and fast):

    sh -n scripts/eval.sh scripts/eval-capture.sh
    ./scripts/eval.sh --help
    ./scripts/eval-capture.sh --help

Do not modify `.github/workflows/release.yml` in this plan.

Milestone 4 (verification and evidence capture):

    ./scripts/check.sh
    ./scripts/smoke.sh

If optional CI hardening checks were added, run them locally too:

    sh -n scripts/eval.sh scripts/eval-capture.sh
    ./scripts/eval.sh --help
    ./scripts/eval-capture.sh --help

After pushing CI workflow changes, record the CI run URL and the result of the
new smoke step in this plan.

## Validation and Acceptance

This plan is complete when all of the following are true:

- `README.md` and `docs/BUILD.md` are aligned with actual CI/harness behavior,
  and the identified drift items are corrected.
- `scripts/smoke.sh` includes an invalid-language expected-failure check that
  passes on a machine without a local model file.
- `./scripts/smoke.sh` output still uses aggregated `PASS` / `FAIL` / `SKIP`
  reporting and includes the new invalid-language check line.
- `.github/workflows/ci.yml` runs both `./scripts/check.sh` and
  `./scripts/smoke.sh`.
- Any optional eval script syntax/help checks added to CI are model-free and do
  not run real translations.
- `.github/workflows/release.yml` is unchanged for this scope, and the rationale
  is recorded in this plan.
- Local verification commands pass (with any environment limitations explicitly
  recorded), and the updated CI run passes after the workflow change.

## Idempotence and Recovery

All work in this plan is documentation updates, shell-script check additions, or
CI workflow edits. These changes are safe to reapply. `./scripts/smoke.sh` must
remain read-only with respect to tracked files and should continue to report
`SKIP` rather than `FAIL` when optional model assets are unavailable.

If the new invalid-language smoke check is flaky due to error-message wording,
match a stable substring that reflects the validated behavior and document that
choice in the `Decision Log`. Do not broaden the match so far that unrelated
failures could pass.

If CI fails after adding the smoke step, record the failing run URL in
`Artifacts and Notes`, patch only the failing step or command, and rerun. Do not
expand release workflow checks to compensate for CI failures.

## Artifacts and Notes

Populate during implementation and verification:

- `README.md` and `docs/BUILD.md` drift-fix summary (what changed and why):
  `README.md` now points local CI-equivalent verification to `./scripts/check.sh`
  and `./scripts/smoke.sh` instead of raw `cargo check`/`cargo test`.
  `docs/BUILD.md` now uses explicit `cpu-only` ad-hoc commands and clarifies
  that the Metal eval/capture examples use `--gpu-layers 0` as a conservative
  CPU-run example on a Metal-capable build (use non-zero values for actual
  offload).
- `./scripts/smoke.sh` output excerpt showing the new invalid-language check and
  final summary counts (2026-02-24 local run):

      PASS  help output
      PASS  version output
      PASS  unknown flag validation (expected failure)
      PASS  conflicting flag validation (expected failure)
      PASS  empty stdin runtime validation (expected failure)
      PASS  benchmark run-count validation (expected failure)
      PASS  invalid language validation (expected failure)
      SKIP  stdin translation smoke (model not found: ...translategemma-12b...gguf)
      Summary: PASS=7 FAIL=0 SKIP=1 TOTAL=8

- `./scripts/check.sh` result summary after the changes (2026-02-24 local run):
  `fmt`, `clippy`, `check`, and `test` all passed for `cpu-only`; final line was
  `==> all checks passed` (24 `petit-core` tests and 10 `petit-tui` tests
  passed).
- Optional CI hardening decision: added eval script syntax/help checks to CI
  (`sh -n scripts/eval.sh scripts/eval-capture.sh`, plus `--help` for both)
  because they are model-free and fast.
- Local eval script syntax/help verification (2026-02-24): `sh -n` passed for
  both scripts and both `--help` commands succeeded (`OK` marker recorded).
- CI run URL(s) showing `scripts/check.sh` and `scripts/smoke.sh` execution:
  deferred until a branch push is performed outside this local-only session.
- Release workflow note: no changes made in this plan (intentional).

## Interfaces and Dependencies

Interfaces to preserve and extend:

- `scripts/smoke.sh`
  - Preserve current aggregated output and counter semantics (`PASS` / `FAIL` /
    `SKIP`).
  - Add one new model-free invalid-language expected-failure check using the
    existing helper style.

- `.github/workflows/ci.yml`
  - Preserve the existing `./scripts/check.sh` step.
  - Add `./scripts/smoke.sh` as an additive runtime validation step.
  - Optional: add model-free syntax/help checks for eval scripts only.

Out-of-scope interfaces (must remain unchanged in this plan):

- `scripts/eval.sh` runtime comparison behavior and fixture schema
- `scripts/eval-capture.sh`
- `.github/workflows/release.yml`

## Revision Note

- 2026-02-24: Initial ExecPlan created after research review to address docs
  alignment, smoke invalid-language coverage, and CI smoke integration while
  deferring eval robustness changes and release workflow modifications.
- 2026-02-24: Executed plan milestones for docs alignment, smoke invalid-language
  coverage, and CI workflow updates (including lightweight eval script
  syntax/help checks); recorded local verification evidence and left remote CI
  URL capture for Verify phase after human review.
- 2026-02-24: Verify phase completed locally, outcomes/retrospective finalized,
  and remote CI URL capture marked deferred because no push was performed in
  this session.
