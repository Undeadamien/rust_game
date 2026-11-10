use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Spacing};

use super::create_border;
use super::create_border_merged;

pub fn render(_app: &App, frame: &mut Frame) {
    let area = frame.area();
    let border = create_border();
    let inner = border.inner(area);
    frame.render_widget(border, area);

    let [left, middle, right] = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .spacing(Spacing::Overlap(1))
    .areas(inner);
    let [map_area, log_area] = Layout::vertical([Constraint::Fill(1), Constraint::Max(20)])
        .spacing(Spacing::Overlap(1))
        .areas(middle);

    frame.render_widget(create_border_merged(), left);
    frame.render_widget(create_border_merged(), right);
    frame.render_widget(create_border_merged(), map_area);
    frame.render_widget(create_border_merged(), log_area);
}
