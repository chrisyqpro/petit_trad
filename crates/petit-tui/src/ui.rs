// SPDX-License-Identifier: GPL-3.0-or-later

//! UI rendering with ratatui

use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, Focus};

/// Render the application UI
pub fn render(app: &App, frame: &mut Frame) {
    let size = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(size);

    let header = header_widget(app);
    frame.render_widget(header, layout[0]);

    let input_area = layout[1];
    let output_area = layout[2];

    let focus_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let border_style = Style::default().fg(Color::Gray);

    let input_block = Block::default()
        .title("Input")
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Input {
            focus_style
        } else {
            border_style
        });
    let input_inner = input_block.inner(input_area);
    let input = Paragraph::new(app.input.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.input_scroll, 0))
        .block(input_block);
    frame.render_widget(input, input_area);

    let output_block = Block::default()
        .title("Output")
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Output {
            focus_style
        } else {
            border_style
        });
    let output = Paragraph::new(app.output.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.output_scroll, 0))
        .block(output_block);
    frame.render_widget(output, output_area);

    if app.focus == Focus::Input {
        let (row, col) = cursor_position(&app.input, app.input_cursor);
        if let Some((x, y)) = cursor_to_screen(input_inner, row, col, app.input_scroll) {
            frame.set_cursor_position((x, y));
        }
    }

    let status_area = layout[3];
    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(status_area);

    let status = status_widget(app);
    frame.render_widget(status, status_chunks[0]);
    let hints = hints_widget(status_chunks[1]);
    frame.render_widget(hints, status_chunks[1]);
}

fn header_widget(app: &App) -> Paragraph<'static> {
    let label_style = Style::default().fg(Color::Gray);
    let value_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let focus_label = match app.focus {
        Focus::Input => "Input",
        Focus::Output => "Output",
    };

    let line = if app.compact_lang_display {
        Line::from(vec![
            Span::styled("Lang: ", label_style),
            Span::styled(
                format!("{}→{}", app.source_lang, app.target_lang),
                value_style,
            ),
            Span::raw("  "),
            Span::styled("Focus: ", label_style),
            Span::styled(focus_label, value_style),
        ])
    } else {
        Line::from(vec![
            Span::styled("Source: ", label_style),
            Span::styled(app.source_lang.clone(), value_style),
            Span::raw("  "),
            Span::styled("Target: ", label_style),
            Span::styled(app.target_lang.clone(), value_style),
            Span::raw("  "),
            Span::styled("Focus: ", label_style),
            Span::styled(focus_label, value_style),
        ])
    };

    Paragraph::new(line)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("petit-trad"))
}

fn status_widget(app: &App) -> Paragraph<'static> {
    let status_text = if app.is_loading {
        format!("{} Translating...", spinner_symbol())
    } else if let Some(prompt) = app.language_prompt() {
        prompt
    } else if let Some(message) = &app.status_message {
        message.clone()
    } else {
        "Ready".to_string()
    };

    let line = Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(Color::White),
    )]);

    Paragraph::new(line)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::DarkGray))
}

fn hints_widget(area: Rect) -> Paragraph<'static> {
    let width = area.width as usize;
    let hints = if width < 70 {
        "Ctrl+Q Quit | Enter Translate | Tab Focus"
    } else if width < 100 {
        "Ctrl+Q Quit | Enter Translate | Tab Focus | Ctrl+R Swap | Ctrl+L Clear"
    } else {
        "Ctrl+Q Quit | Enter Translate | Tab Focus | Ctrl+R Swap | Ctrl+L Clear | Ctrl+S Source | Ctrl+T Target"
    };
    let line = Line::from(vec![Span::styled(hints, Style::default().fg(Color::Gray))]);

    Paragraph::new(line)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::DarkGray))
}

fn spinner_symbol() -> char {
    const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let index = ((millis / 150) % FRAMES.len() as u128) as usize;
    FRAMES[index]
}

fn cursor_position(text: &str, cursor: usize) -> (usize, usize) {
    let mut row = 0;
    let mut col = 0;

    for (idx, ch) in text.chars().enumerate() {
        if idx >= cursor {
            break;
        }
        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (row, col)
}

fn cursor_to_screen(area: Rect, row: usize, col: usize, scroll: u16) -> Option<(u16, u16)> {
    if area.width == 0 || area.height == 0 {
        return None;
    }
    let scroll = scroll as usize;
    if row < scroll {
        return None;
    }
    let visible_row = row - scroll;
    if visible_row >= area.height as usize {
        return None;
    }

    let max_x = area.width.saturating_sub(1) as usize;
    let max_y = area.height.saturating_sub(1) as usize;
    let x = area.x + (col.min(max_x) as u16);
    let y = area.y + (visible_row.min(max_y) as u16);

    Some((x, y))
}
