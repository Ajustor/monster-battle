mod app;
mod screens;
mod sprites;

use anyhow::Result;

fn main() -> Result<()> {
    let mut app = app::App::new()?;
    app.run()
}
