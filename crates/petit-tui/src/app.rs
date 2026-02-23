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
    /// Language edit state (if active)
    lang_edit: Option<LangEdit>,
    /// Show compact language display in header
    pub compact_lang_display: bool,
}

/// Which pane is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LangTarget {
    Source,
    Target,
}

pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

struct LangEdit {
    target: LangTarget,
    buffer: String,
    cursor: usize,
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
            lang_edit: None,
            compact_lang_display: false,
        }
    }
}

impl App {
    pub fn with_languages(
        source_lang: String,
        target_lang: String,
        compact_lang_display: bool,
    ) -> Self {
        Self {
            source_lang,
            target_lang,
            compact_lang_display,
            ..Self::default()
        }
    }

    pub fn is_editing_language(&self) -> bool {
        self.lang_edit.is_some()
    }

    pub fn language_prompt(&self) -> Option<String> {
        let edit = self.lang_edit.as_ref()?;
        let label = match edit.target {
            LangTarget::Source => "source",
            LangTarget::Target => "target",
        };
        Some(format!("Set {} language: {}", label, edit.buffer))
    }

    pub fn begin_language_edit(&mut self, target: LangTarget) {
        let buffer = match target {
            LangTarget::Source => self.source_lang.clone(),
            LangTarget::Target => self.target_lang.clone(),
        };
        self.lang_edit = Some(LangEdit {
            target,
            cursor: buffer.chars().count(),
            buffer,
        });
        self.status_message = None;
    }

    pub fn cancel_language_edit(&mut self) {
        self.lang_edit = None;
        self.status_message = Some("Language edit canceled".to_string());
    }

    pub fn submit_language_edit(&mut self) {
        let edit = match self.lang_edit.take() {
            Some(edit) => edit,
            None => return,
        };

        let new_value = petit_core::language::normalize_lang(&edit.buffer);
        let (new_source, new_target) = match edit.target {
            LangTarget::Source => (new_value.clone(), self.target_lang.clone()),
            LangTarget::Target => (self.source_lang.clone(), new_value.clone()),
        };

        match petit_core::language::validate_pair(&new_source, &new_target) {
            Ok(()) => {
                if edit.target == LangTarget::Source {
                    self.source_lang = new_value;
                } else {
                    self.target_lang = new_value;
                }
                self.status_message = Some("Language updated".to_string());
            }
            Err(err) => {
                self.status_message = Some(err.to_string());
            }
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::Output,
            Focus::Output => Focus::Input,
        };
    }

    pub fn swap_languages(&mut self) {
        std::mem::swap(&mut self.source_lang, &mut self.target_lang);
        self.status_message = Some("Languages swapped".to_string());
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.input_cursor = 0;
        self.input_scroll = 0;
        self.status_message = Some("Input cleared".to_string());
    }

    pub fn insert_char(&mut self, ch: char) {
        self.edit_active_buffer(|buffer, cursor| {
            let insert_len = 1;
            insert_into(buffer, *cursor, ch);
            *cursor += insert_len;
        });
        self.status_message = None;
    }

    pub fn insert_str(&mut self, text: &str) {
        let insert_len = text.chars().count();
        self.edit_active_buffer(|buffer, cursor| {
            insert_str_into(buffer, *cursor, text);
            *cursor += insert_len;
        });
        self.status_message = None;
    }

    pub fn backspace(&mut self) {
        self.edit_active_buffer(|buffer, cursor| {
            if *cursor == 0 {
                return;
            }
            remove_at(buffer, cursor.saturating_sub(1));
            *cursor = cursor.saturating_sub(1);
        });
        self.status_message = None;
    }

    pub fn delete(&mut self) {
        self.edit_active_buffer(|buffer, cursor| {
            if *cursor >= buffer.chars().count() {
                return;
            }
            remove_at(buffer, *cursor);
        });
        self.status_message = None;
    }

    pub fn move_left(&mut self) {
        self.edit_active_buffer(|_, cursor| {
            if *cursor > 0 {
                *cursor -= 1;
            }
        });
    }

    pub fn move_right(&mut self) {
        self.edit_active_buffer(|buffer, cursor| {
            let max = buffer.chars().count();
            if *cursor < max {
                *cursor += 1;
            }
        });
    }

    pub fn move_home(&mut self) {
        if self.is_editing_language() {
            self.edit_active_buffer(|_, cursor| {
                *cursor = 0;
            });
            return;
        }
        if self.focus == Focus::Input {
            let (line, _) = line_col(&self.input, self.input_cursor);
            if let Some(start) = line_start(&self.input, line) {
                self.input_cursor = start;
            }
        }
    }

    pub fn move_end(&mut self) {
        if self.is_editing_language() {
            self.edit_active_buffer(|buffer, cursor| {
                *cursor = buffer.chars().count();
            });
            return;
        }
        if self.focus == Focus::Input {
            let (line, _) = line_col(&self.input, self.input_cursor);
            if let Some(end) = line_end(&self.input, line) {
                self.input_cursor = end;
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.is_editing_language() || self.focus != Focus::Input {
            return;
        }
        if let Some(new_cursor) = move_vertical(&self.input, self.input_cursor, -1) {
            self.input_cursor = new_cursor;
        }
    }

    pub fn move_down(&mut self) {
        if self.is_editing_language() || self.focus != Focus::Input {
            return;
        }
        if let Some(new_cursor) = move_vertical(&self.input, self.input_cursor, 1) {
            self.input_cursor = new_cursor;
        }
    }

    pub fn scroll_input(&mut self, delta: i16) {
        let max = max_scroll(&self.input);
        self.input_scroll = scroll_value(self.input_scroll, delta, max);
    }

    pub fn scroll_output(&mut self, delta: i16) {
        let max = max_scroll(&self.output);
        self.output_scroll = scroll_value(self.output_scroll, delta, max);
    }

    pub fn begin_translation(&mut self) -> Option<TranslationRequest> {
        if self.is_loading {
            self.status_message = Some("Translation already in progress".to_string());
            return None;
        }
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            self.status_message = Some("Input is empty".to_string());
            return None;
        }

        self.is_loading = true;
        self.status_message = Some("Translating...".to_string());

        Some(TranslationRequest {
            text: self.input.clone(),
            source_lang: self.source_lang.clone(),
            target_lang: self.target_lang.clone(),
        })
    }

    pub fn apply_translation_result(&mut self, result: Result<String, String>) {
        self.is_loading = false;
        match result {
            Ok(text) => {
                self.output = text;
                self.output_scroll = 0;
                self.status_message = Some("Translation complete".to_string());
            }
            Err(err) => {
                self.status_message = Some(err);
            }
        }
    }

    fn edit_active_buffer<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut String, &mut usize),
    {
        if let Some(edit) = self.lang_edit.as_mut() {
            update(&mut edit.buffer, &mut edit.cursor);
        } else if self.focus == Focus::Input {
            update(&mut self.input, &mut self.input_cursor);
        }
    }
}

fn insert_into(buffer: &mut String, cursor: usize, ch: char) {
    let mut temp = [0u8; 4];
    let text = ch.encode_utf8(&mut temp);
    insert_str_into(buffer, cursor, text);
}

fn insert_str_into(buffer: &mut String, cursor: usize, text: &str) {
    let mut chars: Vec<char> = buffer.chars().collect();
    let cursor = cursor.min(chars.len());
    chars.splice(cursor..cursor, text.chars());
    *buffer = chars.into_iter().collect();
}

fn remove_at(buffer: &mut String, index: usize) {
    let mut chars: Vec<char> = buffer.chars().collect();
    if index >= chars.len() {
        return;
    }
    chars.remove(index);
    *buffer = chars.into_iter().collect();
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, ch) in text.chars().enumerate() {
        if ch == '\n' {
            starts.push(index + 1);
        }
    }
    starts
}

fn line_col(text: &str, cursor: usize) -> (usize, usize) {
    let starts = line_starts(text);
    let mut line = 0;
    for idx in 0..starts.len() {
        if idx + 1 == starts.len() || cursor < starts[idx + 1] {
            line = idx;
            break;
        }
    }
    let col = cursor.saturating_sub(starts[line]);
    (line, col)
}

fn line_start(text: &str, line: usize) -> Option<usize> {
    let starts = line_starts(text);
    starts.get(line).copied()
}

fn line_end(text: &str, line: usize) -> Option<usize> {
    let starts = line_starts(text);
    let total = text.chars().count();
    if line >= starts.len() {
        return None;
    }
    if line + 1 < starts.len() {
        Some(starts[line + 1].saturating_sub(1))
    } else {
        Some(total)
    }
}

fn move_vertical(text: &str, cursor: usize, delta: isize) -> Option<usize> {
    let starts = line_starts(text);
    let (line, col) = line_col(text, cursor);
    let target_line = if delta.is_negative() {
        let steps = (-delta) as usize;
        line.checked_sub(steps)?
    } else {
        line + delta as usize
    };
    if target_line >= starts.len() {
        return None;
    }
    let target_start = starts[target_line];
    let target_end = line_end(text, target_line)?;
    let target_len = target_end.saturating_sub(target_start);
    let new_col = col.min(target_len);
    Some(target_start + new_col)
}

fn max_scroll(text: &str) -> u16 {
    let lines = line_starts(text).len();
    lines.saturating_sub(1) as u16
}

fn scroll_value(current: u16, delta: i16, max: u16) -> u16 {
    let next = if delta.is_negative() {
        current.saturating_sub((-delta) as u16)
    } else {
        current.saturating_add(delta as u16)
    };
    next.min(max)
}
