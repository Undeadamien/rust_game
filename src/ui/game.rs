use crate::game::Game;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Padding, Paragraph};

//todo: should I even render a map?
pub fn render_map(frame: &mut Frame, rect: Rect, _game: &Game) {
    let block = Block::bordered().title(" Map ");
    let _inner = block.inner(rect);
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

pub fn render_info(game: &Game, frame: &mut Frame, rect: Rect) {
    let block = Block::bordered()
        .title(" Info ")
        .padding(Padding::symmetric(1, 1));
    frame.render_widget(&block, rect);

    let inner = block.inner(rect);
    let vertical = Layout::vertical([Constraint::Fill(1)]);
    let [main] = inner.layout(&vertical);

    let paragraph = Paragraph::new(Text::from_iter([
        Line::from(format!("Name: {}", game.player.name)),
        Line::from(format!(
            "Lives: {}/{}",
            game.player.health, game.player.health_max
        )),
    ]));

    frame.render_widget(paragraph, main);
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
    render_info(game, frame, left);
    render_inventory(frame, right);
}
