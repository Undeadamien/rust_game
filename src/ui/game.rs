use crate::game::Game;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Block;

pub fn render_map(frame: &mut Frame, rect: Rect, _game: &Game) {
    let block = Block::bordered().title(" Map ");
    frame.render_widget(block, rect);
}

pub fn render_log(frame: &mut Frame, rect: Rect) {
    let block = Block::bordered().title(" Log ");
    frame.render_widget(block, rect);
}

pub fn render_inventory(frame: &mut Frame, rect: Rect) {
    let block = Block::bordered().title(" Inventory ");
    frame.render_widget(block, rect);
}

pub fn render_info(frame: &mut Frame, rect: Rect) {
    let block = Block::bordered().title(" Info ");
    frame.render_widget(block, rect);
}

pub fn render(game: &Game, frame: &mut Frame) {
    let area = frame.area();

    let [top, bottom] = Layout::vertical([Constraint::Fill(1), Constraint::Max(10)]).areas(area);
    let [left, middle, right] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(60),
        Constraint::Percentage(20),
    ])
    .spacing(1)
    .areas(top);

    render_log(frame, bottom);
    render_map(frame, middle, game);
    render_info(frame, right);
    render_inventory(frame, left);
}
