// SPDX-License-Identifier: GPL-3.0-or-later

//! Language utilities for TranslateGemma
//!
//! TranslateGemma supports 55 languages using ISO 639-1 codes.

use crate::{Error, Result};

/// Supported ISO 639-1 language codes for TranslateGemma
///
/// Based on the TranslateGemma model card (google/translategemma-12b-it).
const SUPPORTED_LANGUAGES: &[&str] = &[
    "af", // Afrikaans
    "ar", // Arabic
    "bg", // Bulgarian
    "bn", // Bengali
    "ca", // Catalan
    "cs", // Czech
    "da", // Danish
    "de", // German
    "el", // Greek
    "en", // English
    "es", // Spanish
    "et", // Estonian
    "fa", // Persian
    "fi", // Finnish
    "fr", // French
    "gl", // Galician
    "gu", // Gujarati
    "he", // Hebrew
    "hi", // Hindi
    "hr", // Croatian
    "hu", // Hungarian
    "id", // Indonesian
    "it", // Italian
    "ja", // Japanese
    "ka", // Georgian
    "kk", // Kazakh
    "ko", // Korean
    "lt", // Lithuanian
    "lv", // Latvian
    "mk", // Macedonian
    "ml", // Malayalam
    "mr", // Marathi
    "ms", // Malay
    "ne", // Nepali
    "nl", // Dutch
    "no", // Norwegian
    "pl", // Polish
    "pt", // Portuguese
    "ro", // Romanian
    "ru", // Russian
    "sk", // Slovak
    "sl", // Slovenian
    "sq", // Albanian
    "sr", // Serbian
    "sv", // Swedish
    "sw", // Swahili
    "ta", // Tamil
    "te", // Telugu
    "th", // Thai
    "tl", // Tagalog
    "tr", // Turkish
    "uk", // Ukrainian
    "ur", // Urdu
    "vi", // Vietnamese
    "zh", // Chinese
];

/// Normalize a language code to lowercase, preserving region if present.
///
/// Examples:
/// - `"EN"` -> `"en"`
/// - `"en-US"` -> `"en-us"`
/// - `"pt-BR"` -> `"pt-br"`
pub fn normalize_lang(code: &str) -> String {
    code.to_lowercase()
}

/// Extract the base language code from a potentially regional code.
///
/// Examples:
/// - `"en"` -> `"en"`
/// - `"en-us"` -> `"en"`
/// - `"pt-br"` -> `"pt"`
fn base_lang(code: &str) -> &str {
    code.split('-').next().unwrap_or(code)
}

/// Check if a language code is supported by TranslateGemma.
///
/// Accepts both simple codes (`en`) and regional codes (`en-US`).
/// The check is case-insensitive.
pub fn is_supported(code: &str) -> bool {
    let normalized = normalize_lang(code);
    let base = base_lang(&normalized);
    SUPPORTED_LANGUAGES.contains(&base)
}

/// Validate a language pair for translation.
///
/// Returns an error if either language is unsupported.
pub fn validate_pair(source: &str, target: &str) -> Result<()> {
    if !is_supported(source) {
        return Err(Error::UnsupportedLanguage(source.to_string()));
    }
    if !is_supported(target) {
        return Err(Error::UnsupportedLanguage(target.to_string()));
    }
    Ok(())
}

/// Get the list of supported language codes.
pub fn supported_languages() -> &'static [&'static str] {
    SUPPORTED_LANGUAGES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_lang_simple() {
        assert_eq!(normalize_lang("EN"), "en");
        assert_eq!(normalize_lang("Fr"), "fr");
        assert_eq!(normalize_lang("de"), "de");
    }

    #[test]
    fn test_normalize_lang_regional() {
        assert_eq!(normalize_lang("en-US"), "en-us");
        assert_eq!(normalize_lang("pt-BR"), "pt-br");
        assert_eq!(normalize_lang("zh-TW"), "zh-tw");
    }

    #[test]
    fn test_base_lang() {
        assert_eq!(base_lang("en"), "en");
        assert_eq!(base_lang("en-us"), "en");
        assert_eq!(base_lang("pt-br"), "pt");
    }

    #[test]
    fn test_is_supported_simple() {
        assert!(is_supported("en"));
        assert!(is_supported("fr"));
        assert!(is_supported("zh"));
        assert!(is_supported("ja"));
    }

    #[test]
    fn test_is_supported_case_insensitive() {
        assert!(is_supported("EN"));
        assert!(is_supported("Fr"));
        assert!(is_supported("ZH"));
    }

    #[test]
    fn test_is_supported_regional() {
        assert!(is_supported("en-US"));
        assert!(is_supported("en-GB"));
        assert!(is_supported("pt-BR"));
        assert!(is_supported("zh-TW"));
    }

    #[test]
    fn test_is_supported_unsupported() {
        assert!(!is_supported("xx"));
        assert!(!is_supported("xyz"));
        assert!(!is_supported(""));
    }

    #[test]
    fn test_validate_pair_valid() {
        assert!(validate_pair("en", "fr").is_ok());
        assert!(validate_pair("EN", "FR").is_ok());
        assert!(validate_pair("en-US", "de-DE").is_ok());
    }

    #[test]
    fn test_validate_pair_invalid_source() {
        let result = validate_pair("xx", "en");
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::UnsupportedLanguage(code)) if code == "xx"));
    }

    #[test]
    fn test_validate_pair_invalid_target() {
        let result = validate_pair("en", "yy");
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::UnsupportedLanguage(code)) if code == "yy"));
    }

    #[test]
    fn test_supported_languages_count() {
        assert_eq!(supported_languages().len(), 55);
    }

    #[test]
    fn test_supported_languages_contains_common() {
        let langs = supported_languages();
        assert!(langs.contains(&"en"));
        assert!(langs.contains(&"fr"));
        assert!(langs.contains(&"de"));
        assert!(langs.contains(&"es"));
        assert!(langs.contains(&"zh"));
        assert!(langs.contains(&"ja"));
    }
}
