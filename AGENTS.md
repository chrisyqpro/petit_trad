# AGENTS.md - petit_trad

## Project

**petit_trad** - Local translation tool using TranslateGemma (4B/12B/27B). Rust core + TUI.

## Core Docs

- `ARCHITECTURE.md` - High-level architecture map
- `docs/PLANS.md` - Requirements for execution plans
- `docs/execution-plans` - Folder for plans
- `docs/BUILD.md` - Project build guide
- `docs/design-docs/index.md` - Design docs TOC
- `docs/product-specs/index.md` - Product specs TOC

## Rules

1. **Read `ARCHITECTURE.md` and corresponding docs** before structural changes
2. **Plans are treated as first-class artifacts**. Ephemeral lightweight plans are used for small changes
3. **Plans** live in "active" vs "completed" folder in `docs/execution-plans`
4. **Use an ExecPlan (as described in docs/PLANS.md)** from design to implementation, when writing complex features
   or significant refactors
5. **Known technical debt** is tracked in `docs/execution-plans/tech-debt-tracker.md`
6. **Markdown line length** - keep line length <= 120 in git-tracked Markdown files
7. **NEVER** use emoji
8. **Git signatures** - may skip for automated tasks, never skip for merge commits
9. **Commit Message** - strictly follow conventional commits format; line length: title (first line) 50, body 72, in body
    use list (if more than one change) to explain what are changed and why (each in natural human sentence)
