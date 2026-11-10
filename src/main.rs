use crate::app::App;

pub mod app;
pub mod event;
pub mod game;
pub mod logging;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    logging::initialize_logging()?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
