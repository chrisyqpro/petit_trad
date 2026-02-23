# AGENTS.md

Follow pointers for other docs, but load deeper docs only when the task needs them.

## Doc Map

- `ARCHITECTURE.md` - Top-level architecture design: component boundaries, invariants, data flow; For human and agents.
  Read before any structural change.
- `docs/PLANS.md` - ExecPlan requirements; read before writing any plan
- `docs/BUILD.md` - build commands, verification pipeline, release process
- `docs/design-docs/index.md` - TOC for technical design docs
- `docs/product-specs/index.md` - TOC for product specs
- `docs/execution-plans/research/` - research findings for design and plan
- `docs/execution-plans/active/` - current active ExecPlans
- `docs/execution-plans/completed/` - completed ExecPlans (historical context)
- `docs/execution-plans/tech-debt-tracker.md` - known technical debt

## Workflow Detail

Every non-trivial task follows these phases: (Don't modify any code in step 0-2; DON't stop within a phase)

0. **Branch** -- Always work on a new seperate branch
1. **Research** -- Read source files deeply. Never skim. Write findings to `docs/execution-plans/research/<YYYY-MM-DD>-<slug>.md`
   before planning. The research artifact is your review surface for the human, with findings / insights helpful for design
   and plan; if the research is wrong, the plan and implementation will be wrong. Stop for review and approval before moving on.
2. **Plan** -- Create `docs/execution-plans/active/<YYYY-MM-DD>-<slug>.md` following `docs/PLANS.md` exactly. The plan must
   be self-contained: a novice should be able to implement the feature end-to-end from the plan alone. Stop after writing
   the plan and wait for human review and approval.
3. **Execute** -- Implement against the approved plan. Mark tasks done in the Progress section as you go. Commit frequently
   (small, coherent diffs. always run check script and achieve a clean pass before commit). Do not pause for confirmation.
   Resolve ambiguities by logging the decision in the Decision Log and continuing. Once all tasks are finished wait for
   human review.
4. **Verify** -- At this point the work is finished and reviewed. Record final output in the ExecPlan's Artifacts section
   as evidence. Do a final check, move the plan from the active to completed folder and commit anything left.
5. **Pull Request** (Optional) -- If explicitly required, push the branch to remote then send a PR to main branch

## Git Requirements

- Commit message format: Conventional Commits format strictly. Title first line <= 50 chars, one-word scope. Body lines
  <= 72 chars. Use a markdown style list of natural human sentences to explain what and why, when multiple things
  changed, in the body. Body should be one whole block.
- Never skip commit message for tag, merge, or any git operations. Skip gpg signature for intermediate commits, but never
  skip signature for merge, tag or PR.

## Expansion Paths

**Keyword-routed prompts**: when the project benefits from per-stage agent prompts, create `.agents/prompts/` and add a
keyword router section to `AGENTS.md`.
Suggested map: `design`, `plan`, `execute`, `verify`, `review` -> `.agents/prompts/<keyword>.md`
For prompts to be enable in copilot VSCode as a slash command, create softlink in `.github/prompts/`. They don't have to
be keyword-routing type only.

**Agent skills**: when a reusable agent skill is needed (e.g., a commit-message helper or context-gathering workflow),
create `.agents/skills/<skill-name>/SKILL.md` and reference it from `AGENTS.md`.

**Pre-commit hooks**: if format drift becomes a recurring problem despite CI, add a `.pre-commit-config.yaml` and a git
hooks setup that runs `./scripts/check.sh --fix`. The script is already compatible with this use.

## Other Rules

- Line length <= 120 (NOT 80) in git-tracked Markdown.
- For Markdown and git commit message, fill lines naturally close to limit before breaking (soft cap, can exceed by a
  few chars for readability).
- Never use emoji.
