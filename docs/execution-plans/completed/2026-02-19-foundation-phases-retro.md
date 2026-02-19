# Foundation Phases Retro (0-3 + partial 4/5)

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
were maintained during retrospective consolidation.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

This record captures what was already delivered before the planning system was
standardized. It lets new contributors start from completed facts and focus only
on remaining work.

## Progress

- [x] (2026-02-19 08:40Z) Consolidated delivered milestones from prior mixed plan.
- [x] (2026-02-19 08:45Z) Split unfinished work into active phase plans.
- [x] (2026-02-19 08:50Z) Preserved key evidence notes for CUDA validation.

## Surprises & Discoveries

- Observation: Prior roadmap text mixed shipped work, future plans, and design.
  Evidence: Single historical plan document contained all three concerns.

## Decision Log

- Decision: Preserve delivered milestones as one completed retrospective record.
  Rationale: Keeps historical context while preventing active-plan clutter.
  Date/Author: 2026-02-19 / codex.

- Decision: Move unfinished work to phase-specific active ExecPlans.
  Rationale: Aligns execution tracking with current `docs/PLANS.md` policy.
  Date/Author: 2026-02-19 / codex.

## Outcomes & Retrospective

Shipped foundation outcomes:

- Phase 0 bootstrap and workspace setup
- Phase 1 prototype and prompt-format discovery
- Phase 2 `petit-core` translation/runtime foundation
- Phase 3 `petit-tui` loop, UI, config, and stdin integration
- Phase 4 partial delivery: feature wiring + WSL CUDA validation
- Phase 5 partial delivery: README/build-guide/help improvements

Unshipped at record time:

- macOS Metal validation
- CI and release workflows
- performance and UX polish completion
- GUI exploration phase

## Context and Orientation

This is a completed retrospective, not an active implementation plan. It exists
to provide a factual baseline for active plans in:

- `docs/execution-plans/active/2026-02-19-phase-4-cross-platform-ci-release.md`
- `docs/execution-plans/active/2026-02-19-phase-5-polish-performance-ux.md`
- `docs/execution-plans/active/2026-02-19-phase-6-gui-exploration.md`

## Plan of Work

Retrospective consolidation used three steps: extract completed milestones,
separate unfinished milestones, and carry forward evidence needed by active
plans.

## Concrete Steps

Historical consolidation was document-only. Verification command used:

    cd /Users/dzr/src/repo/petit_trad
    cargo test --workspace

## Validation and Acceptance

Accepted when:

- Completed foundation milestones are listed clearly.
- Remaining work is tracked in active plans, not this file.
- Evidence notes required by active plans are preserved.

## Idempotence and Recovery

This file can be edited repeatedly as a historical document without changing
runtime behavior. If details are corrected later, log the correction under
`Revision Note` and keep prior facts explicit.

## Artifacts and Notes

Evidence highlights carried forward:

- 2026-01-20: CUDA build succeeded with `CUDAToolkit_ROOT=/usr/local/cuda` and
  `nvcc` on PATH.
- 2026-02-02: WSL CUDA TUI stdin flow translated
  "Hello, how are you?" -> "Bonjour, comment allez-vous ?".

## Interfaces and Dependencies

This retrospective references outcomes across:

- `crates/petit-core`
- `crates/petit-tui`
- `docs/design-docs/prompt-format.md`

No interface changes are introduced by this file.

## Revision Note

- 2026-02-19: Retrofitted this completed record to full ExecPlan structure from
  `docs/PLANS.md`.
