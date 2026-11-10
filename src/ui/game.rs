use crate::app::App;
use ratatui::Frame;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::Canvas;

use super::create_border;

pub fn render(_app: &App, frame: &mut Frame) {
    let area = frame.area();
    let border = create_border();
    let inner = border.inner(area);
    frame.render_widget(border, area);

    let canvas = Canvas::default()
        .x_bounds([-40.0, 40.0])
        .y_bounds([-40.0, 40.0])
        .marker(Marker::Braille)
        .paint(|ctx| {
            ctx.layer();
            ctx.print(0.0, 0.0, "@"); //todo: replace with the player coordinate
        });
    frame.render_widget(canvas, inner);
}
