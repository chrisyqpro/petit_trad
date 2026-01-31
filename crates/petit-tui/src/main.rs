// SPDX-License-Identifier: GPL-3.0-or-later

//! petit-tui: Terminal interface for petit_trad
//!
//! A TUI application for translating text using TranslateGemma.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use petit_core::{Config, GemmaTranslator, Translator};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Read, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::app::{App, Focus, LangTarget, TranslationRequest};
use crate::cli::CliArgs;
use crate::config::{load_config, AppConfig};

mod app;
mod cli;
mod config;
mod ui;

fn main() -> Result<()> {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = CliArgs::parse()?;
    if cli.show_help {
        println!("{}", CliArgs::usage());
        return Ok(());
    }
    if cli.show_version {
        println!("petit {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let app_config = load_config(&cli)?;
    let _compact_lang_display = app_config.compact_lang_display;
    if app_config.stdin_mode {
        return run_stdin(app_config);
    }

    let (mut terminal, guard) = setup_terminal()?;
    guard.install_panic_hook();

    let mut app = App::with_languages(app_config.source_lang, app_config.target_lang);
    let (tx, rx, worker) = start_translation_worker(app_config.core);
    let result = run_app(&mut terminal, &mut app, &tx, &rx);
    drop(tx);
    let _ = worker.join();

    result
}

fn run_stdin(config: AppConfig) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.trim().is_empty() {
        return Err(anyhow::anyhow!("stdin is empty"));
    }

    let translator = GemmaTranslator::new(config.core)?;
    let output = translator.translate(&input, &config.source_lang, &config.target_lang)?;
    println!("{output}");
    Ok(())
}

fn setup_terminal() -> Result<(Terminal<CrosstermBackend<Stdout>>, TerminalGuard)> {
    let guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok((terminal, guard))
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    tx: &Sender<TranslationRequest>,
    rx: &Receiver<TranslationResponse>,
) -> Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| ui::render(app, frame))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => handle_key_event(app, key, tx),
                Event::Paste(text) => app.insert_str(&text),
                _ => {}
            }
        }

        while let Ok(response) = rx.try_recv() {
            app.apply_translation_result(response.into_result());
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

fn handle_key_event(app: &mut App, key: KeyEvent, tx: &Sender<TranslationRequest>) {
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
        request_translation(app, tx);
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
            Focus::Output => request_translation(app, tx),
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

fn request_translation(app: &mut App, tx: &Sender<TranslationRequest>) {
    let request = match app.begin_translation() {
        Some(request) => request,
        None => return,
    };

    if tx.send(request).is_err() {
        app.apply_translation_result(Err("Translation worker unavailable".to_string()));
    }
}

fn start_translation_worker(config: Config) -> (
    Sender<TranslationRequest>,
    Receiver<TranslationResponse>,
    thread::JoinHandle<()>,
) {
    let (request_tx, request_rx) = mpsc::channel::<TranslationRequest>();
    let (response_tx, response_rx) = mpsc::channel::<TranslationResponse>();

    let worker = thread::spawn(move || {
        let mut translator: Option<GemmaTranslator> = None;
        for request in request_rx {
            if translator.is_none() {
                match GemmaTranslator::new(config.clone()) {
                    Ok(instance) => translator = Some(instance),
                    Err(err) => {
                        let _ = response_tx.send(TranslationResponse::Err(err.to_string()));
                        continue;
                    }
                }
            }

            let response = match translator.as_ref() {
                Some(instance) => instance
                    .translate(&request.text, &request.source_lang, &request.target_lang)
                    .map(TranslationResponse::Ok)
                    .unwrap_or_else(|err| TranslationResponse::Err(err.to_string())),
                None => TranslationResponse::Err("Translator unavailable".to_string()),
            };

            let _ = response_tx.send(response);
        }
    });

    (request_tx, response_rx, worker)
}

enum TranslationResponse {
    Ok(String),
    Err(String),
}

impl TranslationResponse {
    fn into_result(self) -> Result<String, String> {
        match self {
            TranslationResponse::Ok(text) => Ok(text),
            TranslationResponse::Err(err) => Err(err),
        }
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
