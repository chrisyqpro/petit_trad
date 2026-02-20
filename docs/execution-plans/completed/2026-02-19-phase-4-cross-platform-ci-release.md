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
- [x] (2026-02-19 10:22Z) Ran and documented macOS Metal validation checklist.
- [x] (2026-02-19 10:24Z) Added GitHub Actions CI for Linux and macOS CPU checks.
- [x] (2026-02-19 10:24Z) Added tagged release workflow with binary artifacts.
- [x] (2026-02-19 10:26Z) Updated docs with exact commands and troubleshooting.
- [x] (2026-02-19 23:12Z) Captured first CI and release run URLs after pushing to GitHub.

## Surprises & Discoveries

- Observation: Metal feature path successfully loads and runs with the 27B model
  when the model path is configured in user config.
  Evidence: `echo "Hello, how are you?" | cargo run -p petit-tui --features
  metal -- --stdin --target-lang fr` produced
  `Bonjour, comment allez-vous ?`.

- Observation: Metal backend initialization is confirmed in llama runtime logs.
  Evidence: `logs/llama.log` contains
  `llama_model_load_from_file_impl: using device Metal (Apple M1 Max)` and
  multiple `load_tensors: layer ... assigned to device Metal` lines.

- Observation: Workspace tests pass after workflow and doc updates.
  Evidence: `cargo test --workspace` passed with 24 unit tests in `petit-core`
  and no failures.

- Observation: Existing deprecation warnings are visible in `petit-core` during
  build and test.
  Evidence: Warnings reference `LlamaModel::token_to_str` and
  `Special::Tokenize` in `crates/petit-core/src/model_manager.rs`.

## Decision Log

- Decision: Keep GPU checks out of CI and validate GPU paths manually.
  Rationale: GPU-capable runners are not guaranteed and would add flakiness.
  Date/Author: 2026-02-19 / codex.

- Decision: Track macOS Metal validation before release automation finalization.
  Rationale: Release artifacts should be based on a validated platform baseline.
  Date/Author: 2026-02-19 / codex.

- Decision: Scope CI checks to CPU-only features on Linux and macOS.
  Rationale: This keeps PR feedback deterministic and avoids GPU runner drift.
  Date/Author: 2026-02-19 / codex.

- Decision: Publish tagged release assets as `tar.gz` archives with
  `${tag}-${platform}` naming.
  Rationale: Compression reduces artifact size and naming is explicit for users.
  Date/Author: 2026-02-19 / codex.

## Outcomes & Retrospective

Phase 4 implementation landed for repository automation and contributor docs.
The repository now has CI checks at `.github/workflows/ci.yml` for Linux/macOS
CPU-only `cargo check` and `cargo test`, and a tag-driven release workflow at
`.github/workflows/release.yml` that builds and publishes `petit` archives.

macOS Metal validation commands were exercised on an arm64 host and confirmed
that Metal is actually selected by llama.cpp at runtime. This was verified with
both successful translation output and backend log lines in `logs/llama.log`.

First-run workflow verification is complete on GitHub: both CI and Release
workflows finished successfully, and the release contains Linux and macOS
binary artifacts.

## Context and Orientation

The repository already ships feature forwarding for `cuda`, `metal`, `vulkan`,
and `cpu-only` from `crates/petit-tui/Cargo.toml` to
`crates/petit-core/Cargo.toml`. Historical notes indicate WSL CUDA validation
was successful. This phase adds fresh automation and doc coverage for
Linux/macOS CPU verification plus tag release publishing.

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

Environment capture commands used for this phase:

  uname -a
  sw_vers
  rustc -V
  cargo -V

CI/release workflow checks should include:

    git status --short
    rg --files .github/workflows

Release trigger command:

  git tag v0.1.0
  git push origin v0.1.0

## Validation and Acceptance

Accept this plan as complete when:

- Metal validation notes include machine/toolchain details and observed output.
- CI workflow runs on Linux and macOS and passes for current `main`.
- Release workflow produces expected artifacts on tag events.
- Docs include exact commands and expected outcomes for contributors.

Current status in this workspace:

- Completed: Metal checklist documentation, workflow implementation, docs update,
  and local `cargo test --workspace` validation.
- Completed outside local workspace: first remote CI/release executions and
  links recorded below.

## Idempotence and Recovery

Workflow-file edits are idempotent and can be re-applied safely. If a workflow
run fails, keep the failing log reference in this plan, patch only the failing
step, and rerun. Avoid destructive git commands; use additive commits and
rollback by reverting specific workflow commits if needed.

## Artifacts and Notes

Keep short references here as work proceeds:

- CI run URL(s): <https://github.com/chrisyqpro/petit_trad/actions/runs/22210294033> (success)
- Release run URL(s): <https://github.com/chrisyqpro/petit_trad/actions/runs/22210342606> (success)
- Release page URL(s): <https://github.com/chrisyqpro/petit_trad/releases/tag/v0.1.0-phase4-verify-20260219>
- Release assets:

  petit-v0.1.0-phase4-verify-20260219-linux-x64.tar.gz
  petit-v0.1.0-phase4-verify-20260219-macos-arm64.tar.gz
- Metal validation output snippet:

  Running `target/debug/petit --stdin --target-lang fr`
  Bonjour, comment allez-vous ?

- Metal backend log snippet:

  llama_model_load_from_file_impl: using device Metal (Apple M1 Max)
  load_tensors: layer 0 assigned to device Metal, is_swa = 1

## Interfaces and Dependencies

This plan depends on:

- Cargo workspace build/test behavior (`cargo test --workspace`)
- GitHub Actions runner images for Linux and macOS
- Existing backend feature wiring in Cargo manifests

No Rust public API change is required for this phase.

## Revision Note

- 2026-02-19: Retrofitted this file to full ExecPlan structure from
  `docs/PLANS.md`.
- 2026-02-19: Implemented CI/release workflows and updated docs, then recorded
  local Metal and test validation evidence plus pending remote run links.
- 2026-02-19: Updated model-default alignment and validation evidence after
  verifying successful 27B Metal load and translation output.
- 2026-02-19: Aligned docs with XDG config precedence and recorded explicit
  Metal runtime log evidence.
- 2026-02-19: Executed remote workflow verification by pushing `main` and
  annotated tag `v0.1.0-phase4-verify-20260219`; recorded successful CI and
  Release run URLs plus published asset names.
