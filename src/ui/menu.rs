use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::Paragraph,
};

use super::border;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let border = border();
    let inner = border.inner(area);
    frame.render_widget(border, area);

    //todo: replace the placeholder and create the menu
    let text: Text = Text::from(format!(
        concat!("Current state: {:?}\n", "Press `Ctrl-C` to stop exit.\n",),
        app.state
    ));
    let count_line = text.lines.len() as u16; //todo: double check
    let paragraph = Paragraph::new(text).bg(Color::Black).centered();
    let [mid] = Layout::vertical([Constraint::Length(count_line)]) //todo: should consider line_count()
        .flex(Flex::Center)
        .areas(inner);
    frame.render_widget(paragraph, mid);
}
