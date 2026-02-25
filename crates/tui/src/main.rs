mod app;
mod audio;
mod modales;
mod screens;
mod sprites;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialise the audio engine (no-op if the `audio` feature is disabled).
    audio::init();

    let mut app = app::App::new()?;
    app.run()
}
