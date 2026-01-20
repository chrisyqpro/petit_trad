// SPDX-License-Identifier: GPL-3.0-or-later

//! Error types for petit-core

use thiserror::Error;

/// Errors that can occur in petit-core
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Model loading error
    #[error("Failed to load model: {0}")]
    ModelLoad(String),

    /// Inference error
    #[error("Inference error: {0}")]
    Inference(String),

    /// Unsupported language
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}
