# TranslateGemma Prompt Format

Discovered conventions for TranslateGemma models based on prototype testing.

## Model

- **HuggingFace IDs:**
  - `google/translategemma-4b-it` (4B parameters)
  - `google/translategemma-12b-it` (12B parameters)
  - `google/translategemma-27b-it` (27B parameters)
- **Architecture:** Gemma 3 based
- **License:** Gemma (gated, requires HF authentication)

## Prompt Template

TranslateGemma supports two prompt formats:

### 1. HuggingFace Structured Format (transformers)

For use with `tokenizer.apply_chat_template()`:

```python
messages = [
    {
        "role": "user",
        "content": [
            {
                "type": "text",
                "source_lang_code": "en",      # ISO 639-1 code
                "target_lang_code": "fr",      # ISO 639-1 code
                "text": "Hello, how are you?",
            }
        ],
    }
]

prompt = tokenizer.apply_chat_template(
    messages,
    tokenize=False,
    add_generation_prompt=True,
)
```

### 2. Direct Format (llama-cpp / GGUF)

For use with llama.cpp or raw inference - simpler and produces cleaner output:

```
<start_of_turn>user
[en->fr] Hello, how are you?<end_of_turn>
<start_of_turn>model
```

This format:
- Uses `[source->target]` notation with ISO 639-1 codes
- Produces direct translations without explanations
- Works well with GGUF quantized models

### Language Codes

Use ISO 639-1 Alpha-2 codes, optionally with region:
- Simple: `en`, `fr`, `de`, `es`, `zh`, `ja`
- Regional: `en-US`, `en-GB`, `de-DE`, `pt-BR`

Supported languages: 55 total (see model card for full list).

## Generation Config

From model's `generation_config.json`:

```json
{
  "max_length": 2048
}
```

Recommended settings:

```python
outputs = model.generate(
    **inputs,
    max_new_tokens=256,      # Adjust based on expected output length
    do_sample=False,         # Deterministic output
    pad_token_id=tokenizer.eos_token_id,
)
```

Note: The model ignores `top_p` and `top_k` parameters (warning emitted).

## Output Parsing

The model returns translation directly. Extract new tokens only:

```python
response = tokenizer.decode(
    outputs[0][inputs["input_ids"].shape[1]:],
    skip_special_tokens=True,
)
translation = response.strip()
```

## Performance Notes

### HuggingFace Transformers (FP16)

Tested on RTX 5070 Ti (17GB VRAM) with 12B model:

| Metric | Value |
|--------|-------|
| Model size (FP16) | ~24GB |
| VRAM used | 13.6GB (partial CPU offload) |
| Inference time | ~230s per sentence (with offload) |

The 12B FP16 model exceeds typical consumer VRAM.

### llama-cpp (GGUF Q8_0)

Tested with 12B Q8_0 quantized model (12GB) on RTX 5070 Ti:

| Metric | CPU | CUDA |
|--------|-----|------|
| Model size (Q8_0) | ~12GB | ~12GB |
| Inference time | ~2s per sentence | ~0.28s per sentence |
| Translation quality | Excellent | Excellent |

For practical use:
- Use Python 3.12 for prebuilt CUDA wheels
- Use GGUF quantized versions (Q4_K_M ~7GB, Q8_0 ~12GB for 12B)
- Set LD_LIBRARY_PATH to PyTorch's CUDA libs for llama-cpp CUDA support

## Image Translation

TranslateGemma also supports OCR + translation from images:

```python
messages = [
    {
        "role": "user",
        "content": [
            {
                "type": "image",
                "source_lang_code": "ja",
                "target_lang_code": "en",
                "url": "path/to/image.png",
            }
        ],
    }
]
```

Not tested in prototype; focus is text-to-text translation.

---

*Created: 2026-01-20*
*Based on: google/translategemma-12b-it*
