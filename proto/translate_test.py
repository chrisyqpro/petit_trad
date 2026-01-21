#!/usr/bin/env python3
"""TranslateGemma GGUF prototype - tests llama-cpp-python inference."""

import time
from pathlib import Path

from llama_cpp import Llama

# Model path
MODEL_PATH = Path(__file__).parent.parent / "models" / "translategemma-12b-it-GGUF"
GGUF_FILE = MODEL_PATH / "translategemma-12b-it.Q8_0.gguf"


def load_model(model_path: Path, n_gpu_layers: int = -1, n_ctx: int = 2048):
    """Load GGUF model with llama-cpp."""
    print(f"Loading model from: {model_path}")
    model = Llama(
        model_path=str(model_path),
        n_gpu_layers=n_gpu_layers,  # -1 = all layers on GPU
        n_ctx=n_ctx,
        verbose=False,
    )
    return model


def build_prompt(text: str, source_lang: str, target_lang: str) -> str:
    """Build TranslateGemma prompt.

    The model works best with a direct translation instruction.
    """
    # Direct translation request without extra context
    prompt = (
        f"<start_of_turn>user\n"
        f"[{source_lang}->{target_lang}] {text}<end_of_turn>\n"
        f"<start_of_turn>model\n"
    )
    return prompt


def translate(
    model: Llama,
    text: str,
    source_lang: str,
    target_lang: str,
    max_tokens: int = 256,
) -> tuple[str, float]:
    """Translate text and return (translation, time_seconds)."""
    prompt = build_prompt(text, source_lang, target_lang)

    start = time.perf_counter()
    output = model(
        prompt,
        max_tokens=max_tokens,
        stop=["<end_of_turn>", "<eos>"],
        echo=False,
    )
    elapsed = time.perf_counter() - start

    translation: str = output["choices"][0]["text"].strip()  # type: ignore[index]
    return translation, elapsed


def main():
    print("=" * 60)
    print("TranslateGemma GGUF Prototype (llama-cpp)")
    print("=" * 60)

    if not GGUF_FILE.exists():
        print(f"ERROR: Model not found at {GGUF_FILE}")
        return

    # Load model
    print("\nLoading model...")
    model = load_model(GGUF_FILE)
    print("Model loaded successfully")

    # Test translations - use ISO 639-1 codes
    test_cases = [
        ("Hello, how are you?", "en", "fr"),
        ("The weather is nice today.", "en", "de"),
        ("I love programming.", "en", "es"),
        ("Good morning.", "en", "zh"),
        ("Thank you very much.", "en", "ja"),
        ("Je suis developpeur.", "fr", "en"),
        ("Ich lerne Rust.", "de", "en"),
    ]

    print("\n" + "=" * 60)
    print("Translation Tests")
    print("=" * 60)

    results = []
    for text, src, tgt in test_cases:
        print(f"\n[{src} -> {tgt}]")
        print(f"  Input:  {text}")
        translation, elapsed = translate(model, text, src, tgt)
        print(f"  Output: {translation}")
        print(f"  Time:   {elapsed:.2f}s")
        results.append((src, tgt, text, translation, elapsed))

    # Summary
    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)
    total_time = sum(r[4] for r in results)
    avg_time = total_time / len(results)
    print(f"Total tests: {len(results)}")
    print(f"Average time: {avg_time:.2f}s")
    print(f"Total time: {total_time:.2f}s")


if __name__ == "__main__":
    main()
