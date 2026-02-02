# petit_trad — Project Plan

## Overview

Local translation tool using TranslateGemma. See [doc/architecture.md](../doc/architecture.md) for technical design.

---

## Progress

### Phase 0: Project Initialization (done)

- [x] **T0.1** Create directory structure
- [x] **T0.2** Initialize Cargo workspace with `petit-core` and `petit-tui` crates
- [x] **T0.3** Create `.gitignore`
- [x] **T0.4** Create `AGENTS.md`
- [x] **T0.5** Create `doc/architecture.md`

### Phase 1: Python Prototype

- [x] **T1.1** Set up Python environment (`proto/requirements.txt`: transformers, torch, accelerate)
- [x] **T1.2** Write `translate_test.py` to load TranslateGemma and test translation
- [x] **T1.3** Document discovered prompt format in `doc/prompt-format.md`
- [x] **T1.4** Test with various language pairs, note quality and edge cases
- [x] **T1.5** Verify CUDA inference works in WSL environment

### Phase 2: Rust Core Library (`petit-core`)

- [x] **T2.1** Add dependencies: `llama-cpp-2`, `thiserror`, `serde`, `toml`
- [x] **T2.2** Implement `Config` struct for model path, GPU layers, context size
- [x] **T2.3** Implement `ModelManager` — load GGUF model with llama-cpp-2
- [x] **T2.4** Define `Translator` trait with `translate()` method
- [x] **T2.5** Implement `GemmaTranslator` using prompt format from Phase 1
- [x] **T2.6** Add language detection/validation utilities
- [x] **T2.7** Write unit tests with mock model / small test model
- [x] **T2.8** Test CUDA inference in WSL

Notes:
- 2026-01-20: CUDA build succeeds when `CUDAToolkit_ROOT=/usr/local/cuda` and `nvcc` on PATH.
- GPU visible in WSL: RTX 5070 Ti, driver 591.74, CUDA 13.1.
- Example run: `cargo run -p petit-core --example translate --features cuda` with
   models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf.
- Translation output: "Bonjour, comment allez-vous ?" for "Hello, how are you?".
- Latency: 1.16s (first run, includes model load).
- `nvidia-smi` after run: GPU util 3%, memory used 2541 MiB.

#### Phase 2 Detailed Spec (proposed)

Goal: finish `petit-core` so the TUI can call a real translator with llama-cpp-2.

Phase 2A: Dependency and feature wiring
- Un-comment and pin `llama-cpp-2` in workspace `Cargo.toml` and add it to
   `crates/petit-core/Cargo.toml`.
- Map workspace features to backend selection (`cuda`, `metal`, `vulkan`, `cpu-only`).
- Decide default feature set (recommend none, explicit flags required).
- Acceptance: `cargo build -p petit-core` succeeds on CPU-only without GPU libs.

Phase 2B: ModelManager (model load + inference)
- Add `model.rs` (or `model_manager.rs`) in `petit-core`.
- `ModelManager::new(config: Config) -> Result<Self>` loads GGUF model from
   `config.model_path` and validates file exists.
- Provide `infer(&self, prompt: &str, max_new_tokens: u32) -> Result<String>`.
- Encapsulate llama-cpp-2 state so callers do not depend on llama-cpp-2 APIs.
- Map llama errors into `Error::ModelLoad` or `Error::Inference`.
- Acceptance: model load + inference callable from a simple unit or example
   (guarded behind feature or ignored test if model missing).

Phase 2C: GemmaTranslator implementation
- Add `translator/gemma.rs` or `gemma.rs` implementing `Translator`.
- Build prompt using direct format from `doc/prompt-format.md`:
   `<start_of_turn>user\n[${src}->${tgt}] ${text}<end_of_turn>\n<start_of_turn>model`.
- Set deterministic generation: temperature 0, top_p 1.0, top_k 0, repeat
   penalty minimal, and stop tokens if llama-cpp-2 exposes them.
- Strip trailing whitespace and any model echo of prompt.
- Acceptance: `translate()` returns clean translation string.

Phase 2D: Language utilities
- Add `language.rs` with supported language list (ISO 639-1) and helpers:
   `normalize_lang`, `is_supported`, `validate_pair`.
- Accept region codes (e.g. `en-US`), normalize to lowercase + preserve region.
- Decide behavior for unsupported language: return `Error::UnsupportedLanguage`.
- Acceptance: unit tests for normalization and validation.

Phase 2E: Tests and config alignment
- Add unit tests for prompt formatting and language validation.
- Add config parsing tests (TOML round-trip, defaults).
- Align default model path with repo model layout or switch to a safe placeholder
   and require CLI/config to specify the path.
- Acceptance: `cargo test -p petit-core` passes without a model file.

Phase 2F: WSL CUDA validation
- Manual step: run a sample translation with CUDA-enabled build and measure
   basic latency to ensure GPU is used.

Suggested sub-phase split for multiple agents
- Sub-phase 2A: Dependency wiring + ModelManager skeleton (Phase 2A + 2B).
- Sub-phase 2B: GemmaTranslator + prompt formatting (Phase 2C).
- Sub-phase 2C: Language utilities + tests + config alignment (Phase 2D + 2E).
- Sub-phase 2D: CUDA validation steps (Phase 2F, manual).

### Phase 3: TUI Frontend (`petit-tui`)

- [x] **T3.1** Add dependencies: `ratatui`, `crossterm`, `tokio` (optional async)
- [x] **T3.2** Implement basic app loop (init terminal, event loop, restore terminal)
- [x] **T3.3** Create UI layout: input area, output area, language selectors, status bar
- [x] **T3.4** Implement input handling (typing, paste, language switching, quit)
- [x] **T3.5** Integrate `petit-core` — call translation on submit
- [x] **T3.6** Add loading indicator during inference
- [x] **T3.7** Support CLI arguments (`--model`, `--source-lang`, `--target-lang`)
- [x] **T3.8** Support piped stdin for scripting (`echo "hello" | petit --target-lang fr`)
- [x] **T3.9** Add configuration file support (`~/.config/petit_trad/config.toml`)

#### Phase 3 Detailed Spec (updated)

Phase 3A: Terminal + app loop (done)
- Initialize terminal with crossterm raw mode and alternate screen; always restore on exit or error.
- Implement an event loop using `event::poll` with a tick (100-200ms) for redraws/spinner.
- Add safe teardown paths (drop guard or `panic::set_hook`) so the terminal is never left raw.
- Extend App state with focus (input vs output), cursor position, scroll offsets, and status text.
- Acceptance: TUI opens/closes cleanly on Linux/WSL without flicker or stuck terminal.

Phase 3B: UI layout + rendering
- Layout: header/language bar, input pane, output pane, status bar (vertical split).
- Use `Paragraph` with wrapping and scroll; show cursor in active input pane.
- Highlight active pane and show language pair in header; show key hints in status bar.
- Show loading indicator (spinner + "Translating...") when `is_loading` is true.
- Acceptance: multi-line input/output display correctly with scroll controls.

Phase 3C: Input handling (default keybindings)
- Text editing: insert, backspace/delete, newline, cursor movement, and paste handling.
- Keybindings: Ctrl+q quit, Ctrl+Enter translate, Ctrl+r swap languages,
  Ctrl+l clear, Tab toggle focus.
- Language selection controls (pair cycling or direct edits) with validation via petit-core.
- On invalid language, restore the previous value and show a status message.
- Acceptance: input editing and language switching are stable with clear status feedback.

Phase 3D: Translation integration + loading
- Run translations on a worker thread that owns `GemmaTranslator`; communicate via channels.
- Allow only one request at a time; ignore or reject new submits until the current job finishes.
- Update output or status on success/error; keep UI responsive during inference.
- Track `is_loading` in App and disable submit until the job completes.
- Acceptance: translating does not block the UI and refuses overlapping requests.

Phase 3E: CLI + config + stdin
- Use the actual `config/default.toml` schema: `[model]`, `[translation]`, `[ui]`.
- Map config to `petit-core::Config` plus TUI defaults (source/target, UI prefs).
- Precedence: CLI args > env vars > config file > defaults.
- CLI flags: `--model`, `--source-lang`, `--target-lang`, `--gpu-layers`,
  `--context-size`, `--threads`, `--config`, `--no-config`, `--stdin`, `--version`.
- Load config from `~/.config/petit_trad/config.toml` using a cross-platform path helper.
- Support piped stdin or `--stdin` for one-shot translation, then exit.
- Acceptance: `petit --help` works; piped input returns a single translation.

### Phase 4: Cross-Platform Build & CI

- [ ] **T4.1** Configure Cargo features: `cuda`, `metal`, `vulkan`, `cpu-only`
- [ ] **T4.2** Test build on WSL with CUDA (first)
- [ ] **T4.3** Test build on macOS (Metal backend) (second)
- [ ] **T4.4** Set up GitHub Actions CI (third)
- [ ] **T4.5** Create release workflow with pre-built binaries (fourth)
- [ ] **T4.6** Test build on native Linux (deferred)
- [ ] **T4.7** Test build on Windows (CUDA or Vulkan) (deferred)

#### Phase 4 Detailed Spec (re-ordered)

Phase 4A: Feature wiring and build entry points
- Forward `cuda`, `metal`, `vulkan`, `cpu-only` features from `petit-tui` to `petit-core`.
- Document usage (`cargo run -p petit-tui --features cuda` etc.) in README or build guide.
- Decide `cpu-only` behavior and ensure it maps to `petit-core` defaults.

Phase 4B: WSL CUDA validation (first)
- Build and run with `--features cuda` on WSL2.
- Confirm model loads, translation succeeds, and UI stays intact.
- Record exact env vars and required toolkit/driver versions.

Phase 4C: macOS Metal validation (second)
- Build and run with `--features metal` on Apple Silicon.
- Confirm model loads, translation succeeds, and UI stays intact.
- Record toolchain/Xcode requirements if needed.

Phase 4D: GitHub Actions CI (third)
- Add CI workflow for Linux and macOS CPU builds/tests.
- Cache Cargo and llama.cpp artifacts where possible.
- Document any GPU tests as local-only for now.

Phase 4E: Release workflow (fourth)
- Add release workflow to build and upload binaries for Linux/macOS CPU.
- Define tag/version inputs and artifact naming.

Phase 4F: Deferred platforms
- Native Linux GPU and native Windows GPU testing deferred until after CI/release setup.

### Phase 5: Polish & Documentation

- [ ] **T5.1** Write `README.md` with installation, usage, build instructions
- [ ] **T5.2** Write `doc/build-guide.md` for development setup
- [ ] **T5.3** Add `--help` with comprehensive CLI documentation
- [ ] **T5.4** Performance profiling and optimization
- [ ] **T5.5** Error messages and user feedback improvements

### Phase 6 (Future): GUI Frontend

- [ ] **T6.1** Evaluate Tauri vs Electron vs native (egui)
- [ ] **T6.2** Create `petit-gui` crate or separate repo
- [ ] **T6.3** Design UI mockups
- [ ] **T6.4** Implement using `petit-core` as shared library

---

## Considerations & Open Questions

### Model & Inference

1. **Model acquisition strategy**
   - Option A: User manually downloads GGUF from Hugging Face
   - Option B: Built-in download command (`petit model download`)
   - Option C: First-run wizard that fetches model
   - **Recommendation**: Start with A, add B later

2. **Model size selection**
   - **4B** (~2.5GB Q4): Fast, fits any GPU, lower quality
   - **12B** (~7GB Q4): **Default** — balanced quality/speed, fits 16GB VRAM
   - **27B** (~16GB Q4): Best quality, needs 16GB+ VRAM (macOS 64GB target)
   - **Platform defaults**: 12B for Windows/WSL, 27B optional for macOS

3. **Quantization options**
   - Q4_K_M: Good quality/size balance (default)
   - Q5_K_M: Slightly better quality
   - Q8_0: Near-FP16 quality, larger size

4. **Prompt format**
   - TranslateGemma has specific prompt template (TBD in prototype)
   - **Action**: Document in T1.3 after prototype testing

### Platform-Specific

5. **WSL CUDA access**
   - Requires WSL2 with NVIDIA driver support
   - Need to document setup steps in build guide
   - **Action**: Verify in T1.5, document in T5.2

6. **Windows native PowerShell**
   - crossterm supports Windows Console API
   - CUDA requires NVIDIA drivers + CUDA toolkit
   - May need different terminal initialization

7. **macOS Apple Silicon**
   - Metal backend via llama.cpp
   - Need to test on M1/M2/M3 hardware

### Architecture

8. **Sync vs Async inference**
   - TUI can block during inference (show spinner)
   - Server/GUI need async to stay responsive
   - **Decision**: `petit-core` provides sync API; TUI runs in thread; server wraps in async

9. **Model hot-reloading**
   - Allow switching models without restart?
   - Adds complexity to ModelManager
   - **Decision**: Defer to future version

10. **Multiple simultaneous translations**
    - Batch translation for documents?
    - Queue system?
    - **Decision**: Single translation for v1; batch as future feature

### UX

11. **Language code format**
    - ISO 639-1 (en, fr, de) vs full names (English, French, German)?
    - **Recommendation**: Accept both, normalize internally

12. **History / clipboard integration**
    - Save translation history?
    - Auto-copy to clipboard?
    - **Decision**: Add as optional features in Phase 5

13. **Configuration precedence**
    - CLI args > env vars > config file > defaults
    - Standard pattern, implement in T3.9

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| TranslateGemma not available as GGUF | High | Use base Gemma 2B with translation prompts; community GGUF conversions |
| CUDA issues in WSL | Medium | Document setup thoroughly; provide CPU fallback |
| llama-cpp-2 API changes | Medium | Pin version; monitor upstream |
| ratatui breaking changes | Low | Actively maintained; pin version |
| Translation quality issues | Medium | Experiment with prompts; consider larger models |

---

## Success Criteria (v1.0)

- [ ] Translate text between common language pairs (EN↔FR, EN↔DE, EN↔ES, EN↔ZH, EN↔JA minimum)
- [ ] Sub-2-second response time for short sentences on RTX 3060+
- [ ] Works reliably on WSL2 with CUDA
- [ ] Builds and runs on Linux, macOS, Windows
- [ ] Clean TUI with intuitive controls
- [ ] Documented installation and usage

---

*Last updated: 2026-01-31*
