use crate::app::{App, State};
use crossterm::terminal;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::Text;
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

pub fn create_border_merged() -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Plain)
        .merge_borders(MergeStrategy::Exact)
}

fn render_small_term(_app: &App, frame: &mut Frame) {
    let text = Text::from("Terminal too small").centered();
    let area = frame.area().centered(
        Constraint::Length(text.width() as u16),
        Constraint::Length(1),
    );
    frame.render_widget(text, area);
}

pub fn render(app: &App, frame: &mut Frame) {
    //todo: find a better way to handle a small term
    let (width, height) = terminal::size().unwrap();
    if width < 80 || height < 24 {
        render_small_term(app, frame);
        return;
    }
    match app.state {
        State::Game => game::render(app, frame),
        State::Menu => menu::render(app, frame),
    }
}
