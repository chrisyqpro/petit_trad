// SPDX-License-Identifier: GPL-3.0-or-later

//! Config loading and precedence handling for petit-tui.

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use petit_core::language::{normalize_lang, validate_pair};
use petit_core::Config;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::cli::CliArgs;

#[derive(Debug, Default)]
pub struct AppConfig {
    pub core: Config,
    pub source_lang: String,
    pub target_lang: String,
    pub stdin_mode: bool,
    pub compact_lang_display: bool,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    model: ModelConfig,
    #[serde(default)]
    translation: TranslationConfig,
    #[serde(default)]
    ui: UiConfig,
}

#[derive(Debug, Deserialize, Default)]
struct ModelConfig {
    path: Option<PathBuf>,
    gpu_layers: Option<u32>,
    context_size: Option<u32>,
    threads: Option<u32>,
    log_to_file: Option<bool>,
    log_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
struct TranslationConfig {
    default_source: Option<String>,
    default_target: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct UiConfig {
    compact_lang_display: Option<bool>,
}

pub fn load_config(cli: &CliArgs) -> Result<AppConfig> {
    let mut core = Config::default();
    let mut source_lang = "en".to_string();
    let mut target_lang = "fr".to_string();
    let mut stdin_mode = cli.stdin;
    let mut compact_lang_display = false;

    let env_no_config = env_bool("PETIT_TRAD_NO_CONFIG");
    let mut no_config = env_no_config.unwrap_or(false);
    if cli.no_config {
        no_config = true;
    }

    if cli.config.is_some() {
        no_config = false;
    }

    let config_path = config_path(cli, no_config)?;
    if let Some(path) = config_path.as_deref() {
        if path.exists() {
            let file = load_file_config(path)?;
            apply_file_config(
                &file,
                &mut core,
                &mut source_lang,
                &mut target_lang,
                &mut compact_lang_display,
            );
        } else if config_path_is_explicit(cli) && !no_config {
            return Err(anyhow!("Config file not found: {}", path.display()));
        }
    }

    apply_env_config(
        &mut core,
        &mut source_lang,
        &mut target_lang,
        &mut compact_lang_display,
    );
    apply_cli_config(cli, &mut core, &mut source_lang, &mut target_lang);

    source_lang = normalize_lang(&source_lang);
    target_lang = normalize_lang(&target_lang);
    validate_pair(&source_lang, &target_lang)
        .map_err(|err| anyhow!("Invalid language pair: {err}"))?;

    if !stdin_mode && !std::io::stdin().is_terminal() {
        stdin_mode = true;
    }

    Ok(AppConfig {
        core,
        source_lang,
        target_lang,
        stdin_mode,
        compact_lang_display,
    })
}

fn config_path(cli: &CliArgs, no_config: bool) -> Result<Option<PathBuf>> {
    if no_config {
        return Ok(None);
    }

    if let Some(path) = cli.config.clone() {
        return Ok(Some(path));
    }

    if let Some(path) = env_var("PETIT_TRAD_CONFIG") {
        return Ok(Some(PathBuf::from(path)));
    }

    Ok(default_config_path())
}

fn config_path_is_explicit(cli: &CliArgs) -> bool {
    cli.config.is_some() || env_var("PETIT_TRAD_CONFIG").is_some()
}

fn default_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "petit", "petit_trad").map(|dirs| {
        let mut path = dirs.config_dir().to_path_buf();
        path.push("config.toml");
        path
    })
}

fn load_file_config(path: &Path) -> Result<FileConfig> {
    let content = fs::read_to_string(path)
        .map_err(|err| anyhow!("Failed to read config {}: {err}", path.display()))?;
    toml::from_str(&content)
        .map_err(|err| anyhow!("Failed to parse config {}: {err}", path.display()))
}

fn apply_file_config(
    file: &FileConfig,
    core: &mut Config,
    source: &mut String,
    target: &mut String,
    compact_lang_display: &mut bool,
) {
    if let Some(path) = &file.model.path {
        core.model_path = path.clone();
    }
    if let Some(gpu_layers) = file.model.gpu_layers {
        core.gpu_layers = gpu_layers;
    }
    if let Some(context_size) = file.model.context_size {
        core.context_size = context_size;
    }
    if let Some(threads) = file.model.threads {
        core.threads = threads;
    }
    if let Some(log_to_file) = file.model.log_to_file {
        core.log_to_file = log_to_file;
    }
    if let Some(log_path) = &file.model.log_path {
        core.log_path = log_path.clone();
    }

    if let Some(source_lang) = &file.translation.default_source {
        *source = source_lang.clone();
    }
    if let Some(target_lang) = &file.translation.default_target {
        *target = target_lang.clone();
    }
    if let Some(compact) = file.ui.compact_lang_display {
        *compact_lang_display = compact;
    }

}

fn apply_env_config(
    core: &mut Config,
    source: &mut String,
    target: &mut String,
    compact_lang_display: &mut bool,
) {
    if let Some(path) = env_var("PETIT_TRAD_MODEL") {
        core.model_path = PathBuf::from(path);
    }
    if let Some(value) = env_u32("PETIT_TRAD_GPU_LAYERS") {
        core.gpu_layers = value;
    }
    if let Some(value) = env_u32("PETIT_TRAD_CONTEXT_SIZE") {
        core.context_size = value;
    }
    if let Some(value) = env_u32("PETIT_TRAD_THREADS") {
        core.threads = value;
    }
    if let Some(value) = env_bool("PETIT_TRAD_LOG_TO_FILE") {
        core.log_to_file = value;
    }
    if let Some(value) = env_var("PETIT_TRAD_LOG_PATH") {
        core.log_path = PathBuf::from(value);
    }
    if let Some(value) = env_var("PETIT_TRAD_SOURCE_LANG") {
        *source = value;
    }
    if let Some(value) = env_var("PETIT_TRAD_TARGET_LANG") {
        *target = value;
    }
    if let Some(value) = env_bool("PETIT_TRAD_COMPACT_LANG") {
        *compact_lang_display = value;
    }
}

fn apply_cli_config(cli: &CliArgs, core: &mut Config, source: &mut String, target: &mut String) {
    if let Some(path) = &cli.model {
        core.model_path = path.clone();
    }
    if let Some(gpu_layers) = cli.gpu_layers {
        core.gpu_layers = gpu_layers;
    }
    if let Some(context_size) = cli.context_size {
        core.context_size = context_size;
    }
    if let Some(threads) = cli.threads {
        core.threads = threads;
    }
    if let Some(value) = &cli.source_lang {
        *source = value.clone();
    }
    if let Some(value) = &cli.target_lang {
        *target = value.clone();
    }
}

fn env_var(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.is_empty())
}

fn env_u32(key: &str) -> Option<u32> {
    env_var(key).and_then(|value| value.parse::<u32>().ok())
}

fn env_bool(key: &str) -> Option<bool> {
    env_var(key).and_then(|value| match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    })
}
