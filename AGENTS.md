# AGENTS.md - petit_trad

## Core Docs

- `ARCHITECTURE.md` - High-level architecture map
- `docs/PLANS.md` - Requirements for execution plans
- `docs/execution-plans` - Folder for plans
- `docs/BUILD.md` - Project build guide
- `docs/design-docs/index.md` - Design docs TOC
- `docs/product-specs/index.md` - Product specs TOC

## Rules

1. Read `ARCHITECTURE.md` and corresponding docs before structural changes
2. Plans are treated as first-class artifacts. Ephemeral lightweight plans are used for small changes
3. Plans are created in `active` folder in `docs/execution-plans` and should be moved to `completed` folder after completion
4. Use an ExecPlan (as described in `docs/PLANS.md`) from design to implementation, when writing complex features
   or significant refactors
5. Known technical debt is tracked in `docs/execution-plans/tech-debt-tracker.md`
6. Keep line length <= 120 in git-tracked Markdown files
7. Never use emoji
8. Always check and format before commit
9. Commit message should strictly follow conventional commits format. Line length: title (first line) 50, body 72.
   In title, only use one word for scope. (e.g. plan instead of execute-plan). In body, use list (if more than one change)
   to explain what are changed and why (each in natural human sentence). No unnecessary empty line.
10. Never skip git signature and commit message for tag, merge and so on.
