use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use super::create_border;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let border = create_border();
    let inner = border.inner(area);
    frame.render_widget(border, area);

    let title = Paragraph::new("Rust Game").centered();
    let items: Vec<ListItem> = app
        .menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let sel = i == app.menu.selected;
            ListItem::new(
                Line::from(vec![
                    Span::raw(if sel { "> " } else { "" }),
                    Span::raw(item.label),
                    Span::raw(if sel { " <" } else { "" }),
                ])
                .centered(),
            )
        })
        .collect();
    let width = (app.menu.items.iter().map(|f| f.label.len()).max().unwrap() + 4) as u16;
    let list = List::new(items).highlight_style(Style::new());

    let [top, _, bottom] = Layout::vertical([
        Constraint::Length(1), //todo: use _.line_count()
        Constraint::Length(1),
        Constraint::Length(list.len() as u16),
    ])
    .flex(Flex::Center)
    .areas(inner);

    let [list_slot] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(bottom);

    frame.render_widget(title, top);
    frame.render_widget(list, list_slot);
}
