#!/usr/bin/env python3
"""TranslateGemma prototype - validates model behavior and prompt format."""

import time
from pathlib import Path

import torch
from llama_cpp import Llama
from transformers import AutoTokenizer

# Model paths
HF_MODEL_DIR = Path(__file__).parent.parent / "models" / "translategemma-12b-it"
GGUF_DIR = Path(__file__).parent.parent / "models" / "translategemma-12b-it-GGUF"
GGUF_QUANT = "Q8_0"


def find_gguf_file(gguf_dir: Path, quant: str) -> Path | None:
    if not gguf_dir.exists():
        return None
    matches = sorted(gguf_dir.glob(f"*{quant}*.gguf"))
    if matches:
        return matches[0]
    return None


def load_tokenizer(tokenizer_dir: Path):
    if tokenizer_dir.exists():
        return AutoTokenizer.from_pretrained(tokenizer_dir)
    raise FileNotFoundError(
        f"Tokenizer directory not found: {tokenizer_dir}. Download HF model files first."
    )


def load_gguf_model(gguf_path: Path) -> Llama:
    print(f"Loading GGUF model: {gguf_path}")
    return Llama(
        model_path=str(gguf_path),
        n_ctx=2048,
        n_gpu_layers=-1,
        n_threads=0,
        logits_all=False,
        embedding=False,
        verbose=False,
    )


def build_prompt(tokenizer, text: str, source_lang: str, target_lang: str) -> str:
    messages = [
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "source_lang_code": source_lang,
                    "target_lang_code": target_lang,
                    "text": text,
                }
            ],
        }
    ]
    return tokenizer.apply_chat_template(
        messages,
        tokenize=False,
        add_generation_prompt=True,
    )


def translate_gguf(
    llm: Llama,
    tokenizer,
    text: str,
    source_lang: str,
    target_lang: str,
    max_new_tokens: int = 256,
) -> tuple[str, float]:
    prompt = build_prompt(tokenizer, text, source_lang, target_lang)
    start = time.perf_counter()
    output = llm(
        prompt,
        max_tokens=max_new_tokens,
        temperature=0.0,
        stop=["<end_of_turn>", "</s>", "<eos>"] if "<end_of_turn>" in prompt else None,
    )
    elapsed = time.perf_counter() - start
    text_out = output["choices"][0]["text"].strip()
    return text_out, elapsed


def main():
    print("=" * 60)
    print("TranslateGemma Prototype")
    print("=" * 60)

    # Check CUDA
    print(f"\nCUDA available: {torch.cuda.is_available()}")
    if torch.cuda.is_available():
        print(f"CUDA device: {torch.cuda.get_device_name(0)}")
        print(f"VRAM: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f} GB")

    # Load tokenizer and GGUF model
    print("\nLoading tokenizer...")
    tokenizer = load_tokenizer(HF_MODEL_DIR)
    print("Tokenizer loaded")

    gguf_path = find_gguf_file(GGUF_DIR, GGUF_QUANT)
    if gguf_path is None:
        raise FileNotFoundError(
            f"GGUF file not found in {GGUF_DIR}. Expected a file with '{GGUF_QUANT}' "
            "in the name."
        )
    print("\nLoading GGUF model...")
    llm = load_gguf_model(gguf_path)
    print("GGUF model loaded successfully")

    # Test translations - use ISO 639-1 language codes
    test_cases = [
        ("Hello, how are you?", "en", "fr"),
        ("The weather is nice today.", "en", "de"),
        ("I love programming.", "en", "es"),
    ]

    print("\n" + "=" * 60)
    print("Translation Tests")
    print("=" * 60)

    for text, src, tgt in test_cases:
        print(f"\n[{src} -> {tgt}]")
        print(f"  Input:  {text}")
        translation, elapsed = translate_gguf(llm, tokenizer, text, src, tgt)
        print(f"  Output: {translation}")
        print(f"  Time:   {elapsed:.2f}s")

    # VRAM usage after inference
    if torch.cuda.is_available():
        print(f"\nVRAM used: {torch.cuda.memory_allocated() / 1e9:.1f} GB")


if __name__ == "__main__":
    main()
