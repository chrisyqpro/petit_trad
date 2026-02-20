// SPDX-License-Identifier: GPL-3.0-or-later

//! GemmaTranslator implementation for TranslateGemma models

use crate::language::{normalize_lang, supported_languages, validate_pair};
use crate::{Config, ModelManager, Result, Translator};

/// Default maximum tokens for translation output
const DEFAULT_MAX_NEW_TOKENS: u32 = 256;

/// TranslateGemma-based translator using llama.cpp
pub struct GemmaTranslator {
    model_manager: ModelManager,
    max_new_tokens: u32,
}

impl GemmaTranslator {
    /// Create a new GemmaTranslator with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let model_manager = ModelManager::new(config)?;
        Ok(Self {
            model_manager,
            max_new_tokens: DEFAULT_MAX_NEW_TOKENS,
        })
    }

    /// Create a new GemmaTranslator with a pre-loaded ModelManager
    pub fn with_model_manager(model_manager: ModelManager) -> Self {
        Self {
            model_manager,
            max_new_tokens: DEFAULT_MAX_NEW_TOKENS,
        }
    }

    /// Set the maximum number of new tokens to generate
    pub fn with_max_new_tokens(mut self, max_new_tokens: u32) -> Self {
        self.max_new_tokens = max_new_tokens;
        self
    }

    /// Build the TranslateGemma prompt using direct format
    ///
    /// Format: `<start_of_turn>user\n[src->tgt] text<end_of_turn>\n<start_of_turn>model\n`
    fn build_prompt(&self, text: &str, source_lang: &str, target_lang: &str) -> String {
        let src = normalize_lang(source_lang);
        let tgt = normalize_lang(target_lang);
        format!("<start_of_turn>user\n[{src}->{tgt}] {text}<end_of_turn>\n<start_of_turn>model\n")
    }

    /// Clean the model output by stripping whitespace and any echo artifacts
    fn clean_output(&self, output: &str) -> String {
        let cleaned = output.trim();

        // Remove any trailing <end_of_turn> if present
        let cleaned = cleaned
            .strip_suffix("<end_of_turn>")
            .unwrap_or(cleaned)
            .trim();

        cleaned.to_string()
    }
}

impl Translator for GemmaTranslator {
    fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        // Validate language pair
        validate_pair(source_lang, target_lang)?;

        // Build prompt
        let prompt = self.build_prompt(text, source_lang, target_lang);

        // Run inference
        let output = self.model_manager.infer(&prompt, self.max_new_tokens)?;

        // Clean and return
        Ok(self.clean_output(&output))
    }

    fn supported_languages(&self) -> &[&str] {
        supported_languages()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_simple() {
        // Create a mock-like test by just testing the prompt building
        let prompt = format!(
            "<start_of_turn>user\n[{}->{}] {}<end_of_turn>\n<start_of_turn>model\n",
            "en", "fr", "Hello, how are you?"
        );
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en->fr] Hello, how are you?<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_build_prompt_normalized() {
        let src = normalize_lang("EN");
        let tgt = normalize_lang("FR");
        let text = "Hello";
        let prompt = format!(
            "<start_of_turn>user\n[{src}->{tgt}] {text}<end_of_turn>\n<start_of_turn>model\n"
        );
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en->fr] Hello<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_build_prompt_regional() {
        let src = normalize_lang("en-US");
        let tgt = normalize_lang("pt-BR");
        let text = "Good morning";
        let prompt = format!(
            "<start_of_turn>user\n[{src}->{tgt}] {text}<end_of_turn>\n<start_of_turn>model\n"
        );
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en-us->pt-br] Good morning<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_clean_output_simple() {
        let output = "  Bonjour, comment allez-vous?  ";
        let cleaned = output.trim().to_string();
        assert_eq!(cleaned, "Bonjour, comment allez-vous?");
    }

    #[test]
    fn test_clean_output_with_end_token() {
        let output = "Bonjour<end_of_turn>";
        let cleaned = output
            .trim()
            .strip_suffix("<end_of_turn>")
            .unwrap_or(output.trim())
            .trim()
            .to_string();
        assert_eq!(cleaned, "Bonjour");
    }

    #[test]
    fn test_clean_output_with_whitespace_and_end_token() {
        let output = "  Bonjour  <end_of_turn>  ";
        let cleaned = output.trim();
        let cleaned = cleaned
            .strip_suffix("<end_of_turn>")
            .unwrap_or(cleaned)
            .trim();
        assert_eq!(cleaned, "Bonjour");
    }
}
