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
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: u32,

    /// Context size for the model
    #[serde(default = "default_context_size")]
    pub context_size: u32,

    /// Number of threads for CPU inference
    #[serde(default = "default_threads")]
    pub threads: u32,
}

fn default_gpu_layers() -> u32 {
    999 // Offload all layers by default
}

fn default_context_size() -> u32 {
    2048
}

fn default_threads() -> u32 {
    4
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/translate-gemma-12b-q4_k_m.gguf"),
            gpu_layers: default_gpu_layers(),
            context_size: default_context_size(),
            threads: default_threads(),
        }
    }
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
    fn test_default_values() {
        let config = Config::default();
        assert_eq!(config.model_path, PathBuf::from("models/translate-gemma-12b-q4_k_m.gguf"));
        assert_eq!(config.gpu_layers, 999);
        assert_eq!(config.context_size, 2048);
        assert_eq!(config.threads, 4);
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config {
            model_path: PathBuf::from("/path/to/model.gguf"),
            gpu_layers: 32,
            context_size: 4096,
            threads: 8,
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
        let config = Config::from_toml(toml_str).expect("parse should succeed");

        assert_eq!(config.model_path, PathBuf::from("my-model.gguf"));
        // Defaults should apply
        assert_eq!(config.gpu_layers, 999);
        assert_eq!(config.context_size, 2048);
        assert_eq!(config.threads, 4);
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
model_path = "/models/translate-gemma-27b.gguf"
gpu_layers = 64
context_size = 8192
threads = 16
"#;
        let config = Config::from_toml(toml_str).expect("parse should succeed");

        assert_eq!(config.model_path, PathBuf::from("/models/translate-gemma-27b.gguf"));
        assert_eq!(config.gpu_layers, 64);
        assert_eq!(config.context_size, 8192);
        assert_eq!(config.threads, 16);
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

