// SPDX-License-Identifier: GPL-3.0-or-later

//! Config loading and precedence handling for petit-tui.

use anyhow::{anyhow, Result};
use petit_core::language::{normalize_lang, validate_pair};
use petit_core::Config;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::cli::CliArgs;

#[derive(Debug)]
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
    let mut file = load_merged_file_config(cli)?;

    let mut core = Config {
        model_path: take_required(file.model.path.take(), "model.path")?,
        gpu_layers: take_required(file.model.gpu_layers.take(), "model.gpu_layers")?,
        context_size: take_required(file.model.context_size.take(), "model.context_size")?,
        threads: take_required(file.model.threads.take(), "model.threads")?,
        log_to_file: take_required(file.model.log_to_file.take(), "model.log_to_file")?,
        log_path: take_required(file.model.log_path.take(), "model.log_path")?,
    };
    let mut source_lang = take_required(
        file.translation.default_source.take(),
        "translation.default_source",
    )?;
    let mut target_lang = take_required(
        file.translation.default_target.take(),
        "translation.default_target",
    )?;
    let mut stdin_mode = cli.stdin;
    let mut compact_lang_display = file.ui.compact_lang_display.unwrap_or(false);

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

fn load_merged_file_config(cli: &CliArgs) -> Result<FileConfig> {
    let mut merged = load_file_config(Path::new("config/default.toml"))?;

    let env_no_config = env_bool("PETIT_TRAD_NO_CONFIG").unwrap_or(false);
    let no_user_config = cli.no_config || env_no_config;
    if no_user_config {
        return Ok(merged);
    }

    if let Some(path) = cli.config.clone() {
        if !path.exists() {
            return Err(anyhow!("Config file not found: {}", path.display()));
        }
        let overlay = load_file_config(&path)?;
        merge_file_config(&mut merged, overlay);
        return Ok(merged);
    }

    if let Some(path) = xdg_user_config_path() {
        if path.exists() {
            let overlay = load_file_config(&path)?;
            merge_file_config(&mut merged, overlay);
        }
    }

    Ok(merged)
}

fn merge_file_config(base: &mut FileConfig, overlay: FileConfig) {
    if overlay.model.path.is_some() {
        base.model.path = overlay.model.path;
    }
    if overlay.model.gpu_layers.is_some() {
        base.model.gpu_layers = overlay.model.gpu_layers;
    }
    if overlay.model.context_size.is_some() {
        base.model.context_size = overlay.model.context_size;
    }
    if overlay.model.threads.is_some() {
        base.model.threads = overlay.model.threads;
    }
    if overlay.model.log_to_file.is_some() {
        base.model.log_to_file = overlay.model.log_to_file;
    }
    if overlay.model.log_path.is_some() {
        base.model.log_path = overlay.model.log_path;
    }

    if overlay.translation.default_source.is_some() {
        base.translation.default_source = overlay.translation.default_source;
    }
    if overlay.translation.default_target.is_some() {
        base.translation.default_target = overlay.translation.default_target;
    }
    if overlay.ui.compact_lang_display.is_some() {
        base.ui.compact_lang_display = overlay.ui.compact_lang_display;
    }
}

fn take_required<T>(value: Option<T>, field: &str) -> Result<T> {
    value.ok_or_else(|| anyhow!("Missing required config value: {field}"))
}

fn xdg_user_config_path() -> Option<PathBuf> {
    let base = env_var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env_var("HOME").map(|home| PathBuf::from(home).join(".config")));

    base.map(|base| {
        let mut path = base;
        path.push("petit_trad");
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
