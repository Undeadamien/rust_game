use crate::app::App;
use ratatui::{Frame, widgets::Paragraph};

use super::border;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let paragraph = Paragraph::new(format!(
        concat!("Current state: {:?}\n", "Press `Ctrl-C` to stop exit."),
        app.state
    ))
    .block(border())
    .centered();
    frame.render_widget(paragraph, area);
}
