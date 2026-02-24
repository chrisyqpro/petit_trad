# Runtime Smoke Harness

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this work, the repository will have `scripts/smoke.sh`, a runtime smoke
script that
runs a small runtime smoke suite and prints a clear status line for each check,
plus a final summary. A developer will be able to run one command and quickly
see what passed, what failed, and what was skipped because local runtime assets
(such as a model file) are not present.

The visible result is readable output with stable labels such as `PASS`,
`FAIL`, and `SKIP`, instead of fail-fast behavior that hides later checks.

## Progress

- [x] (2026-02-23 20:13Z) Completed research and recorded findings in
  `docs/execution-plans/research/2026-02-23-runtime-smoke-harness.md`.
- [x] (2026-02-23 20:32Z) Recreated ExecPlan after research approval (prior
  draft was discarded because workflow stop point was missed).
- [x] (2026-02-23 20:40Z) Implemented `scripts/smoke.sh` with aggregated
  `PASS` / `FAIL` / `SKIP` reporting, expected-failure checks, and optional
  model-backed smoke.
- [x] (2026-02-23 20:40Z) Documented the smoke harness in `docs/BUILD.md`.
- [x] (2026-02-23 20:40Z) Ran `./scripts/smoke.sh` and `sh -n scripts/smoke.sh`;
  recorded
  evidence and environment limitation (Cargo dependency fetch blocked by
  network-restricted sandbox).
- [x] (2026-02-23 20:53Z) Relocated the harness to `scripts/smoke.sh` per
  review feedback and reran the harness successfully from the new path.
- [x] (2026-02-24 01:57Z) Completed verify phase: recorded final smoke
  evidence (including user-confirmed model-backed pass), ran
  `./scripts/check.sh` cleanly, and prepared this plan to move to `completed/`.

## Surprises & Discoveries

- Observation: `scripts/check.sh` uses fail-fast shell behavior (`set -eu`) and
  prints only stage headers reached before a failure.
  Evidence: Static read of `scripts/check.sh`.

- Observation: `crates/petit-tui/src/main.rs` can provide reliable runtime smoke
  coverage without model initialization through `--help`, `--version`, and early
  validation failures.
  Evidence: `run()` short-circuits help/version before config load, and several
  CLI/runtime validation errors occur before `GemmaTranslator::new(...)`.

- Observation: The default configured model path points under `models/`, and
  this worktree does not contain a `models/` directory.
  Evidence: `config/default.toml` path plus local `ls models` failure during
  research.

- Observation: The harness correctly continues after failures and prints a
  final summary, but Cargo-backed checks fail in this sandbox before reaching
  application logic because dependencies are not locally cached and network
  access is blocked.
  Evidence: `./scripts/smoke.sh` output shows repeated crates.io resolution
  failure and ends with `Summary: PASS=0 FAIL=6 SKIP=1 TOTAL=7`.

- Observation: After relocation to `scripts/smoke.sh`, a subsequent run in the
  same environment completed successfully for all model-free checks and still
  reported the model-backed check as `SKIP` because no model file exists.
  Evidence: `./scripts/smoke.sh` ended with
  `Summary: PASS=6 FAIL=0 SKIP=1 TOTAL=7`.

## Decision Log

- Decision: `smoke.sh` will be additive and will not replace `scripts/check.sh`.
  Rationale: `scripts/check.sh` remains the canonical format/lint/build/test
  gate; the smoke harness targets runtime sanity and readability.
  Date/Author: 2026-02-23 / codex.

- Decision: Include expected-failure checks and count them as `PASS` when the
  failure is the expected one (exit status and output substring both match).
  Rationale: This exercises meaningful runtime and validation paths on machines
  without model assets.
  Date/Author: 2026-02-23 / codex.

- Decision: Model-dependent smoke checks must report `SKIP` instead of `FAIL`
  when no local model path is available.
  Rationale: Fresh clones often lack model files, and the harness should remain
  useful without producing misleading failures.
  Date/Author: 2026-02-23 / codex.

## Outcomes & Retrospective

Implemented the requested runtime smoke harness and documented it in
`docs/BUILD.md`. The script provides clear per-check statuses (`PASS`, `FAIL`,
`SKIP`) and a final summary while continuing after individual failures.

Validation in this sandbox confirms the aggregation/reporting behavior,
successful model-free application-path checks, and `SKIP` handling for a
missing model file. An earlier run failed on Cargo dependency resolution, but a
subsequent run completed successfully after dependencies became available in the
local cache.

The user additionally confirmed a model-backed run passed when
`SMOKE_MODEL_PATH` was supplied, which completes the intended optional runtime
coverage path.

## Context and Orientation

The repository currently provides `scripts/check.sh` as the documented
verification command in `docs/BUILD.md`. That script is intentionally fail-fast
and is appropriate for gating commits and CI, but it is not designed to present
an aggregated runtime status summary.

This change introduces a separate script, `scripts/smoke.sh`, focused on
runtime smoke checks for the `petit-tui` executable. A smoke check is a short
command that validates a high-level path still works (for example, the binary
starts and prints help, or a validation error is returned for bad input).

Relevant repository files:

- `scripts/check.sh`: existing canonical verification script (behavior remains
  unchanged).
- `docs/BUILD.md`: developer-facing documentation for build/verification
  commands (must be updated to mention `scripts/smoke.sh`).
- `crates/petit-tui/src/main.rs`: runtime entrypoint and mode dispatch.
- `crates/petit-tui/src/cli.rs`: CLI argument parsing and early error messages.
- `crates/petit-tui/src/config.rs`: config-loading path that is exercised by
  some smoke checks before model initialization.
- `config/default.toml`: default model path and runtime config defaults.

## Plan of Work

Add a new executable file `scripts/smoke.sh` using POSIX `sh`.
The script will define helper functions to run commands, capture output, and
record results without aborting after the first failure. It will keep counters
for `PASS`, `FAIL`, and `SKIP`, print one readable status line per check, then
print a final summary and exit non-zero only when one or more checks truly fail.

Implement two categories of checks. The first category will always run and will
be model-free: positive checks (`--help`, `--version`) and expected-failure
checks for CLI/runtime validation (`Unknown argument`, conflicting flags, empty
stdin, invalid benchmark runs). For expected-failure checks, the harness must
confirm both a non-zero exit code and a recognizable error substring.

The second category will be an optional model-backed smoke check. The script
will look for a model path from `SMOKE_MODEL_PATH` first, then optionally a
known default path from `config/default.toml` if present. If no model file is
available, it will report `SKIP` with a reason. If a model file is found, it
will run a minimal stdin translation command and report success or failure.

Update `docs/BUILD.md` with a short section describing `./scripts/smoke.sh`, what its
statuses mean, and how it complements (rather than replaces) `./scripts/check.sh`.

## Concrete Steps

Run commands from the repository root:

    cd /Users/dzr/.local/share/codex/worktrees/294a/petit_trad

Implement and mark executable:

    chmod +x scripts/smoke.sh

Run smoke harness (fresh clone / no model expected):

    ./scripts/smoke.sh

Run optional model-backed smoke (if model file exists locally):

    SMOKE_MODEL_PATH=/absolute/path/to/model.gguf ./scripts/smoke.sh

Canonical verification command remains:

    ./scripts/check.sh

As implementation proceeds, this section must be updated with concise output
excerpts and actual commands run.

Executed during implementation:

    chmod +x scripts/smoke.sh
    sh -n scripts/smoke.sh
    ./scripts/smoke.sh

Observed `./scripts/smoke.sh` excerpt in this sandbox:

    Runtime Smoke Harness (petit_trad)
    Working directory: /Users/dzr/.local/share/codex/worktrees/294a/petit_trad
    PASS  help output
    PASS  version output
    PASS  unknown flag validation (expected failure)
    PASS  conflicting flag validation (expected failure)
    PASS  empty stdin runtime validation (expected failure)
    PASS  benchmark run-count validation (expected failure)
    SKIP  stdin translation smoke (model not found: /Users/dzr/.local/share/codex/worktrees/294a/petit_trad/models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf)
    Summary: PASS=6 FAIL=0 SKIP=1 TOTAL=7

## Validation and Acceptance

The work is complete when all of these are true:

- `./scripts/smoke.sh` prints a clear status line for every configured smoke check.
- Status labels are explicit (`PASS`, `FAIL`, `SKIP`) and easy to scan.
- The script continues running after an individual check fails and still reports
  later checks.
- Expected-failure checks only count as `PASS` when both the non-zero exit and
  expected error text are observed.
- The final summary includes status counts and the script exits non-zero when at
  least one check is `FAIL`.
- Missing local model assets produce `SKIP` for the model-backed smoke check,
  not `FAIL`.
- `docs/BUILD.md` documents how to use `./scripts/smoke.sh` and how it relates to
  `./scripts/check.sh`.

## Idempotence and Recovery

`scripts/smoke.sh` must be safe to run repeatedly. It should not modify repository
files, and any Cargo build artifacts it triggers are normal and acceptable.
Failures should be reported with enough context (command and short output) for a
developer to rerun the failing check directly.

If the optional model-backed check fails due to local environment setup, the
harness should still provide useful signal from the model-free checks. Developers
can rerun with `SMOKE_MODEL_PATH` set to a known-good model file path.

## Artifacts and Notes

Populate during execution and verification:

- `./scripts/smoke.sh` output excerpt: Model-free checks passed, model-backed
  check was skipped, and the final line was
  `Summary: PASS=6 FAIL=0 SKIP=1 TOTAL=7`.
- `./scripts/smoke.sh` exit code: `0` on the successful relocation rerun.
- User-reported model-backed smoke result: Passed final translation smoke when
  `SMOKE_MODEL_PATH` was provided (no skip).
- `docs/BUILD.md` update summary: Added `Runtime Smoke Harness` section with
  `./scripts/smoke.sh` usage, status semantics, `SMOKE_MODEL_PATH` override, and
  clarification that `scripts/check.sh` remains canonical.
- `./scripts/check.sh` result summary: Clean pass on 2026-02-24 (`fmt`,
  `clippy`, `check`, `test` all passed; unit tests 24 in `petit-core` and 10
  in `petit-tui`).

## Interfaces and Dependencies

Required new interface:

- Executable script `scripts/smoke.sh` (POSIX `sh` compatible).

Shell behavior requirements:

- Internal status aggregation (`PASS`, `FAIL`, `SKIP`) with counters.
- Final process exit code `0` only when no `FAIL` occurred.
- Command execution via `cargo run -p petit-tui -- ...` so the real runtime
  entrypoint is exercised.

Initial smoke check set to implement:

- Help output success check.
- Version output success check.
- Unknown-flag expected-failure check.
- Conflicting `--no-config` + `--config` expected-failure check.
- Empty stdin expected-failure check.
- Invalid benchmark run count (`--runs 0`) expected-failure check.
- Optional model-backed stdin translation smoke (`SKIP` when no model file is
  available).

## Revision Note

- 2026-02-23: Initial approved-plan draft created after research review, with
  the prior premature plan draft intentionally discarded.
- 2026-02-23: Updated after execution to record implemented `scripts/smoke.sh`,
  documentation changes, and sandbox-limited validation evidence.
- 2026-02-23: Updated to relocate the smoke harness from repository root to the
  `scripts/` directory per review feedback.
- 2026-02-23: Updated validation evidence after successful rerun from
  `scripts/smoke.sh` (PASS=6, SKIP=1).
- 2026-02-24: Final verify-phase update with clean `./scripts/check.sh` run and
  user-confirmed model-backed smoke pass.
