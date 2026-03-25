// SPDX-License-Identifier: GPL-3.0-or-later

//! GemmaTranslator implementation for TranslateGemma models

use crate::language::{normalize_lang, supported_languages, validate_pair};
use crate::{Config, GlossaryCandidate, GlossaryStore, ModelManager, Result, Translator};

/// Default maximum tokens for translation output
const DEFAULT_MAX_NEW_TOKENS: u32 = 256;

fn build_prompt(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    glossary_terms: &[(&str, &str)],
) -> String {
    let src = normalize_lang(source_lang);
    let tgt = normalize_lang(target_lang);
    if glossary_terms.is_empty() {
        return format!(
            "<start_of_turn>user\n[{src}->{tgt}] {text}<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    let mut prompt = format!(
        "<start_of_turn>user\n[{src}->{tgt}]\nUse the glossary terms exactly when they match the source text:\n"
    );
    for (source_term, target_term) in glossary_terms {
        prompt.push_str(&format!("- {source_term} -> {target_term}\n"));
    }
    prompt.push_str(&format!(
        "\nText:\n{text}<end_of_turn>\n<start_of_turn>model\n"
    ));
    prompt
}

fn build_prompt_with_lookup<F>(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    lookup: F,
) -> Result<String>
where
    F: FnOnce(&str, &str, &str) -> Result<Vec<GlossaryCandidate>>,
{
    validate_pair(source_lang, target_lang)?;
    let glossary_candidates = lookup(source_lang, target_lang, text)?;
    let glossary_terms = glossary_candidates
        .iter()
        .map(|candidate| {
            (
                candidate.source_term.as_str(),
                candidate.target_term.as_str(),
            )
        })
        .collect::<Vec<_>>();
    Ok(build_prompt(
        text,
        source_lang,
        target_lang,
        &glossary_terms,
    ))
}

/// TranslateGemma-based translator using llama.cpp
pub struct GemmaTranslator {
    model_manager: ModelManager,
    glossary_store: Option<GlossaryStore>,
    max_new_tokens: u32,
}

impl GemmaTranslator {
    /// Create a new GemmaTranslator with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let glossary_store = Some(GlossaryStore::from_config(&config.glossary)?);
        let model_manager = ModelManager::new(config)?;
        Ok(Self {
            model_manager,
            glossary_store,
            max_new_tokens: DEFAULT_MAX_NEW_TOKENS,
        })
    }

    /// Create a new GemmaTranslator with a pre-loaded ModelManager
    pub fn with_model_manager(model_manager: ModelManager) -> Self {
        Self {
            model_manager,
            glossary_store: None,
            max_new_tokens: DEFAULT_MAX_NEW_TOKENS,
        }
    }

    /// Set the maximum number of new tokens to generate
    pub fn with_max_new_tokens(mut self, max_new_tokens: u32) -> Self {
        self.max_new_tokens = max_new_tokens;
        self
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
        let prompt = build_prompt_with_lookup(
            text,
            source_lang,
            target_lang,
            |source_lang, target_lang, text| match &self.glossary_store {
                Some(glossary_store) => {
                    glossary_store.select_candidates(source_lang, target_lang, text)
                }
                None => Ok(Vec::new()),
            },
        )?;

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
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_build_prompt_simple() {
        let prompt = build_prompt("Hello, how are you?", "en", "fr", &[]);
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
        let prompt = build_prompt(text, &src, &tgt, &[]);
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
        let prompt = build_prompt(text, &src, &tgt, &[]);
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en-us->pt-br] Good morning<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_build_prompt_without_glossary_matches_current_format() {
        let prompt = build_prompt("Hello", "en", "fr", &[]);
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en->fr] Hello<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_build_prompt_with_glossary_terms_includes_block() {
        let prompt = build_prompt(
            "Your balance is available in the savings account.",
            "en",
            "fr",
            &[
                ("account balance", "solde du compte"),
                ("savings account", "compte d'epargne"),
            ],
        );

        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en->fr]\nUse the glossary terms exactly when they match the source text:\n- account balance -> solde du compte\n- savings account -> compte d'epargne\n\nText:\nYour balance is available in the savings account.<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_translate_uses_glossary_candidates_after_validation() {
        let lookup_calls = Rc::new(Cell::new(0));
        let lookup_calls_ref = Rc::clone(&lookup_calls);
        let prompt = build_prompt_with_lookup(
            "Your balance is available in the savings account.",
            "en",
            "fr",
            move |source_lang, target_lang, text| {
                assert_eq!(source_lang, "en");
                assert_eq!(target_lang, "fr");
                assert_eq!(text, "Your balance is available in the savings account.");
                lookup_calls_ref.set(lookup_calls_ref.get() + 1);
                Ok(vec![
                    GlossaryCandidate {
                        source_term: "account balance".into(),
                        target_term: "solde du compte".into(),
                    },
                    GlossaryCandidate {
                        source_term: "savings account".into(),
                        target_term: "compte d'epargne".into(),
                    },
                ])
            },
        )
        .expect("translation should build a prompt");

        assert_eq!(lookup_calls.get(), 1);
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en->fr]\nUse the glossary terms exactly when they match the source text:\n- account balance -> solde du compte\n- savings account -> compte d'epargne\n\nText:\nYour balance is available in the savings account.<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_translate_falls_back_to_plain_prompt_when_lookup_returns_none() {
        let lookup_calls = Rc::new(Cell::new(0));
        let lookup_calls_ref = Rc::clone(&lookup_calls);
        let prompt = build_prompt_with_lookup(
            "Hello",
            "en",
            "fr",
            move |source_lang, target_lang, text| {
                assert_eq!(source_lang, "en");
                assert_eq!(target_lang, "fr");
                assert_eq!(text, "Hello");
                lookup_calls_ref.set(lookup_calls_ref.get() + 1);
                Ok(Vec::new())
            },
        )
        .expect("translation should build a prompt");

        assert_eq!(lookup_calls.get(), 1);
        assert_eq!(
            prompt,
            "<start_of_turn>user\n[en->fr] Hello<end_of_turn>\n<start_of_turn>model\n"
        );
    }

    #[test]
    fn test_translate_rejects_invalid_pair_before_lookup() {
        let lookup_calls = Rc::new(Cell::new(0));
        let lookup_calls_ref = Rc::clone(&lookup_calls);
        let err = build_prompt_with_lookup(
            "Hello",
            "xx",
            "fr",
            move |_source_lang, _target_lang, _text| {
                lookup_calls_ref.set(lookup_calls_ref.get() + 1);
                Ok(vec![GlossaryCandidate {
                    source_term: "hello".into(),
                    target_term: "bonjour".into(),
                }])
            },
        )
        .expect_err("invalid language pair should fail first");

        assert_eq!(lookup_calls.get(), 0);
        assert!(err.to_string().contains("Unsupported language"));
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
