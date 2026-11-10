use rust_game::{app::App, logging};

fn main() -> color_eyre::Result<()> {
    logging::initialize_logging()?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
