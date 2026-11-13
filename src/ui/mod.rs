use crate::app::{App, State};
use crossterm::terminal;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::text::Text;

mod game;
mod menu;

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
    match &app.state {
        State::Game(game) => game::render(game, frame),
        State::Menu(menu) => menu::render(menu, frame),
    }
}
