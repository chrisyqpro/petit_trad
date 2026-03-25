// SPDX-License-Identifier: GPL-3.0-or-later

//! CLI argument parsing for petit-tui.

use anyhow::{Result, anyhow};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct CliArgs {
    pub model: Option<PathBuf>,
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
    pub gpu_layers: Option<u32>,
    pub context_size: Option<u32>,
    pub threads: Option<u32>,
    pub config: Option<PathBuf>,
    pub no_config: bool,
    pub stdin: bool,
    pub benchmark: bool,
    pub benchmark_text: Option<String>,
    pub benchmark_warmup_runs: Option<u32>,
    pub benchmark_runs: Option<u32>,
    pub benchmark_max_new_tokens: Option<u32>,
    pub glossary_enabled: Option<bool>,
    pub glossary_path: Option<PathBuf>,
    pub glossary_embedding_model_dir: Option<PathBuf>,
    pub glossary_max_matches: Option<usize>,
    pub show_version: bool,
    pub show_help: bool,
}

impl CliArgs {
    pub fn parse() -> Result<Self> {
        Self::parse_from(std::env::args().skip(1))
    }

    pub fn parse_from(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter();
        let mut cli = CliArgs::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--model" => cli.model = Some(parse_path(&mut args, "--model")?),
                "--source-lang" => {
                    cli.source_lang = Some(parse_string(&mut args, "--source-lang")?)
                }
                "--src" => cli.source_lang = Some(parse_string(&mut args, "--src")?),
                "--target-lang" => {
                    cli.target_lang = Some(parse_string(&mut args, "--target-lang")?)
                }
                "--tgt" => cli.target_lang = Some(parse_string(&mut args, "--tgt")?),
                "--gpu-layers" => cli.gpu_layers = Some(parse_u32(&mut args, "--gpu-layers")?),
                "--context-size" => {
                    cli.context_size = Some(parse_u32(&mut args, "--context-size")?)
                }
                "--threads" => cli.threads = Some(parse_u32(&mut args, "--threads")?),
                "--config" => cli.config = Some(parse_path(&mut args, "--config")?),
                "--no-config" => cli.no_config = true,
                "--stdin" => cli.stdin = true,
                "--benchmark" => cli.benchmark = true,
                "--text" => cli.benchmark_text = Some(parse_string(&mut args, "--text")?),
                "--warmup-runs" => {
                    cli.benchmark_warmup_runs = Some(parse_u32(&mut args, "--warmup-runs")?)
                }
                "--runs" => cli.benchmark_runs = Some(parse_u32(&mut args, "--runs")?),
                "--max-new-tokens" => {
                    cli.benchmark_max_new_tokens = Some(parse_u32(&mut args, "--max-new-tokens")?)
                }
                "--glossary" => set_glossary_enabled(&mut cli, true)?,
                "--no-glossary" => set_glossary_enabled(&mut cli, false)?,
                "--glossary-path" => {
                    cli.glossary_path = Some(parse_path(&mut args, "--glossary-path")?)
                }
                "--glossary-embedding-model-dir" => {
                    cli.glossary_embedding_model_dir =
                        Some(parse_path(&mut args, "--glossary-embedding-model-dir")?)
                }
                "--glossary-max-matches" => {
                    cli.glossary_max_matches =
                        Some(parse_usize(&mut args, "--glossary-max-matches")?)
                }
                "--version" | "-V" => cli.show_version = true,
                "--help" | "-h" => cli.show_help = true,
                unknown => return Err(anyhow!("Unknown argument: {unknown}")),
            }
        }

        if cli.no_config && cli.config.is_some() {
            return Err(anyhow!("--no-config cannot be used with --config"));
        }

        Ok(cli)
    }

    pub fn usage() -> &'static str {
        concat!(
            "petit - Local TranslateGemma TUI\n\n",
            "Usage:\n",
            "  petit [options]\n\n",
            "Options:\n",
            "  --model <path>         Path to GGUF model\n",
            "  --source-lang <code>   Source language (e.g. en)\n",
            "  --src <code>           Alias for --source-lang\n",
            "  --target-lang <code>   Target language (e.g. fr)\n",
            "  --tgt <code>           Alias for --target-lang\n",
            "  --gpu-layers <n>       GPU layers to offload\n",
            "  --context-size <n>     Context window size\n",
            "  --threads <n>          CPU threads for inference\n",
            "  --config <path>        Config file path\n",
            "  --no-config            Ignore config file\n",
            "  --stdin                Read text from stdin and exit\n",
            "  --benchmark            Run benchmark mode and exit (uses config precedence)\n",
            "  --text <value>         Benchmark input text (benchmark mode)\n",
            "  --warmup-runs <n>      Warmup runs before measured runs (benchmark mode)\n",
            "  --runs <n>             Measured benchmark runs (benchmark mode)\n",
            "  --max-new-tokens <n>   Max output tokens for benchmark run (benchmark mode)\n",
            "  --glossary             Enable glossary-constrained translation\n",
            "  --no-glossary          Disable glossary-constrained translation\n",
            "  --glossary-path <path> Path to glossary TSV file\n",
            "  --glossary-embedding-model-dir <path> Glossary embedding model directory\n",
            "  --glossary-max-matches <n> Max glossary candidates to inject\n",
            "  --version, -V          Print version\n",
            "  --help, -h             Print help\n"
        )
    }
}

fn parse_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("Missing value for {name}"))
}

fn parse_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(parse_string(args, name)?))
}

fn parse_u32(args: &mut impl Iterator<Item = String>, name: &str) -> Result<u32> {
    let value = parse_string(args, name)?;
    value
        .parse::<u32>()
        .map_err(|_| anyhow!("Invalid value for {name}: {value}"))
}

fn parse_usize(args: &mut impl Iterator<Item = String>, name: &str) -> Result<usize> {
    let value = parse_string(args, name)?;
    value
        .parse::<usize>()
        .map_err(|_| anyhow!("Invalid value for {name}: {value}"))
}

fn set_glossary_enabled(cli: &mut CliArgs, enabled: bool) -> Result<()> {
    if let Some(existing) = cli.glossary_enabled {
        if existing != enabled {
            return Err(anyhow!(
                "Conflicting glossary flags: --glossary and --no-glossary"
            ));
        }
        return Ok(());
    }

    cli.glossary_enabled = Some(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn usage_lists_glossary_flags() {
        let usage = CliArgs::usage();
        assert!(usage.contains("--glossary"));
        assert!(usage.contains("--no-glossary"));
        assert!(usage.contains("--glossary-path"));
        assert!(usage.contains("--glossary-embedding-model-dir"));
        assert!(usage.contains("--glossary-max-matches"));
    }

    #[test]
    fn parse_accepts_glossary_flags_and_values() {
        let cli = CliArgs::parse_from(args(&[
            "--glossary",
            "--glossary-path",
            "/tmp/glossary.tsv",
            "--glossary-embedding-model-dir",
            "/tmp/models/embeddinggemma-300m-ONNX",
            "--glossary-max-matches",
            "6",
        ]))
        .expect("glossary flags should parse");

        assert_eq!(cli.glossary_enabled, Some(true));
        assert_eq!(cli.glossary_path, Some(PathBuf::from("/tmp/glossary.tsv")));
        assert_eq!(
            cli.glossary_embedding_model_dir,
            Some(PathBuf::from("/tmp/models/embeddinggemma-300m-ONNX"))
        );
        assert_eq!(cli.glossary_max_matches, Some(6));
    }

    #[test]
    fn parse_rejects_conflicting_glossary_flags() {
        let err = CliArgs::parse_from(args(&["--glossary", "--no-glossary"]))
            .expect_err("conflicting flags should be rejected");
        assert!(
            err.to_string().contains("cannot") || err.to_string().contains("glossary"),
            "unexpected error: {err}"
        );
    }
}
