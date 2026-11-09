use crate::app::{App, State};
use ratatui::Frame;
use ratatui::{
    layout::Alignment,
    widgets::{Block, BorderType},
};

mod menu;
mod play;

pub const NAME: &str = "rust_game";

pub fn border() -> Block<'static> {
    Block::bordered()
        .title(NAME)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Plain)
}

pub fn render(app: &App, frame: &mut Frame) {
    match app.state {
        State::Play => play::render(app, frame),
        State::Menu => menu::render(app, frame),
    }
}
