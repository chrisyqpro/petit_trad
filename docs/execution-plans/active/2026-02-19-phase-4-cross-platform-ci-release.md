# Phase 4: Cross-Platform Build, CI, and Release

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this work, maintainers can verify macOS Metal behavior and rely on CI and
release automation for repeatable builds. The user-visible effect is that tagged
releases provide trusted binaries and every pull request gets automated build and
test feedback.

## Progress

- [x] (2026-02-19 09:00Z) Initial phase scope recorded from prior roadmap notes.
- [ ] (2026-02-19 09:00Z) Run and document macOS Metal validation checklist.
- [ ] (2026-02-19 09:00Z) Add GitHub Actions CI for Linux and macOS CPU checks.
- [ ] (2026-02-19 09:00Z) Add tagged release workflow with binary artifacts.
- [ ] (2026-02-19 09:00Z) Update docs with exact commands and troubleshooting.

## Surprises & Discoveries

- Observation: No new platform findings recorded yet in this plan revision.
  Evidence: Not run yet in this plan iteration.

## Decision Log

- Decision: Keep GPU checks out of CI and validate GPU paths manually.
  Rationale: GPU-capable runners are not guaranteed and would add flakiness.
  Date/Author: 2026-02-19 / codex.

- Decision: Track macOS Metal validation before release automation finalization.
  Rationale: Release artifacts should be based on a validated platform baseline.
  Date/Author: 2026-02-19 / codex.

## Outcomes & Retrospective

In progress. Complete this section when the phase is closed, including shipped
artifacts, missed items, and follow-up work.

## Context and Orientation

The repository already ships feature forwarding for `cuda`, `metal`, `vulkan`,
and `cpu-only` from `crates/petit-tui/Cargo.toml` to
`crates/petit-core/Cargo.toml`. Historical notes indicate WSL CUDA validation
was successful. Remaining gaps are:

- macOS Metal validation evidence
- `.github/workflows` CI pipeline
- `.github/workflows` release pipeline

Primary files likely touched by this plan:

- `.github/workflows/ci.yml` (new)
- `.github/workflows/release.yml` (new)
- `README.md`
- `docs/BUILD.md`
- `docs/execution-plans/active/2026-02-19-phase-4-cross-platform-ci-release.md`

## Plan of Work

Milestone 1 validates the Metal runtime path on Apple Silicon and captures
exact environment details. Milestone 2 introduces CPU-only CI for Linux and
macOS so every PR has deterministic feedback. Milestone 3 adds release
automation for tagged builds. Milestone 4 updates docs so contributors can
reproduce each step without prior context.

## Concrete Steps

Run from repository root:

    cd /Users/dzr/src/repo/petit_trad

Metal validation commands:

    cargo run -p petit-tui --features metal
    echo "Hello" | cargo run -p petit-tui --features metal -- --stdin

Baseline test command after workflow edits:

    cargo test --workspace

CI/release workflow checks should include:

    git status --short
    rg --files .github/workflows

## Validation and Acceptance

Accept this plan as complete when:

- Metal validation notes include machine/toolchain details and observed output.
- CI workflow runs on Linux and macOS and passes for current `main`.
- Release workflow produces expected artifacts on tag events.
- Docs include exact commands and expected outcomes for contributors.

## Idempotence and Recovery

Workflow-file edits are idempotent and can be re-applied safely. If a workflow
run fails, keep the failing log reference in this plan, patch only the failing
step, and rerun. Avoid destructive git commands; use additive commits and
rollback by reverting specific workflow commits if needed.

## Artifacts and Notes

Keep short references here as work proceeds:

- CI run URL(s):
- Release run URL(s):
- Metal validation output snippet:

## Interfaces and Dependencies

This plan depends on:

- Cargo workspace build/test behavior (`cargo test --workspace`)
- GitHub Actions runner images for Linux and macOS
- Existing backend feature wiring in Cargo manifests

No Rust public API change is required for this phase.

## Revision Note

- 2026-02-19: Retrofitted this file to full ExecPlan structure from
  `docs/PLANS.md`.
