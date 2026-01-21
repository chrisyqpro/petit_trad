// SPDX-License-Identifier: GPL-3.0-or-later

//! petit-core: Translation engine for petit_trad
//!
//! This crate provides the core translation functionality using TranslateGemma
//! via llama.cpp bindings.

pub mod config;
pub mod error;
pub mod gemma;
pub mod language;
pub mod model_manager;

pub use config::Config;
pub use error::Error;
pub use gemma::GemmaTranslator;
pub use model_manager::ModelManager;

/// Result type for petit-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Translator trait defining the translation interface
///
/// This trait allows different backends (llama-cpp, candle, mock) to be used
/// interchangeably.
pub trait Translator {
    /// Translate text from source language to target language
    fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String>;

    /// Get list of supported language codes
    fn supported_languages(&self) -> &[&str];
}
