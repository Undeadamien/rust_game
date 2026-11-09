#![allow(unused)]

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, ListState, Paragraph},
};

use super::border;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let border = border();
    let inner = border.inner(area);
    frame.render_widget(border, area);

    let title = Paragraph::new("Rust Game").centered();

    //todo: find a way to add "> {} <" for the selected
    let labels = app.menu.items.clone();
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
    let width = (labels.iter().map(|f| f.label.len()).max().unwrap() + 4) as u16;
    let list = List::new(items).highlight_style(Style::new());

    let [top, _, bottom] = Layout::vertical([
        Constraint::Length(1), //todo: use _.line_count()
        Constraint::Length(1),
        Constraint::Percentage(list.len() as u16),
    ])
    .flex(Flex::Center)
    .areas(inner);

    let [list_slot] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(bottom);

    let mut list_state = ListState::default();
    list_state.select(Some(app.menu.selected));

    frame.render_widget(title, top);
    frame.render_stateful_widget(list, list_slot, &mut list_state);
}
