// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration types for petit-core

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for glossary-constrained translation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GlossaryConfig {
    /// Enable glossary retrieval
    #[serde(default)]
    pub enabled: bool,

    /// Path to the glossary TSV file
    #[serde(default)]
    pub path: PathBuf,

    /// Path to the local embedding model directory
    #[serde(default)]
    pub embedding_model_dir: PathBuf,

    /// Maximum glossary candidates to inject into the prompt
    #[serde(default)]
    pub max_matches: usize,
}

/// Configuration for the translation engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

    /// Glossary configuration
    #[serde(default)]
    pub glossary: GlossaryConfig,
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
        assert_eq!(config.glossary, GlossaryConfig::default());
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
            glossary: GlossaryConfig {
                enabled: true,
                path: PathBuf::from("/tmp/glossary.tsv"),
                embedding_model_dir: PathBuf::from("/tmp/models/embeddinggemma-300m-ONNX"),
                max_matches: 4,
            },
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
        assert_eq!(config.glossary, GlossaryConfig::default());
    }

    #[test]
    fn test_parse_glossary_toml_roundtrip() {
        let toml_str = r#"
model_path = "/models/translategemma-12b-it.gguf"
gpu_layers = 16
context_size = 2048
threads = 4
log_to_file = false
log_path = "/var/log/petit/llama.log"

[glossary]
enabled = true
path = "config/glossary.tsv"
embedding_model_dir = "models/embeddinggemma-300m-ONNX"
max_matches = 6
"#;

        let config = Config::from_toml(toml_str).expect("parse should succeed");
        assert!(config.glossary.enabled);
        assert_eq!(config.glossary.path, PathBuf::from("config/glossary.tsv"));
        assert_eq!(
            config.glossary.embedding_model_dir,
            PathBuf::from("models/embeddinggemma-300m-ONNX")
        );
        assert_eq!(config.glossary.max_matches, 6);

        let roundtrip = config.to_toml().expect("serialize should succeed");
        let reparsed = Config::from_toml(&roundtrip).expect("roundtrip parse should succeed");
        assert_eq!(config, reparsed);
    }

    #[test]
    fn test_parse_disabled_glossary_toml() {
        let toml_str = r#"
model_path = "/models/translategemma-12b-it.gguf"
gpu_layers = 32
context_size = 4096
threads = 8
log_to_file = true
log_path = "/tmp/llama.log"

[glossary]
enabled = false
path = "config/glossary.tsv"
embedding_model_dir = "models/embeddinggemma-300m-ONNX"
max_matches = 0
"#;

        let config = Config::from_toml(toml_str).expect("parse should succeed");
        assert!(!config.glossary.enabled);
        assert_eq!(config.glossary.path, PathBuf::from("config/glossary.tsv"));
        assert_eq!(
            config.glossary.embedding_model_dir,
            PathBuf::from("models/embeddinggemma-300m-ONNX")
        );
        assert_eq!(config.glossary.max_matches, 0);
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
