use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn initialize_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .with_ansi(false)
                .with_writer(std::fs::File::create(
                    std::env::current_dir()?.join("rust_game.log"),
                )?),
        )
        .with(ErrorLayer::default())
        .init();
    Ok(())
}
