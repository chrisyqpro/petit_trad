# Phase 5: Polish, Performance, and UX Feedback

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `docs/PLANS.md`.

## Purpose / Big Picture

After this work, translation performance behavior is measured and documented,
and users receive clearer feedback when configuration or runtime errors occur.
The visible outcome is faster and more predictable interaction plus clearer
error/status messaging.

## Progress

- [x] (2026-02-19 09:10Z) Phase objective captured from historical roadmap notes.
- [ ] (2026-02-19 09:10Z) Define benchmark matrix for short and medium inputs.
- [ ] (2026-02-19 09:10Z) Gather baseline latency/startup measurements.
- [ ] (2026-02-19 09:10Z) Implement focused optimizations with evidence.
- [ ] (2026-02-19 09:10Z) Improve error/status messages for common failures.
- [ ] (2026-02-19 09:10Z) Update docs with measured expectations.

## Surprises & Discoveries

- Observation: No profiling evidence has been added to this plan yet.
  Evidence: Measurements are pending first milestone execution.

## Decision Log

- Decision: Treat measured before/after data as mandatory acceptance evidence.
  Rationale: Performance claims without data are not actionable.
  Date/Author: 2026-02-19 / codex.

- Decision: Prioritize user-facing error clarity over broad refactors.
  Rationale: Immediate user experience gains with lower regression risk.
  Date/Author: 2026-02-19 / codex.

## Outcomes & Retrospective

In progress. Fill this section with shipped changes, unresolved gaps, and
lessons learned when the phase closes.

## Context and Orientation

The current application already supports TUI mode, stdin mode, language
validation, and worker-thread inference. Remaining polish work is not about new
features; it is about quality of existing behavior.

Likely touch points:

- `crates/petit-tui/src/main.rs` (status/error handling)
- `crates/petit-tui/src/config.rs` (configuration error messaging)
- `crates/petit-core/src/model_manager.rs` (performance-sensitive inference path)
- `README.md`
- `docs/BUILD.md`
- this plan file

## Plan of Work

Milestone 1 defines a repeatable benchmark matrix so later measurements are
comparable. Milestone 2 captures baseline numbers. Milestone 3 applies targeted
optimizations and validates gains. Milestone 4 improves error/status messaging.
Milestone 5 documents outcomes and operational guidance.

## Concrete Steps

Run from repository root:

    cd /Users/dzr/src/repo/petit_trad

Baseline quality gate:

    cargo test --workspace

Example measurement loop (to be finalized in milestone 1):

    echo "Hello, how are you?" | cargo run -p petit-tui -- --stdin --target-lang fr

Capture repeatable timing evidence with shell timing tools and record results in
`Artifacts and Notes`.

## Validation and Acceptance

Accept this plan as complete when:

- A benchmark matrix is documented in this file.
- Before/after performance evidence is recorded for key scenarios.
- At least one measurable improvement is demonstrated.
- Common error paths report actionable messages.
- Workspace tests pass with no regressions.

## Idempotence and Recovery

Profiling commands are safe to rerun. Keep benchmark inputs stable across runs.
If an optimization regresses behavior, revert only the affected commit and
preserve measurement evidence in this plan so future attempts are informed.

## Artifacts and Notes

Record evidence here as work proceeds:

- Benchmark matrix:
- Baseline numbers:
- Post-change numbers:
- Example improved error messages:

## Interfaces and Dependencies

Key dependencies:

- `llama-cpp-2` runtime behavior in `petit-core`
- TUI state and status messaging flow in `petit-tui`
- Existing config precedence guarantees

No new public interface is required unless a validated optimization demands it.

## Revision Note

- 2026-02-19: Retrofitted this file to full ExecPlan structure from
  `docs/PLANS.md`.
