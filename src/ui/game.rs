use crate::game::Game;
use crate::game::map::Terrain;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Padding, Paragraph};

const NODE_SPACING: u16 = 2;

//todo: should I even render a map?
pub fn render_map(frame: &mut Frame, rect: Rect, game: &Game) {
    let block = Block::bordered().title(" Map ");
    let inner = block.inner(rect);
    let map = &game.map;
    let nodes = map.nodes();

    frame.render_widget(block, rect);
    let buf = frame.buffer_mut();
    let current_node = nodes[game.current as usize].clone();
    for node in nodes {
        let pos = Position::new(
            inner.x + (node.x * NODE_SPACING) + inner.width / 2 - current_node.x,
            inner.y + (node.y * NODE_SPACING) + inner.height / 2 - current_node.y,
        );
        if !inner.contains(pos) {
            continue;
        }
        match node.terrain {
            Terrain::City => buf[pos].set_symbol(""),
            Terrain::Forest => buf[pos].set_symbol("󱢗"),
            Terrain::Ruin => buf[pos].set_symbol("🬩"),
        };
        if node.id == game.current {
            buf[pos].set_style(Style::new().fg(Color::Red));
        }
    }
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
