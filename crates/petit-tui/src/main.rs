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

use crate::app::App;

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
            if let Event::Key(key) = event::read()? {
                handle_key_event(app, key);
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
    }
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
