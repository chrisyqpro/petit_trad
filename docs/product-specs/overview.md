# Product Spec Overview

## Summary

`petit_trad` is a local translation tool using TranslateGemma models.
The first product surface is a terminal UI (`petit`) with one-shot stdin mode.

## Primary Users

- Developers and power users who want local/private translation
- CLI-first workflows that need scriptable one-shot translation

## Core Value

- Local-only translation with no cloud API dependency
- Practical terminal workflow for interactive and scripted usage
- Shared Rust core that supports multiple frontends over time
