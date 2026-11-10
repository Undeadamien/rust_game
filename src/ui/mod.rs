use crate::app::{App, State};
use ratatui::Frame;
use ratatui::{
    layout::Alignment,
    widgets::{Block, BorderType},
};

mod game;
mod menu;

pub fn create_border() -> Block<'static> {
    Block::bordered()
        .title(format!("[ {name} ]", name = "rust_game"))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Plain)
}

pub fn render(app: &App, frame: &mut Frame) {
    match app.state {
        State::Game => game::render(app, frame),
        State::Menu => menu::render(app, frame),
    }
}
