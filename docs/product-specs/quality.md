# Product Quality Targets

## Risk Summary

<!-- markdownlint-disable MD013 -->

| Risk                                          | Impact | Mitigation                                                                                      |
| --------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------- |
| GGUF model availability/compatibility changes | High   | Keep model assumptions explicit and support overrides                                           |
| GPU setup complexity (especially WSL)         | Medium | Maintain clear setup docs and CPU fallback                                                      |
| Inference backend API changes                 | Medium | Pin versions and run periodic upgrade validation                                                |
| Translation quality regressions               | Medium | Keep prompt/spec docs stable and test key language pairs                                        |
| Glossary retrieval false positives            | Medium | Use exact-match promotion, deterministic shortlist caps, and prompt-level tests                 |
| Missing local glossary embedding model assets | Medium | Keep the model directory explicit, fail fast on missing files, and document the required layout |

## v1 Success Criteria

- Translate common pairs (EN↔FR, EN↔DE, EN↔ES, EN↔ZH, EN↔JA minimum)
- Sub-2-second response for short inputs on RTX 3060+ class hardware
- Reliable operation on WSL2 with CUDA
- Builds and runs on Linux, macOS, and Windows
- TUI controls remain clear and predictable
- Installation and usage documentation is complete
- Glossary-enabled translation improves terminology consistency without breaking baseline
  translation behavior when the feature is disabled
