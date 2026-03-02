//! Monster Battle Composer — TUI music composition tool.
//!
//! Uses the `monster-battle-audio` engine's mini‑notation to compose and
//! preview game tracks interactively.

mod app;
mod project;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    let mut app = app::App::new()?;
    app.run()
}
