// SPDX-License-Identifier: GPL-3.0-or-later

//! petit-tui: Terminal interface for petit_trad
//!
//! A TUI application for translating text using TranslateGemma.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::{App, Focus, LangTarget};

mod app;
mod ui;

fn main() -> Result<()> {
    let (mut terminal, guard) = setup_terminal()?;
    guard.install_panic_hook();

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    result
}

fn setup_terminal() -> Result<(Terminal<CrosstermBackend<Stdout>>, TerminalGuard)> {
    let guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok((terminal, guard))
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| ui::render(app, frame))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => handle_key_event(app, key),
                Event::Paste(text) => app.insert_str(&text),
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    if app.is_editing_language() {
        handle_language_edit_key(app, key);
        return;
    }

    // TODO: Confirm Ctrl+Enter behavior across terminals; keep fallbacks until verified.
    if (key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL))
        || (key.code == KeyCode::Char('m') && key.modifiers.contains(KeyModifiers::CONTROL))
        || key.code == KeyCode::F(5)
    {
        app.request_translate();
        return;
    }

    if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.swap_languages();
        return;
    }

    if key.code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.clear_input();
        return;
    }

    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.begin_language_edit(LangTarget::Source);
        return;
    }

    if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.begin_language_edit(LangTarget::Target);
        return;
    }

    if key.code == KeyCode::Tab {
        app.toggle_focus();
        return;
    }

    match key.code {
        KeyCode::Enter => match app.focus {
            Focus::Input => app.insert_char('\n'),
            Focus::Output => app.request_translate(),
        },
        KeyCode::Backspace => app.backspace(),
        KeyCode::Delete => app.delete(),
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Up => match app.focus {
            Focus::Input => app.move_up(),
            Focus::Output => app.scroll_output(-1),
        },
        KeyCode::Down => match app.focus {
            Focus::Input => app.move_down(),
            Focus::Output => app.scroll_output(1),
        },
        KeyCode::Home => app.move_home(),
        KeyCode::End => app.move_end(),
        KeyCode::PageUp => match app.focus {
            Focus::Input => app.scroll_input(-3),
            Focus::Output => app.scroll_output(-3),
        },
        KeyCode::PageDown => match app.focus {
            Focus::Input => app.scroll_input(3),
            Focus::Output => app.scroll_output(3),
        },
        KeyCode::Char(ch) => {
            if is_text_input(&key) {
                app.insert_char(ch);
            }
        }
        _ => {}
    }
}

fn handle_language_edit_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.cancel_language_edit();
        return;
    }

    if key.code == KeyCode::Enter {
        app.submit_language_edit();
        return;
    }

    match key.code {
        KeyCode::Backspace => app.backspace(),
        KeyCode::Delete => app.delete(),
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Home => app.move_home(),
        KeyCode::End => app.move_end(),
        KeyCode::Char(ch) => {
            if is_text_input(&key) {
                app.insert_char(ch);
            }
        }
        _ => {}
    }
}

fn is_text_input(key: &KeyEvent) -> bool {
    !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT)
}

struct TerminalGuard {
    cleaned: Arc<AtomicBool>,
}

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        Ok(Self {
            cleaned: Arc::new(AtomicBool::new(false)),
        })
    }

    fn install_panic_hook(&self) {
        let cleaned = Arc::clone(&self.cleaned);
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            cleanup_terminal(&cleaned);
            default_hook(info);
        }));
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        cleanup_terminal(&self.cleaned);
    }
}

fn cleanup_terminal(cleaned: &AtomicBool) {
    if cleaned.swap(true, Ordering::SeqCst) {
        return;
    }

    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = stdout.flush();
}
