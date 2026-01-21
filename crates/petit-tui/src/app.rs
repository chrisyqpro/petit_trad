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
    /// Which pane is focused
    pub focus: Focus,
    /// Cursor position in the input buffer
    pub input_cursor: usize,
    /// Scroll offset for the input pane
    pub input_scroll: u16,
    /// Scroll offset for the output pane
    pub output_scroll: u16,
    /// Status message shown in the footer
    pub status_message: Option<String>,
}

/// Which pane is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Output,
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
            focus: Focus::Input,
            input_cursor: 0,
            input_scroll: 0,
            output_scroll: 0,
            status_message: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
}
