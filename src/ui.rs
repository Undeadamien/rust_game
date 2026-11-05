use crate::app::{App, State};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

//todo: dedup the border?

pub fn render_play(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let block = Block::bordered()
        .title("rust_game")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let paragraph = Paragraph::new(format!(
        concat!("Current state: {:?}\n", "Press `Ctrl-C` to stop exit."),
        app.state
    ))
    .block(block)
    .centered();
    frame.render_widget(paragraph, area);
}

pub fn render_menu(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let block = Block::bordered()
        .title("rust_game")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    //todo: replace the placeholder and create the menu
    let text: Text = Text::from(format!(
        concat!("Current state: {:?}\n", "Press `Ctrl-C` to stop exit.\n",),
        app.state
    ));
    let count_line = text.lines.len() as u16; //todo: double check this line
    let paragraph = Paragraph::new(text).bg(Color::Black).centered();
    let [mid] = Layout::vertical([Constraint::Length(count_line)]) //todo: should consider line_count()
        .flex(Flex::Center)
        .areas(inner);
    frame.render_widget(paragraph, mid);
}

pub fn render(app: &App, frame: &mut Frame) {
    match app.state {
        State::Play => render_play(app, frame),
        State::Menu => render_menu(app, frame),
    }
}
