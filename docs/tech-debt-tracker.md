# Technical Debt Tracker

This file tracks known technical debt items that are not yet assigned to an active execution plan.

## Open Items

- Translation harness glossary isolation
  - Note: The default real-translation harnesses should verify one deterministic baseline, but
    `scripts/eval.sh`, `scripts/eval-capture.sh`, and `scripts/smoke.sh` currently rely on script
    conventions rather than dedicated regression coverage for glossary isolation. This makes the
    harness behavior easy to regress when config/env handling changes.
  - TODO: add focused regression tests for the harness scripts so we verify both the default
    no-glossary path and an explicit glossary-enabled path later.

- Ambiguous single-word translation behavior
  - Note: In plain non-glossary mode, some single-word inputs with multiple valid senses can cause
    TranslateGemma to return explanatory option lists instead of one direct translation. Example
    observed on 2026-03-24: `statement` for `en -> fr` returned a contextual explanation, while the
    glossary-constrained path correctly returned the banking term `releve de compte`.
  - TODO: tighten the non-glossary prompt or decoding behavior so ambiguous single-word requests
    still prefer one direct translation output, while preserving glossary-constrained behavior for
    domain-specific terms.

## Notes

- When debt becomes actionable, create or link an ExecPlan in `docs/execution-plans/active/` and
  update this tracker entry.
