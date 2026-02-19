# Phase 6: GUI Exploration and Direction

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this work, the project has a documented GUI direction and a small
integration proof that reuses `petit-core`. The visible outcome is a clear,
testable path from TUI-only operation to a future GUI without architecture drift.

## Progress

- [x] (2026-02-19 09:20Z) Initial exploration scope captured in plan form.
- [ ] (2026-02-19 09:20Z) Build decision matrix for GUI stack candidates.
- [ ] (2026-02-19 09:20Z) Select preferred stack with explicit tradeoffs.
- [ ] (2026-02-19 09:20Z) Produce minimal integration spike using `petit-core`.
- [ ] (2026-02-19 09:20Z) Record recommended repository structure and rollout.

## Surprises & Discoveries

- Observation: No spike or benchmark evidence has been captured yet.
  Evidence: Candidate implementations are not started in this plan revision.

## Decision Log

- Decision: Require a runnable spike before final GUI stack commitment.
  Rationale: Avoid architecture lock-in based on assumptions alone.
  Date/Author: 2026-02-19 / codex.

- Decision: Preserve TUI as supported baseline during GUI exploration.
  Rationale: TUI is current production surface and must remain stable.
  Date/Author: 2026-02-19 / codex.

## Outcomes & Retrospective

In progress. Complete with final stack decision, spike result, and follow-ups.

## Context and Orientation

Current production path is TUI-first. `petit-core` already encapsulates
translation runtime logic and is the intended shared backend boundary. GUI work
must not duplicate inference logic or bypass `petit-core` contracts.

Likely touch points:

- `docs/design-docs/architecture.md`
- `docs/product-specs/scope.md`
- potential future crate path (for example `crates/petit-gui/`) if chosen
- this plan file

## Plan of Work

Milestone 1 builds a decision matrix across candidate stacks. Milestone 2
selects one stack with rationale tied to project constraints. Milestone 3
implements a minimal spike proving `petit-core` reuse. Milestone 4 proposes
repository structure and phased delivery.

## Concrete Steps

Run from repository root:

    cd /Users/dzr/src/repo/petit_trad

Baseline test command before and after spike:

    cargo test --workspace

If a spike crate is added, include explicit build/run commands here and in the
`Artifacts and Notes` section.

## Validation and Acceptance

Accept this plan as complete when:

- Decision matrix is documented with clear tradeoffs.
- One stack is selected with rationale and rejected alternatives.
- A minimal runnable spike demonstrates `petit-core` integration.
- Follow-up implementation plan is documented and actionable.

## Idempotence and Recovery

Exploration work should be additive and isolated. Keep spike code separate from
production paths so it can be safely revised or removed. If a spike direction is
rejected, preserve evidence in this plan and revert spike-specific commits.

## Artifacts and Notes

Record evidence here as work proceeds:

- Decision matrix summary:
- Spike repository path:
- Spike run output:
- Final stack decision statement:

## Interfaces and Dependencies

Critical boundary:

- `petit-core` remains the sole translation backend API surface.

Evaluation dependencies:

- candidate GUI framework/toolchain
- build and packaging implications on supported platforms

## Revision Note

- 2026-02-19: Retrofitted this file to full ExecPlan structure from
  `docs/PLANS.md`.
