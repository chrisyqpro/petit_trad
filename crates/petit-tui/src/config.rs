// SPDX-License-Identifier: GPL-3.0-or-later

//! Config loading and precedence handling for petit-tui.

use anyhow::{Result, anyhow};
use petit_core::Config;
use petit_core::config::GlossaryConfig as CoreGlossaryConfig;
use petit_core::language::{normalize_lang, validate_pair};
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
    #[serde(default)]
    glossary: GlossaryFileConfig,
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

#[derive(Debug, Deserialize, Default)]
struct GlossaryFileConfig {
    enabled: Option<bool>,
    path: Option<PathBuf>,
    embedding_model_dir: Option<PathBuf>,
    max_matches: Option<usize>,
}

impl GlossaryFileConfig {
    fn into_core(self) -> CoreGlossaryConfig {
        CoreGlossaryConfig {
            enabled: self.enabled.unwrap_or(false),
            path: self.path.unwrap_or_default(),
            embedding_model_dir: self.embedding_model_dir.unwrap_or_default(),
            max_matches: self.max_matches.unwrap_or_default(),
        }
    }
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
        glossary: file.glossary.into_core(),
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

    if let Some(path) = xdg_user_config_path()
        && path.exists()
    {
        let overlay = load_file_config(&path)?;
        merge_file_config(&mut merged, overlay);
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
    if let Some(value) = overlay.glossary.enabled {
        base.glossary.enabled = Some(value);
    }
    if let Some(value) = overlay.glossary.path {
        base.glossary.path = Some(value);
    }
    if let Some(value) = overlay.glossary.embedding_model_dir {
        base.glossary.embedding_model_dir = Some(value);
    }
    if let Some(value) = overlay.glossary.max_matches {
        base.glossary.max_matches = Some(value);
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
    let mut config: FileConfig = toml::from_str(&content)
        .map_err(|err| anyhow!("Failed to parse config {}: {err}", path.display()))?;
    expand_file_config_paths(&mut config);
    Ok(config)
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
    if let Some(value) = env_bool("PETIT_TRAD_GLOSSARY_ENABLED") {
        core.glossary.enabled = value;
    }
    if let Some(value) = env_var("PETIT_TRAD_GLOSSARY_PATH") {
        core.glossary.path = PathBuf::from(value);
    }
    if let Some(value) = env_var("PETIT_TRAD_GLOSSARY_EMBEDDING_MODEL_DIR") {
        core.glossary.embedding_model_dir = PathBuf::from(value);
    }
    if let Some(value) = env_usize("PETIT_TRAD_GLOSSARY_MAX_MATCHES") {
        core.glossary.max_matches = value;
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
    if let Some(value) = cli.glossary_enabled {
        core.glossary.enabled = value;
    }
    if let Some(path) = &cli.glossary_path {
        core.glossary.path = path.clone();
    }
    if let Some(path) = &cli.glossary_embedding_model_dir {
        core.glossary.embedding_model_dir = path.clone();
    }
    if let Some(value) = cli.glossary_max_matches {
        core.glossary.max_matches = value;
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

fn env_usize(key: &str) -> Option<usize> {
    env_var(key).and_then(|value| value.parse::<usize>().ok())
}

fn expand_file_config_paths(config: &mut FileConfig) {
    if let Some(path) = config.model.path.take() {
        config.model.path = Some(expand_home_path(path));
    }
    if let Some(path) = config.model.log_path.take() {
        config.model.log_path = Some(expand_home_path(path));
    }
    if let Some(path) = config.glossary.path.take() {
        config.glossary.path = Some(expand_home_path(path));
    }
    if let Some(path) = config.glossary.embedding_model_dir.take() {
        config.glossary.embedding_model_dir = Some(expand_home_path(path));
    }
}

fn expand_home_path(path: PathBuf) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path;
    };
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return path;
    };

    match raw {
        "~" | "$HOME" | "${HOME}" => return home,
        _ => {}
    }

    if let Some(rest) = raw.strip_prefix("~/") {
        return home.join(rest);
    }
    if let Some(rest) = raw.strip_prefix("$HOME/") {
        return home.join(rest);
    }
    if let Some(rest) = raw.strip_prefix("${HOME}/") {
        return home.join(rest);
    }

    path
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct CwdGuard {
        old_cwd: PathBuf,
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.old_cwd);
        }
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: impl Into<String>) -> Self {
            let previous = env::var(key).ok();
            unsafe {
                env::set_var(key, value.into());
            }
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            unsafe {
                env::remove_var(key);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => unsafe {
                    env::set_var(self.key, value);
                },
                None => unsafe {
                    env::remove_var(self.key);
                },
            }
        }
    }

    fn write_temp_config(contents: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "petit-trad-glossary-config-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, contents).expect("temp config should be writable");
        path
    }

    fn with_repo_root() -> (PathBuf, PathBuf) {
        let old_cwd = env::current_dir().expect("current dir should be readable");
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("crate dir should have a parent")
            .to_path_buf();
        env::set_current_dir(&repo_root).expect("repo root should be settable");
        (old_cwd, repo_root)
    }

    #[test]
    fn load_config_should_apply_glossary_precedence_over_env_and_file() {
        let _guard = env_guard();
        let (old_cwd, _repo_root) = with_repo_root();
        let _cwd_guard = CwdGuard { old_cwd };
        let config_path = write_temp_config(
            r#"
[glossary]
enabled = false
path = "config/glossary.tsv"
max_matches = 3
"#,
        );

        let _enabled = EnvVarGuard::set("PETIT_TRAD_GLOSSARY_ENABLED", "true");
        let _path = EnvVarGuard::set("PETIT_TRAD_GLOSSARY_PATH", "/env/glossary.tsv");
        let _model_dir = EnvVarGuard::set(
            "PETIT_TRAD_GLOSSARY_EMBEDDING_MODEL_DIR",
            "/env/models/embeddinggemma-300m-ONNX",
        );
        let _max_matches = EnvVarGuard::set("PETIT_TRAD_GLOSSARY_MAX_MATCHES", "7");

        let cli = CliArgs {
            config: Some(config_path.clone()),
            glossary_enabled: Some(false),
            glossary_path: Some(PathBuf::from("/cli/glossary.tsv")),
            glossary_embedding_model_dir: Some(PathBuf::from(
                "/cli/models/embeddinggemma-300m-ONNX",
            )),
            glossary_max_matches: Some(11),
            ..CliArgs::default()
        };

        let result = load_config(&cli).expect("config should load");

        assert!(!result.core.glossary.enabled);
        assert_eq!(
            result.core.glossary.path,
            PathBuf::from("/cli/glossary.tsv")
        );
        assert_eq!(
            result.core.glossary.embedding_model_dir,
            PathBuf::from("/cli/models/embeddinggemma-300m-ONNX")
        );
        assert_eq!(result.core.glossary.max_matches, 11);
        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn load_config_should_default_source_to_auto() {
        let _guard = env_guard();
        let (old_cwd, _repo_root) = with_repo_root();
        let _cwd_guard = CwdGuard { old_cwd };

        let _no_config = EnvVarGuard::remove("PETIT_TRAD_NO_CONFIG");
        let _source_lang = EnvVarGuard::remove("PETIT_TRAD_SOURCE_LANG");
        let _target_lang = EnvVarGuard::remove("PETIT_TRAD_TARGET_LANG");

        let cli = CliArgs {
            no_config: true,
            ..CliArgs::default()
        };

        let result = load_config(&cli).expect("config should load");

        assert_eq!(result.source_lang, "auto");
        assert_eq!(result.target_lang, "fr");
    }

    #[test]
    fn load_config_should_keep_default_glossary_embedding_model_dir_when_not_overridden() {
        let _guard = env_guard();
        let (old_cwd, _repo_root) = with_repo_root();
        let _cwd_guard = CwdGuard { old_cwd };
        let config_path = write_temp_config(
            r#"
[glossary]
enabled = true
path = "config/glossary.tsv"
max_matches = 6
"#,
        );

        let _enabled = EnvVarGuard::remove("PETIT_TRAD_GLOSSARY_ENABLED");
        let _path = EnvVarGuard::remove("PETIT_TRAD_GLOSSARY_PATH");
        let _model_dir = EnvVarGuard::remove("PETIT_TRAD_GLOSSARY_EMBEDDING_MODEL_DIR");
        let _max_matches = EnvVarGuard::remove("PETIT_TRAD_GLOSSARY_MAX_MATCHES");

        let cli = CliArgs {
            config: Some(config_path.clone()),
            ..CliArgs::default()
        };

        let result = load_config(&cli).expect("config should load");

        assert_eq!(
            result.core.glossary.embedding_model_dir,
            PathBuf::from("models/embeddinggemma-300m-ONNX")
        );

        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn load_config_should_expand_home_in_glossary_paths_from_file() {
        let _guard = env_guard();
        let home = PathBuf::from("/tmp/petit-home");
        let _home = EnvVarGuard::set("HOME", home.display().to_string());
        let (old_cwd, _repo_root) = with_repo_root();
        let _cwd_guard = CwdGuard { old_cwd };
        let config_path = write_temp_config(
            r#"
[glossary]
enabled = true
path = "$HOME/.config/petit_trad/glossary.tsv"
embedding_model_dir = "~/models/embeddinggemma-300m-ONNX"
max_matches = 6
"#,
        );

        let cli = CliArgs {
            config: Some(config_path.clone()),
            ..CliArgs::default()
        };

        let result = load_config(&cli).expect("config should load");

        assert_eq!(
            result.core.glossary.path,
            home.join(".config/petit_trad/glossary.tsv")
        );
        assert_eq!(
            result.core.glossary.embedding_model_dir,
            home.join("models/embeddinggemma-300m-ONNX")
        );

        let _ = std::fs::remove_file(config_path);
    }
}
