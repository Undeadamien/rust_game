use std::{io, time::Duration};

use crossterm::event::{self, KeyCode};
use ratatui::layout::{Alignment, Constraint, Layout, Spacing};
use ratatui::style::{Color, Style};
use ratatui::symbols::border::PLAIN;
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType};
use ratatui::{DefaultTerminal, Frame};

fn main() {
    let _ = ratatui::run(run);
}

fn run(terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
    loop {
        if quit()? {
            break;
        }
        terminal.draw(render)?;
    }
    Ok(())
}

fn render(frame: &mut Frame) {
    let horizontal = Layout::horizontal([Constraint::Ratio(3, 4), Constraint::Fill(1)])
        .spacing(Spacing::Overlap(1));
    let vertical = Layout::vertical([Constraint::Ratio(2, 3), Constraint::Fill(1)])
        .spacing(Spacing::Overlap(1));
    let [left, right] = frame.area().layout(&horizontal);
    let [top, bottom] = left.layout(&vertical);

    //dedup the blocks
    let map = Block::bordered()
        .border_type(BorderType::Plain)
        .merge_borders(MergeStrategy::Fuzzy)
        .title_alignment(Alignment::Left)
        .title(vec![
            Span::raw(PLAIN.horizontal_top.repeat(1)), //need to be extracted
            Span::styled("[", Style::default().fg(Color::White)),
            Span::styled("Placeholder", Style::default().fg(Color::Yellow)),
            Span::styled("]", Style::default().fg(Color::White)),
        ]);
    let backpack = Block::bordered()
        .border_type(BorderType::Plain)
        .merge_borders(MergeStrategy::Fuzzy)
        .title_alignment(Alignment::Left)
        .title(vec![
            Span::raw(PLAIN.horizontal_top.repeat(1)), //need to be extracted
            Span::styled("[", Style::default().fg(Color::White)),
            Span::styled("Placeholder", Style::default().fg(Color::Yellow)),
            Span::styled("]", Style::default().fg(Color::White)),
        ]);
    let info = Block::bordered()
        .border_type(BorderType::Plain)
        .merge_borders(MergeStrategy::Fuzzy)
        .title_alignment(Alignment::Left)
        .title(vec![
            Span::raw(PLAIN.horizontal_top.repeat(1)), //need to be extracted
            Span::styled("[", Style::default().fg(Color::White)),
            Span::styled("Placeholder", Style::default().fg(Color::Yellow)),
            Span::styled("]", Style::default().fg(Color::White)),
        ]);

    frame.render_widget(backpack, right);
    frame.render_widget(map, top);
    frame.render_widget(info, bottom);
}

fn quit() -> Result<bool, io::Error> {
    if event::poll(Duration::from_millis(100))? {
        let q_pressed = event::read()?
            .as_key_press_event()
            .is_some_and(|key| key.code == KeyCode::Char('q'));
        return Ok(q_pressed);
    }
    Ok(false)
}
