# AGENTS.md

THIS DOC IS HOLY INSTRUCTION THAT YOU SHALL FOLLOW EVERY WORD. Follow pointers for other docs, but
load deeper docs only when the task needs them.

## Doc Map

- `ARCHITECTURE.md` - Top-level architecture design: component boundaries, invariants, data flow;
  For human and agents. Read before any structural change.
- `docs/PLANS.md` - ExecPlan requirements; read before writing any plan
- `docs/BUILD.md` - build commands, verification pipeline, release process
- `docs/design-docs/index.md` - TOC for technical design docs
- `docs/product-specs/index.md` - TOC for product specs
- `docs/execution-plans/research/` - research findings for design and plan
- `docs/execution-plans/active/` - current active ExecPlans
- `docs/execution-plans/completed/` - completed ExecPlans (historical context)
- `docs/tech-debt-tracker.md` - known technical debt

## Workflow Detail

The specific instructions of each step / phase of the workflow is defined as skills. If there is no
direct call of a skill, usually the first words of the prompt (especially the first one of a
session) contain the keyword of the correct skill. The current avaiable keywords are: **Research**,
**Design**, **Plan**, **Execute**, **Review**, **Commit**, **Pull Request**

## Rules

- Only modify this file when explicitly told to.
- Always check if there is related skills for the your task.
- Always work on new git branch, unless you are instructed explicitly to work on a branch.
- Use linter and formatter to enforce code and doc convention.
- If it's a project with check script, always run check script with fix first before commit or hand
  in for review, and only proceed with a clear pass.
- Execution-plans docs are work logs instead of persistent docs. You will only refer to them when
  explicitly asked to continue a step of on-going workflow or resume previously paused work. Never
  refer to execution-plans when start new work (use persistent design and product docs as truth).
  This also means you should always make sure the persistent docs are updated and aligned with the
  code.
- Never use emoji.
