use crate::app::App;

pub mod app;
pub mod event;
pub mod game;
pub mod rng;
pub mod ui;
pub mod world;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let (seed, load_path) = parse_args();
    let terminal = ratatui::init();
    let result = App::new(seed, load_path).run(terminal);
    ratatui::restore();
    result
}

fn parse_args() -> (Option<u64>, Option<String>) {
    let mut seed = None;
    let mut load = None;
    let mut args = std::env::args().skip(1);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--seed" | "-s" => {
                seed = args.next().and_then(|value| value.parse::<u64>().ok());
            }
            "--load" | "-l" => {
                load = args.next();
            }
            "--help" | "-h" => {
                println!("wayfarer — a tiny terminal wayfaring game");
                println!();
                println!("Usage: wayfarer [--seed N] [--load FILE]");
                println!();
                println!("  -s, --seed N     generate a world from seed N");
                println!("  -l, --load FILE  load a save file");
                println!();
                println!(
                    "In game: hjkl to travel, n/N to survey, Enter to interact, : for commands."
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("wayfarer: unknown argument {other}");
            }
        }
    }

    (seed, load)
}
