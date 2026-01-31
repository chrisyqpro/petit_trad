// SPDX-License-Identifier: GPL-3.0-or-later

use petit_core::{Config, GemmaTranslator, Result, Translator};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn parse_arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}

fn parse_u32(args: &[String], name: &str, default: u32) -> u32 {
    parse_arg(args, name)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let model_path = parse_arg(&args, "--model").unwrap_or_else(|| {
        "models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf".to_string()
    });
    let source_lang = parse_arg(&args, "--src").unwrap_or_else(|| "en".to_string());
    let target_lang = parse_arg(&args, "--tgt").unwrap_or_else(|| "fr".to_string());
    let text = parse_arg(&args, "--text").unwrap_or_else(|| "Hello, how are you?".to_string());

    let gpu_layers = parse_u32(&args, "--gpu-layers", 999);
    let context_size = parse_u32(&args, "--ctx", 2048);
    let threads = parse_u32(&args, "--threads", 4);

    let config = Config {
        model_path: PathBuf::from(model_path),
        gpu_layers,
        context_size,
        threads,
        log_to_file: false,
        log_path: PathBuf::from("logs/llama.log"),
    };

    let translator = GemmaTranslator::new(config)?;

    let start = Instant::now();
    let output = translator.translate(&text, &source_lang, &target_lang)?;
    let elapsed = start.elapsed();

    println!("Source: {text}");
    println!("Target: {output}");
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
