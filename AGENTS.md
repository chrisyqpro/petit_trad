# AGENTS.md — petit_trad

Instructions for AI agents working on this project.

## Project

**petit_trad** — Local translation tool using TranslateGemma (4B/12B/27B). Rust core + TUI.

## Key Files

- `doc/architecture.md` — System design, tech stack, data flow
- `doc/plan.md` — Shared project plan and progress
- `.agent/plan.md` — Current tasks and progress (agent local, not in git)
- `doc/prompt-format.md` — TranslateGemma prompt conventions (when created)

## Structure

```
crates/petit-core/   # Translation engine library
crates/petit-tui/    # Terminal interface (binary: petit)
proto/               # Python prototype
doc/                 # Permanent documentation
.agent/              # Agent workspace (gitignored)
```

## Rules

1. **Read `doc/architecture.md`** before making structural changes
2. **Update `.agent/plan.md`** when completing tasks, copy from `doc` folder if the file doesn't exist
3. **No cloud APIs** — We run TranslateGemma locally via llama-cpp-2
4. **Cross-platform** — Must work on WSL, Linux, macOS, Windows
5. **Markdown line length** — Keep lines ≤120 characters in git-tracked `doc/` Markdown files
6. **NEVER** use emoji
7. **Git signatures** — Agents may skip git signature for automated tasks but NEVER skip it for merge commits