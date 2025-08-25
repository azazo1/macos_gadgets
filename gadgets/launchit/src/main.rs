mod app;
mod ui;
mod launchctl;

use anyhow::Result;
use app::App;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()?;
    Ok(())
}
