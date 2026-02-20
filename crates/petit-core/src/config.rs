// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration types for petit-core

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the translation engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Path to the GGUF model file
    pub model_path: PathBuf,

    /// Number of GPU layers to offload (0 = CPU only)
    pub gpu_layers: u32,

    /// Context size for the model
    pub context_size: u32,

    /// Number of threads for CPU inference
    pub threads: u32,

    /// Write llama.cpp logs to a file instead of stderr
    pub log_to_file: bool,

    /// Path to the log file when log_to_file is enabled
    pub log_path: PathBuf,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Config(format!("Failed to read config: {}", e)))?;
        toml::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))
    }

    /// Serialize configuration to TOML string
    pub fn to_toml(&self) -> crate::Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {}", e)))
    }

    /// Parse configuration from TOML string
    pub fn from_toml(s: &str) -> crate::Result<Self> {
        toml::from_str(s)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_toml_with_required_values() {
        let toml_str = r#"
model_path = "models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf"
gpu_layers = 999
context_size = 2048
threads = 4
log_to_file = false
log_path = "logs/llama.log"
"#;

        let config = Config::from_toml(toml_str).expect("parse should succeed");
        assert_eq!(
            config.model_path,
            PathBuf::from("models/translategemma-12b-it-GGUF/translategemma-12b-it.Q8_0.gguf")
        );
        assert_eq!(config.gpu_layers, 999);
        assert_eq!(config.context_size, 2048);
        assert_eq!(config.threads, 4);
        assert!(!config.log_to_file);
        assert_eq!(config.log_path, PathBuf::from("logs/llama.log"));
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config {
            model_path: PathBuf::from("/path/to/model.gguf"),
            gpu_layers: 32,
            context_size: 4096,
            threads: 8,
            log_to_file: true,
            log_path: PathBuf::from("/tmp/llama.log"),
        };

        let toml_str = config.to_toml().expect("serialize should succeed");
        let parsed = Config::from_toml(&toml_str).expect("parse should succeed");

        assert_eq!(config, parsed);
    }

    #[test]
    fn test_parse_minimal_toml() {
        let toml_str = r#"
model_path = "my-model.gguf"
"#;
        let result = Config::from_toml(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
model_path = "/models/translategemma-27b-it.gguf"
gpu_layers = 64
context_size = 8192
threads = 16
log_to_file = true
log_path = "/var/log/petit/llama.log"
"#;
        let config = Config::from_toml(toml_str).expect("parse should succeed");

        assert_eq!(
            config.model_path,
            PathBuf::from("/models/translategemma-27b-it.gguf")
        );
        assert_eq!(config.gpu_layers, 64);
        assert_eq!(config.context_size, 8192);
        assert_eq!(config.threads, 16);
        assert!(config.log_to_file);
        assert_eq!(config.log_path, PathBuf::from("/var/log/petit/llama.log"));
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml_str = "this is not valid toml {{{{";
        let result = Config::from_toml(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_missing() {
        let result = Config::from_file(std::path::Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
    }
}
