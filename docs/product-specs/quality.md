# Product Quality Targets

## Risk Summary

| Risk | Impact | Mitigation |
|------|--------|------------|
| GGUF model availability/compatibility changes | High | Keep model assumptions explicit and support overrides |
| GPU setup complexity (especially WSL) | Medium | Maintain clear setup docs and CPU fallback |
| Inference backend API changes | Medium | Pin versions and run periodic upgrade validation |
| Translation quality regressions | Medium | Keep prompt/spec docs stable and test key language pairs |

## v1 Success Criteria

- Translate common pairs (EN↔FR, EN↔DE, EN↔ES, EN↔ZH, EN↔JA minimum)
- Sub-2-second response for short inputs on RTX 3060+ class hardware
- Reliable operation on WSL2 with CUDA
- Builds and runs on Linux, macOS, and Windows
- TUI controls remain clear and predictable
- Installation and usage documentation is complete
