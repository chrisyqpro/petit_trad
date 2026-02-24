# Research: Harness Follow-up (Docs Alignment, Smoke Coverage, Eval Robustness, CI Scope)

## Purpose

This research captures the current state of the recently added harness tooling
(`scripts/smoke.sh`, `scripts/eval.sh`, `scripts/eval-capture.sh`) and the
remaining gaps found during review. It exists to support a follow-up ExecPlan
that another agent can execute without redoing the discovery work.

The follow-up scope is intentionally limited to:

- docs alignment fixes for harness/CI wording
- one missing smoke-harness runtime check (invalid language path)
- CI/release integration decision and any required workflow updates

This research explicitly excludes refactoring duplicate logic between
`eval.sh` and `eval-capture.sh`.

## Current Repository State (relevant files)

Harness scripts present:

- `scripts/check.sh` (canonical code-quality verification)
- `scripts/smoke.sh` (runtime smoke harness)
- `scripts/eval.sh` (translation regression harness)
- `scripts/eval-capture.sh` (capture exact fixtures from a working backend)

Harness fixtures present:

- `eval/fixtures/smoke-inputs.tsv` (curated corpus inputs)
- `eval/fixtures/smoke.tsv` (captured exact outputs)

Workflow/docs:

- `.github/workflows/ci.yml` runs `./scripts/check.sh` only
- `.github/workflows/release.yml` builds/publishes tagged binaries
- `docs/BUILD.md` documents check/smoke/eval/capture workflows
- `README.md` documents CI/release and benchmark workflows

## Findings: Requirement Coverage vs Suggested Enhancements

### 1. Translation eval harness exists, but robustness work is deferred from this follow-up plan

`scripts/eval.sh` is implemented and usable. It supports model path override,
feature selection, fixture path selection, and runtime overrides
(`--gpu-layers`, `--context-size`, `--threads`). It prints per-case results and a
summary and exits non-zero on failures/errors.

However, it currently uses exact string comparison only. The fixture schema is a
single TSV row shape with `expected_output`, and the runtime comparison is exact
stdout equality. There is no support for tolerant checks such as `must_contain`
or `expected_any_of`, and there is no fixture type for expected error cases.

Impact: the harness is good for exact-captured regression checks on one
backend/model setup, but it is brittle for intentionally variable outputs and
cannot directly cover CLI/runtime error expectations in the eval suite.

Decision for this follow-up: document this as deferred work and do not include
eval schema/matching upgrades in the next plan. The immediate plan will focus on
docs alignment, smoke coverage, and CI scope only.

### 2. Runtime smoke harness exists, but one high-signal check is missing

`scripts/smoke.sh` already covers:

- `--help`
- `--version`
- unknown flag validation
- conflicting `--no-config` + `--config`
- empty stdin validation
- benchmark `--runs 0` validation
- optional model-backed stdin translation smoke (`SKIP` if no model)

The missing suggested check is invalid-language validation.

Important implementation detail: `crates/petit-tui/src/config.rs` validates the
language pair in `load_config()` before model initialization. This means an
invalid-language smoke check can remain model-free and should not be gated by
local model availability.

### 3. Docs are mostly aligned, with a small set of concrete drift issues

Confirmed drift:

- `README.md` says “same CI commands locally” but shows only `cargo check` and
  `cargo test`. CI actually runs `./scripts/check.sh`, which includes fmt and
  clippy as well.
- `docs/BUILD.md` ad-hoc command snippet uses `cargo check --features
"$FEATURES"` without defining `FEATURES` in that section.
- `docs/BUILD.md` eval/capture Metal examples use `--features metal` with
  `--gpu-layers 0`, which reads like a GPU example but disables GPU offload.
  This may be intentional for portability, but the docs do not explain that.

No broader design-doc or product-spec drift was found for these harness changes.
The current BUILD/README coverage is the right durable location for this level
of tooling behavior.

## Overlap / Simplification Notes (deferred)

`scripts/eval.sh` and `scripts/eval-capture.sh` duplicate several behaviors:
option parsing defaults, fixture safety validation, and the `cargo run`
translation invocation with environment-reset logic.

This is real duplication, but refactoring it now would mix structural cleanup
with future eval-behavior changes. It should be handled as a separate task after
the next round of harness requirements is confirmed.

## CI and Release Integration Decision (recommended)

### Smoke harness in CI: Yes

Recommendation: add `./scripts/smoke.sh` to `.github/workflows/ci.yml` after
`./scripts/check.sh`.

Reasoning:

- The script is intentionally model-free by default and returns `SKIP` for the
  optional model-backed check.
- It exercises user-visible runtime/validation behavior not covered by
  `scripts/check.sh`.
- It is fast enough to be practical on standard CI runners.

### Eval harness in CI: No real translation eval in default CI (for now)

Recommendation: do not run `./scripts/eval.sh` as a real translation suite in
standard CI.

Reasoning:

- It requires a local GGUF model file and runtime backend support not available
  on standard GitHub-hosted runners in this repo.
- The fixture outputs are exact-captured and model/backend-dependent.
- Adding a fake or mocked CI run would not validate the actual behavior the
  harness exists to check.

Optional CI hardening (low-cost): run shell syntax/help-interface checks for
`scripts/eval.sh` and `scripts/eval-capture.sh` (`sh -n`, `--help`) so CI still
catches script breakage without requiring model assets.

### Smoke/eval in release workflow: No

Recommendation: keep `.github/workflows/release.yml` focused on building and
publishing artifacts; do not add smoke or eval steps there.

Reasoning:

- Release workflow should not duplicate runtime checks already handled by CI.
- Release runners still do not have model assets for eval.
- Adding smoke to release provides little new signal and increases release-path
  failure surface.

If release validation becomes necessary later, it should use a separate
post-release verification workflow with explicit artifacts/test inputs rather
than expanding `release.yml`.

## Follow-up Plan Requirements (for ExecPlan)

The follow-up ExecPlan should cover:

1. Docs alignment fixes

- `README.md` CI wording must point to `./scripts/check.sh` (and optionally
  mention `./scripts/smoke.sh` as runtime sanity coverage).
- `docs/BUILD.md` ad-hoc command snippet must not reference an undefined
  `$FEATURES` variable.
- `docs/BUILD.md` must clarify whether Metal eval examples with `--gpu-layers 0`
  are intentionally CPU-run examples or switch them to a real offload example.

2. Smoke harness coverage completion

- Add an invalid-language expected-failure check to `scripts/smoke.sh`.
- Keep this check model-free by relying on config validation in `load_config()`.
- Update `docs/BUILD.md` only if the smoke harness check list is documented in a
  way that must stay exact.

3. CI/release workflow scope update

- Add `./scripts/smoke.sh` to `.github/workflows/ci.yml`.
- Optionally add `sh -n` and `--help` checks for eval scripts in CI.
- Explicitly keep `release.yml` unchanged and record the rationale in the
  follow-up plan decision log (and docs only if needed for contributor clarity).

Deferred follow-up (not in the next plan):

- Eval harness robustness upgrades (`must_contain`, `expected_any_of`,
  expected-error fixtures, `--fail-fast`) and related fixture schema changes.
- Eval/eval-capture shared-logic refactor.

## Risks and Constraints to Capture in the Plan

- CI smoke integration must preserve current `scripts/check.sh` as the canonical
  code-quality gate (smoke is additive, not a replacement).
- If optional eval script syntax/help checks are added to CI, they must remain
  model-free and fast so CI does not gain a hidden runtime asset dependency.
