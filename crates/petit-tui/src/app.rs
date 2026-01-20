// SPDX-License-Identifier: GPL-3.0-or-later

//! Application state and logic

/// Application state
pub struct App {
    /// Input text to translate
    pub input: String,
    /// Translated output
    pub output: String,
    /// Source language code
    pub source_lang: String,
    /// Target language code
    pub target_lang: String,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Whether translation is in progress
    pub is_loading: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            source_lang: "en".to_string(),
            target_lang: "fr".to_string(),
            should_quit: false,
            is_loading: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
}
