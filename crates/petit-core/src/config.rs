// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration types for petit-core

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the translation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
