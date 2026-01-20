// SPDX-License-Identifier: GPL-3.0-or-later

//! petit-tui: Terminal interface for petit_trad
//!
//! A TUI application for translating text using TranslateGemma.

use anyhow::Result;

mod app;
mod ui;

fn main() -> Result<()> {
    println!("petit_trad v{}", env!("CARGO_PKG_VERSION"));
    println!("Translation TUI - Coming soon!");
    println!();
    println!("This is a placeholder. Implementation pending:");
    println!("  - Model loading (petit-core)");
    println!("  - TUI interface (ratatui)");
    Ok(())
}
