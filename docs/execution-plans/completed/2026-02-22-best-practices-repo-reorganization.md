# Top Priority

Here is a special task, which is a top priority currently.
Below are several articles that share practices regarding how to work with AI code agents / harness:

- On Claude Code: https://boristane.com/blog/how-i-use-claude-code/
- On pi: https://lucumr.pocoo.org/2026/1/31/pi/, https://mariozechner.at/posts/2025-11-30-pi-coding-agent/
- VCZH's personal workflow for github copilot's per-request usage quota:
  https://x.com/geniusvczh/status/2024463703438246008?s=20,
  https://github.com/vczh-libraries/Tools/blob/master/Copilot/AGENTS.md
- OpenAI on Codex: https://openai.com/index/harness-engineering/,
  https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html,
  https://developers.openai.com/cookbook/articles/codex_exec_plans

Your target is to design a best practise according to these sharing, then clean up and reorganize this repo accordingly.

## Requirements and constraints

- Don't over-engineering. This project is small and simple currently, so we only need what is
  useful for now, as a strong and extensive base tempalte which can be scaled into any projects.
- You only need to modify according this repo. You can use template if necessary but we don't
  summarize for a general repo kickstarter / template at this phase.
- You should include necessary, reasonable build, check, test, format, ci / cd and pr etc
  mechanics (imagine what a real individual contributor developer should do in developing
  process), but fully e2e automated workflow (AI review each other etc) is not a target yet.
  Human still review the plan and result, sign off the pr merge etc.
- On the other hand, during each automated phase, we need the agent to work as less interven as
  possible. They should be able to work out the whole phase without interruption unless necessary
  safe approval is needed. E.g. Human feedback is required after design / plan / task is
  finished, but during the draft / implementation it should work with no interruption.

---

## Design

### Key Takeaways from Referenced Articles

**Boris Tane -- "How I Use Claude Code"**

The core insight is strict separation of thinking from typing. Every task
follows Research (read deeply, write findings to a file) then Plan (write
plan.md, never use built-in plan mode) then Annotate (human edits the plan
inline, 1-6 rounds) then Implement (one command: "implement it all, mark
tasks done, don't stop"). The persistent plan file is the shared mutable
state between human and agent. The "don't implement yet" guard prevents
wasted effort. During implementation, feedback is terse -- single sentences,
screenshots, references to existing code. Human stays in the driver's seat
for architecture and scope decisions.

**Mario Zechner -- "What I learned building pi"** and
**Armin Ronacher -- "Pi: The Minimal Agent Within OpenClaw"**

Both emphasize minimalism as a design philosophy. Pi uses only four tools
(read, write, edit, bash) and a system prompt under 1000 tokens. Key
practices: file-based state over built-in features (plans and todos live in
files, not agent memory); progressive disclosure (skills and docs are loaded
only when needed, not stuffed into context); AGENTS.md as the per-project
context file loaded hierarchically from global to project-specific; agent
can extend itself by writing code rather than downloading extensions.
Armin's approach: no built-in plan mode (use files), no built-in todos (use
files), no MCP (use CLI tools + README), no sub-agents for routine context
gathering (do it in a separate session first, create an artifact). Context
engineering -- controlling exactly what enters the model's context -- is
paramount.

**VCZH -- "GitHub Copilot keyword-routed workflow"**

Designed for Copilot's per-request quota model where each request is a
fresh context. AGENTS.md acts as a router: the first word of the user
message (scrum, design, plan, execute, verify, review, etc) maps to a
specific prompt file in `.github/prompts/`. Each prompt file contains the
instructions for exactly one workflow stage. `copilot-instructions.md` is
the baseline always-loaded guideline. Knowledge base files provide domain
context without polluting the agent prompt. This is a powerful pattern for
scaling workflow stages independently as the project grows.

**OpenAI / Codex -- "Using PLANS.md for multi-hour problem solving"** and
**matklad -- "ARCHITECTURE.md"**

AGENTS.md is the universal entry point for any coding agent. ExecPlans
(self-contained living documents) are the mechanism for complex multi-step
work, with mandatory Progress, Surprises, Decision Log, and Retrospective
sections. Plans must be novice-guiding and outcome-focused. Verification is
not optional. ARCHITECTURE.md is a lightweight bird's-eye view: top-level
shape, component boundaries, invariants, data flow. It answers "where is
the thing that does X?" and "what does this thing do?". Name important
entities but don't deep-link (links go stale). Keep it short and update
infrequently.

### Principles for This Repo

Drawn from all five sources, filtered by the constraints in this task:

1. **Progressive disclosure.** AGENTS.md is the landing pad. It tells the
   agent what to read next, not everything at once. Deep docs (PLANS.md,
   design docs, product specs) are loaded only when needed.

2. **File-based state.** Plans, decisions, research, and progress live in
   files under version control. The repo is the single source of truth, not
   chat history or agent memory.

3. **Separate thinking from typing.** Research and plan before implementing.
   Human reviews between phases. During implementation, the agent works
   autonomously against an approved plan.

4. **Verification is mandatory.** A single canonical verification command
   that both agents and humans run. No claim of "done" without evidence.

5. **Minimal and sufficient.** Only add what the project needs today, but
   structure it so new stages, checks, or docs can be added without
   reorganizing. Don't over-engineer for hypothetical scale.

### Current State Assessment

Strengths of the current repo:

- ARCHITECTURE.md follows matklad's style well: concise, names components,
  states invariants, avoids deep-linking.
- PLANS.md is thorough and closely follows the OpenAI/Codex ExecPlan model.
- The execution-plans directory with active/completed split is functioning.
- CI exists with check + test on two platforms.
- The doc tree (design-docs, product-specs) is organized.

Gaps to address:

- AGENTS_TEST.md sits at root unused. It was an exploratory draft with
  ideas that should be absorbed or discarded.
- No verification script exists. Agents have no single command to run all
  checks.
- No format or lint tooling is configured (no rustfmt.toml, no clippy
  lints, CI does not run `cargo fmt --check` or `cargo clippy`).
- No Copilot-specific instructions (.github/copilot-instructions.md) even
  though AGENTS.md is meant to work with multiple agents.
- AGENTS.md rules are decent but the doc lacks a clear workflow section and
  a doc map for progressive disclosure.
- No PR template with verification checklist.
- docs/BUILD.md does not reference a verification script.

### Planned Changes

#### A. AGENTS.md Overhaul

Restructure into three concise sections:

1. **Doc Map** -- key files and when to read them. This enables progressive
   disclosure: the agent reads AGENTS.md, then loads deeper docs only when
   the task requires it.
2. **Workflow** -- the lifecycle of a task: research, plan, execute, verify.
   Not a full keyword router (over-engineering for this project), but a
   clear enumeration of stages and what each produces. Human checkpoint
   between plan and execute. During execute, agent works without stopping.
3. **Rules** -- non-negotiable conventions: line length, commit format, no
   emoji, always verify before commit. Consolidate and slightly tighten the
   existing rules.

Include the canonical verification command inline:

    ./scripts/check.sh

Keep the file under ~60 lines so it fits in a single agent context load.

#### B. Remove AGENTS_TEST.md

It is a draft that was never adopted. Its useful ideas (doc map, operating
rules, verification discipline, definition of done) are now absorbed into
the new AGENTS.md. The file will be deleted.

#### C. Add `scripts/check.sh`

A single POSIX-compatible script that runs, in order:

1. `cargo fmt` (auto-format, only when `--fix` flag is passed) or
   `cargo fmt --check` (verify-only, the default)
2. `cargo clippy --workspace -- -D warnings` -- lint
3. `cargo test --workspace` -- tests

The `--fix` flag is for local pre-commit use: it auto-formats and then
runs the rest of the pipeline. Without `--fix`, the script is
check-only (used in CI). The script defaults to `--features cpu-only`
for agent and CI use but accepts an optional `--features` override for
local GPU builds. This is the canonical verification command for agents
and humans alike, referenced in AGENTS.md, BUILD.md, and CI.

#### D. Add Tooling Config

- `rustfmt.toml` -- workspace formatting settings. Use Rust edition 2024
  defaults.
- Add `[workspace.lints.clippy]` section to root `Cargo.toml` to enable
  a baseline clippy lint set at workspace level.

Note on `max_width`: Rust's default is 100. We keep it at 100 (the
Rust community standard) rather than 120 (our Markdown rule). Code and
prose have different density; 100 columns for code is already generous.

#### E. Update CI

Add two steps to `.github/workflows/ci.yml`:

- `cargo fmt --check` (fail the build on format drift)
- `cargo clippy --workspace --features cpu-only -- -D warnings`

This ensures format and lint are enforced in CI, not just locally.

#### F. Adopt `.agents/` Folder and Restructure Instruction Layers

All agent-related configuration lives under `.agents/` at repo root.
This avoids scattering across `.github/`, `.pi/`, etc. and makes
migration between tools trivial (rename or symlink the folder).

The two-layer instruction model:

1. **AGENTS.md** (repo root) -- the universal entry point. Loaded
   automatically by Copilot, Pi, Codex, and most other agents. Keeps
   only concise, portable content: a doc map, workflow summary, and
   non-negotiable rules that apply to any project. Target: under ~60
   lines. This file is designed to be reusable across projects with
   minimal edits.

2. **`.agents/instructions.md`** -- project-specific instructions.
   AGENTS.md points here. Contains the detailed doc map (full paths),
   project-specific workflow details, expanded tooling references, and
   notes on future expansion paths (see below). When switching agent
   tools, only this file and its siblings need adjustment.

For Copilot specifically: `.github/copilot-instructions.md` remains as
a thin redirect (~3 lines) that says "read AGENTS.md and follow its
pointers." Copilot auto-loads both AGENTS.md and this file, so the
redirect is a safety net ensuring the detailed instructions are always
reachable.

Future expansion paths documented in `.agents/instructions.md`:

- `.agents/prompts/` -- keyword-routed prompt files (VCZH pattern).
  When the project grows to need distinct per-stage prompts (design,
  plan, execute, verify, review), each becomes a file here. AGENTS.md
  can then add a keyword router section pointing to them.
- `.agents/skills/` -- reusable agent skills (Pi pattern). When the
  first custom skill is needed, create this directory with a README
  explaining the skill format.

#### G. Add PR Template

`.github/pull_request_template.md` with a minimal checklist:

- Verification evidence (`scripts/check.sh` output or CI link)
- Plan link (if applicable)
- Key decisions or rationale

This helps both agents and humans produce consistent PRs.

#### H. Update docs/BUILD.md

Add a "Verification" section that references `scripts/check.sh` and
explains what it runs. Currently BUILD.md has "Quick Checks" with just
`cargo check` and `cargo test`. Expand it to include the full verification
pipeline.

#### I. Final Cleanup

- Confirm all Markdown files respect <= 120 line length.
- Confirm no stale references remain after AGENTS_TEST.md removal.
- `TOP_PRIORITY.md` receives a completion note after implementation.

### What This Design Does NOT Do

To stay within the "no over-engineering" constraint:

- No keyword-router prompt files yet. The VCZH pattern is powerful but
  only needed for multi-stage per-request workflows. This project is
  too small for that now. AGENTS.md's workflow section covers stages in
  prose. A note in `.agents/instructions.md` documents the future
  expansion path (`.agents/prompts/`) so the next contributor knows
  where and how to add them.
- No `docs/state/STATUS.md` or execution-logs directory. ExecPlans
  already track progress and decisions inline. A separate status file
  adds maintenance cost with no current payoff.
- No automated agent review pipeline. Human reviews plans and results.
- No `.agents/skills/` directory yet. A note in `.agents/instructions.md`
  documents the future expansion path so the directory can be created
  when the first skill is needed.
- Pre-commit enforcement: `scripts/check.sh` supports a `--fix` flag
  that auto-formats before verifying. The AGENTS.md rule requires
  running `./scripts/check.sh --fix` before every commit. CI enforces
  the check mode (without --fix) to catch any drift. No git hooks for
  now -- the combination of explicit rule + CI enforcement is sufficient
  and avoids hook-setup friction for new contributors.

### Validation

After implementation, the following should be true:

- Running `./scripts/check.sh` from repo root passes (format, clippy,
  tests).
- CI pipeline runs format check, clippy, and tests on both ubuntu and
  macos.
- AGENTS.md is a concise, progressive-disclosure entry point under ~60
  lines.
- AGENTS_TEST.md no longer exists.
- A new contributor (human or agent) can read AGENTS.md, understand the
  workflow, find the verification command, and navigate to deeper docs
  without prior context.

---

## Execution Plan

### Task List

- [x] (2026-02-22) T1. Create `rustfmt.toml` at repo root
- [x] (2026-02-22) T2. Add workspace clippy lints to `Cargo.toml`
- [x] (2026-02-22) T3. Run `cargo fmt` to normalize existing code
- [x] (2026-02-22) T4. Fix any clippy warnings surfaced by the new lint config
- [x] (2026-02-22) T5. Create `scripts/check.sh`
- [x] (2026-02-22) T6. Rewrite `AGENTS.md` (concise, progressive-disclosure entry point)
- [x] (2026-02-22) T7. Create `.agents/instructions.md` (project-specific details)
- [x] (2026-02-22) T8. Create `.github/copilot-instructions.md` (thin redirect)
- [x] (2026-02-22) T9. Delete `AGENTS_TEST.md`
- [x] (2026-02-22) T10. Update `.github/workflows/ci.yml` (add fmt + clippy steps)
- [x] (2026-02-22) T11. Create `.github/pull_request_template.md`
- [x] (2026-02-22) T12. Update `docs/BUILD.md` (add Verification section)
- [x] (2026-02-22) T13. Markdown line-length audit across all git-tracked .md files
- [x] (2026-02-22) T14. Run `./scripts/check.sh` and confirm clean pass
- [x] (2026-02-22) T15. Final review: stale references, coherence, completeness

### Task Details

**T1. Create `rustfmt.toml`**

Create file at repo root with Rust edition 2024 defaults. Keep it
minimal -- only specify `edition` so the workspace edition is used for
formatting style. Leave `max_width` at Rust's default 100.

File: `rustfmt.toml`

    edition = "2024"

**T2. Add workspace clippy lints to `Cargo.toml`**

Add a `[workspace.lints.clippy]` section at the end of the root
`Cargo.toml`. Enable a conservative baseline: warn on common
categories. Also add `[workspace.lints.rust]` for `unsafe_code = "warn"`.
Each member crate's `Cargo.toml` must inherit workspace lints with
`[lints] workspace = true`.

**T3. Run `cargo fmt`**

Execute `cargo fmt` from repo root to normalize all existing Rust
source files to the new formatting rules. This is a one-time bulk
format. Any diffs are expected and correct.

**T4. Fix clippy warnings**

Run `cargo clippy --workspace --features cpu-only -- -D warnings`.
Fix any warnings. If a warning requires a non-trivial code change,
add an `#[allow(...)]` with a justification comment rather than risk
behavioral changes.

**T5. Create `scripts/check.sh`**

Create `scripts/check.sh`, make it executable. The script:

- Accepts `--fix` flag: when present, runs `cargo fmt` (auto-format)
  instead of `cargo fmt --check`.
- Accepts `--features <value>` to override the default `cpu-only`.
- Runs in order: fmt, clippy, test.
- Exits on first failure with a clear message.
- Uses `set -euo pipefail` for strictness.

Expected invocations:

    # CI / verify-only (default)
    ./scripts/check.sh

    # Local pre-commit
    ./scripts/check.sh --fix

    # With GPU features
    ./scripts/check.sh --features metal

**T6. Rewrite `AGENTS.md`**

Replace the entire file. New structure (target: under 60 lines):

1. Header with repo name.
2. Doc Map section -- bullet list of key docs and when to read them.
   Include `.agents/instructions.md` as the first pointer for detailed
   project-specific context.
3. Workflow section -- brief prose on the research, plan, execute,
   verify lifecycle. State that human reviews between plan and execute.
   During execute, agent works autonomously. Reference verification
   command.
4. Rules section -- numbered list. Merge and tighten existing rules.
   Add: "Run `./scripts/check.sh --fix` before every commit."
   Keep the commit format rule, line-length rule, no-emoji rule.

**T7. Create `.agents/instructions.md`**

This is the project-specific detailed instruction file. Contains:

- Full doc map with repo-relative paths for all key docs
  (ARCHITECTURE.md, docs/PLANS.md, docs/BUILD.md,
  docs/design-docs/index.md, docs/product-specs/index.md,
  docs/execution-plans/).
- Expanded workflow details: what a research artifact looks like, how
  ExecPlans work (reference docs/PLANS.md), how to use the
  verification script, PR conventions.
- Verification section: exact command and what it runs.
- Future expansion paths section: notes on `.agents/prompts/` and
  `.agents/skills/` for when the project grows.

**T8. Create `.github/copilot-instructions.md`**

Thin redirect file (~5 lines) that tells Copilot to read `AGENTS.md`
at repo root and follow its pointers. No duplicated content.

**T9. Delete `AGENTS_TEST.md`**

Remove the file from repo root. Its useful ideas have been absorbed
into the new AGENTS.md and `.agents/instructions.md`.

**T10. Update `.github/workflows/ci.yml`**

Add two new steps before the existing "Check workspace" step:

1. `cargo fmt --check` -- fail on format drift.
2. `cargo clippy --workspace --features cpu-only -- -D warnings` --
   fail on lint warnings.

Keep existing `cargo check` and `cargo test` steps.
Add `components: clippy, rustfmt` to the Rust toolchain installation
step to ensure both are available.

**T11. Create `.github/pull_request_template.md`**

Minimal markdown checklist:

    ## Checklist

    - [ ] `./scripts/check.sh` passes (or CI is green)
    - [ ] Plan link (if applicable): <!-- link -->
    - [ ] Key decisions or rationale noted in PR description

**T12. Update `docs/BUILD.md`**

Replace the "Quick Checks" section with a "Verification" section that
references `scripts/check.sh`, explains each step it runs (fmt,
clippy, test), and shows both check-only and fix modes. Keep the
existing CI-mirror commands as a subsection or note for backward
compatibility. Also add brief note about `--features` override.

**T13. Markdown line-length audit**

Scan all git-tracked `.md` files for lines exceeding 120 characters.
Fix any violations. Focus on the files modified in this task and the
existing docs. Use the AGENTS.md rule: fill lines as long as possible
before breaking, break naturally for readability.

**T14. Run `./scripts/check.sh` and confirm clean pass**

Execute the script from repo root. All three stages (fmt check,
clippy, test) must pass. Record the output as evidence.

**T15. Final review**

- Grep for references to `AGENTS_TEST.md` -- must find none.
- Confirm AGENTS.md is under 60 lines.
- Read `.agents/instructions.md` and verify all doc map links are
  correct paths.
- Confirm CI workflow has all four steps (fmt, clippy, check, test).
- Confirm `docs/BUILD.md` references the verification script.
- Mark this execution plan's tasks as done.
- Add a completion note at the bottom of this file.

### Execution Order and Dependencies

Tasks are grouped into phases for linear execution. Within a phase,
tasks are independent and can be done in any order.

Phase 1 (tooling foundation): T1, T2, T3, T4, T5
Phase 2 (documentation): T6, T7, T8, T9, T10, T11, T12
Phase 3 (validation): T13, T14, T15

Dependencies: T3 depends on T1. T4 depends on T2. T5 depends on T1
and T2 (needs to know what checks to run). T14 depends on T5 and T10.
T15 depends on all prior tasks.

---

## Completion Note

All 15 tasks completed 2026-02-22. Implementation evidence:

- `./scripts/check.sh` passes: fmt, clippy (-D warnings), 26 tests.
- CI updated with fmt + clippy steps on ubuntu and macos.
- AGENTS.md rewritten (39 lines), `.agents/instructions.md` created.
- AGENTS_TEST.md deleted, `.github/copilot-instructions.md` added.
- PR template and BUILD.md Verification section added.
- Two clippy fixes applied: `enumerate()` loop (app.rs) and collapsed
  `if let && path.exists()` (config.rs). Unsafe FFI in model_manager.rs
  annotated with `#![allow(unsafe_code)]` and justification comment.
