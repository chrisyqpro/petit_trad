// SPDX-License-Identifier: GPL-3.0-or-later

//! Error types for petit-core

use thiserror::Error;

/// Errors that can occur in petit-core
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Glossary configuration error
    #[error("Glossary configuration error: {0}")]
    GlossaryConfig(String),

    /// Glossary file read error
    #[error("Glossary file read error: {0}")]
    GlossaryRead(String),

    /// Glossary parse error
    #[error("Glossary parse error: {0}")]
    GlossaryParse(String),

    /// Glossary embedding initialization error
    #[error("Glossary embedding initialization error: {0}")]
    GlossaryEmbeddingInit(String),

    /// Glossary embedding generation error
    #[error("Glossary embedding generation error: {0}")]
    GlossaryEmbeddingGenerate(String),

    /// Glossary index build error
    #[error("Glossary index build error: {0}")]
    GlossaryIndexBuild(String),

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
