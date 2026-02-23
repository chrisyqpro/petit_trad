// SPDX-License-Identifier: GPL-3.0-or-later

//! petit-tui: Terminal interface for petit_trad
//!
//! A TUI application for translating text using TranslateGemma.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use petit_core::{Config, GemmaTranslator, Translator};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Read, Stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::app::{App, Focus, LangTarget, TranslationRequest};
use crate::cli::CliArgs;
use crate::config::{AppConfig, load_config};

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
    if cli.benchmark {
        return run_benchmark(app_config, &cli);
    }
    if app_config.stdin_mode {
        return run_stdin(app_config);
    }

    let (mut terminal, guard) = setup_terminal()?;
    guard.install_panic_hook();

    let mut app = App::with_languages(
        app_config.source_lang,
        app_config.target_lang,
        app_config.compact_lang_display,
    );
    let (tx, rx, worker) = start_translation_worker(app_config.core);
    let result = run_app(&mut terminal, &mut app, &tx, &rx);
    drop(tx);
    let _ = worker.join();

    result
}

fn run_benchmark(config: AppConfig, cli: &CliArgs) -> Result<()> {
    let warmup_runs = cli.benchmark_warmup_runs.unwrap_or(0);
    let runs = cli.benchmark_runs.unwrap_or(1);
    let max_new_tokens = cli.benchmark_max_new_tokens.unwrap_or(256);
    if runs == 0 {
        return Err(anyhow::anyhow!("--runs must be at least 1"));
    }

    let text = if let Some(text) = &cli.benchmark_text {
        text.clone()
    } else if cli.stdin {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        if input.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "benchmark input is empty (provide --text or pipe stdin)"
            ));
        }
        input
    } else {
        "Hello, how are you?".to_string()
    };

    println!("Model: {}", config.core.model_path.display());
    println!(
        "Config: gpu_layers={} context_size={} threads={}",
        config.core.gpu_layers, config.core.context_size, config.core.threads
    );
    println!(
        "Languages: {} -> {}",
        config.source_lang, config.target_lang
    );
    println!("Input chars: {}", text.chars().count());
    println!("Warmup runs: {warmup_runs}");
    println!("Measured runs: {runs}");
    println!("Max new tokens: {max_new_tokens}");

    let startup_start = Instant::now();
    let translator = GemmaTranslator::new(config.core)?.with_max_new_tokens(max_new_tokens);
    let startup_elapsed = startup_start.elapsed();
    println!("Startup: {:.2?}", startup_elapsed);

    for run_idx in 0..warmup_runs {
        let start = Instant::now();
        let _ = translator.translate(&text, &config.source_lang, &config.target_lang)?;
        let elapsed = start.elapsed();
        println!("Warmup {}: {:.2?}", run_idx + 1, elapsed);
    }

    let mut run_times = Vec::with_capacity(runs as usize);
    let mut last_output = String::new();
    for run_idx in 0..runs {
        let start = Instant::now();
        let output = translator.translate(&text, &config.source_lang, &config.target_lang)?;
        let elapsed = start.elapsed();
        println!("Run {}: {:.2?}", run_idx + 1, elapsed);
        run_times.push(elapsed);
        last_output = output;
    }

    let min = run_times.iter().copied().min().unwrap_or(Duration::ZERO);
    let max = run_times.iter().copied().max().unwrap_or(Duration::ZERO);
    let avg = duration_avg(&run_times);
    let total_measured = run_times.iter().map(Duration::as_secs_f64).sum::<f64>();

    println!("Source: {text}");
    println!("Target: {last_output}");
    println!("Average: {:.2?}", avg);
    println!("Min: {:.2?}", min);
    println!("Max: {:.2?}", max);
    println!(
        "Measured total: {:.2?}",
        Duration::from_secs_f64(total_measured)
    );

    Ok(())
}

fn duration_avg(values: &[Duration]) -> Duration {
    if values.is_empty() {
        return Duration::ZERO;
    }
    let total_secs = values.iter().map(Duration::as_secs_f64).sum::<f64>();
    Duration::from_secs_f64(total_secs / values.len() as f64)
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
    rx: &Receiver<WorkerEvent>,
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

        while let Ok(event) = rx.try_recv() {
            match event {
                WorkerEvent::TranslatorInitializing => app.begin_worker_initialization(),
                WorkerEvent::TranslatorReady => app.apply_worker_ready(),
                WorkerEvent::TranslatorInitFailed(err) => app.apply_worker_init_error(err),
                WorkerEvent::Translation(response) => {
                    app.apply_translation_result(response.into_result())
                }
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

fn handle_key_event(app: &mut App, key: KeyEvent, tx: &Sender<TranslationRequest>) {
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    if app.is_editing_language() {
        handle_language_edit_key(app, key);
        return;
    }

    if is_translate_shortcut(&key) {
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

fn is_translate_shortcut(key: &KeyEvent) -> bool {
    key.code == KeyCode::Enter
}

fn request_translation(app: &mut App, tx: &Sender<TranslationRequest>) {
    let request = match app.begin_translation() {
        Some(request) => request,
        None => return,
    };

    if tx.send(request).is_err() {
        app.apply_worker_unavailable();
    }
}

fn start_translation_worker(
    config: Config,
) -> (
    Sender<TranslationRequest>,
    Receiver<WorkerEvent>,
    thread::JoinHandle<()>,
) {
    let (request_tx, request_rx) = mpsc::channel::<TranslationRequest>();
    let (response_tx, response_rx) = mpsc::channel::<WorkerEvent>();

    let worker = thread::spawn(move || {
        let _ = response_tx.send(WorkerEvent::TranslatorInitializing);

        let mut translator: Option<GemmaTranslator> = None;
        let mut init_error: Option<String> = None;
        match GemmaTranslator::new(config) {
            Ok(instance) => {
                translator = Some(instance);
                let _ = response_tx.send(WorkerEvent::TranslatorReady);
            }
            Err(err) => {
                let err_text = err.to_string();
                init_error = Some(err_text.clone());
                let _ = response_tx.send(WorkerEvent::TranslatorInitFailed(err_text));
            }
        }

        for request in request_rx {
            let response = match translator.as_ref() {
                Some(instance) => instance
                    .translate(&request.text, &request.source_lang, &request.target_lang)
                    .map(TranslationResponse::Ok)
                    .unwrap_or_else(|err| TranslationResponse::Err(err.to_string())),
                None => {
                    let err_text = init_error
                        .as_deref()
                        .map(|err| format!("Translator unavailable: {err}"))
                        .unwrap_or_else(|| "Translator unavailable".to_string());
                    TranslationResponse::Err(err_text)
                }
            };

            let _ = response_tx.send(WorkerEvent::Translation(response));
        }
    });

    (request_tx, response_rx, worker)
}

#[derive(Debug)]
enum WorkerEvent {
    TranslatorInitializing,
    TranslatorReady,
    TranslatorInitFailed(String),
    Translation(TranslationResponse),
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_shortcuts_match_expected_keys() {
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let shift_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        let ctrl_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);

        assert!(is_translate_shortcut(&enter));
        assert!(is_translate_shortcut(&shift_enter));
        assert!(is_translate_shortcut(&ctrl_enter));
    }

    #[test]
    fn non_shortcuts_do_not_trigger_translation() {
        let char_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let char_m = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
        let ctrl_m = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL);
        let f5 = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        assert!(!is_translate_shortcut(&char_a));
        assert!(!is_translate_shortcut(&char_m));
        assert!(!is_translate_shortcut(&ctrl_m));
        assert!(!is_translate_shortcut(&f5));
        assert!(!is_translate_shortcut(&ctrl_r));
        assert!(!is_translate_shortcut(&tab));
    }

    #[test]
    fn worker_reports_init_failure_when_model_missing() {
        let missing_model = std::env::temp_dir().join(format!(
            "petit-missing-model-{}-{}.gguf",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let config = Config {
            model_path: missing_model,
            gpu_layers: 0,
            context_size: 2048,
            threads: 1,
            log_to_file: false,
            log_path: std::path::PathBuf::from("logs/test-llama.log"),
        };

        let (tx, rx, worker) = start_translation_worker(config);

        let first = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("worker should emit initializing event");
        assert!(matches!(first, WorkerEvent::TranslatorInitializing));

        let second = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("worker should emit init failure");
        match second {
            WorkerEvent::TranslatorInitFailed(err) => {
                assert!(
                    err.contains("Model file not found"),
                    "unexpected error: {err}"
                );
            }
            other => panic!("unexpected worker event: {other:?}"),
        }

        drop(tx);
        worker.join().expect("worker thread should exit cleanly");
    }
}
