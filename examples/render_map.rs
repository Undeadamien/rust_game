use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use grid::Grid;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use rust_game::event::{Event, EventHandler};
use unicode_segmentation::UnicodeSegmentation;

fn handle_key_event(key_event: KeyEvent) {
    match key_event.code {
        _ => {}
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    char: String,
    style: Option<Style>,
}

pub struct Label {
    text: String,
    row: usize,
    col: usize,
    style: Style,
}

fn create_grid() -> (Grid<Cell>, Label) {
    let size: usize = 40;
    let empty_cell = Cell {
        char: String::from(" "),
        style: None,
    };
    let mut grid: Grid<Cell> = Grid::from_vec(vec![empty_cell; size.pow(2)], size);
    grid.fill(Cell {
        char: String::from("·"),
        style: Some(Style::new().fg(Color::Green).add_modifier(Modifier::DIM)),
    });

    let string = String::from("│ hello │");
    let graphemes: Vec<&str> = string.graphemes(true).collect();
    let width = graphemes.len();
    for (c, grapheme) in graphemes.iter().enumerate() {
        let (top, bottom) = match c {
            0 => ("╭", "╰"),
            c if c == width - 1 => ("╮", "╯"),
            _ => ("─", "─"),
        };
        grid[(1, 3 + c)] = Cell {
            char: top.into(),
            style: Some(Style::new()),
        };
        if c == 0 || c == width - 1 {
            grid[(2, 3 + c)] = Cell {
                char: (*grapheme).into(),
                style: Some(Style::new()),
            };
        }
        grid[(3, 3 + c)] = Cell {
            char: bottom.into(),
            style: Some(Style::new()),
        };
    }

    let label = Label {
        text: graphemes[1..width - 1].concat(),
        row: 2,
        col: 4,
        style: Style::new().fg(Color::Yellow),
    };

    (grid, label)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let event_handler = EventHandler::new();
    let (grid, label) = create_grid();
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &grid, &label))?;
            match event_handler.next()? {
                Event::Crossterm(event) => match event {
                    CrosstermEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        if key_event.code == KeyCode::Char('q') {
                            break Ok(());
                        };
                        handle_key_event(key_event)
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    })
}

fn render(frame: &mut Frame, grid: &Grid<Cell>, label: &Label) {
    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let horizontal = Layout::horizontal([Constraint::Percentage(100)]).spacing(1);
    let [top, main] = frame.area().layout(&vertical);
    let [area] = main.layout(&horizontal);

    let title = Line::from_iter([
        Span::from("Canvas Widget").bold(),
        Span::from(" (Press 'q' to quit)"),
    ]);
    frame.render_widget(title.centered(), top);

    render_canvas(frame, area, grid, label);
}

pub fn render_canvas(frame: &mut Frame, area: Rect, grid: &Grid<Cell>, label: &Label) {
    let map_width = grid.cols() as u16;
    let map_height = grid.rows() as u16;
    let map_area = Rect::new(
        area.x + area.width.saturating_sub(map_width) / 2,
        area.y + area.height.saturating_sub(map_height) / 2,
        map_width,
        map_height,
    );

    let buf = frame.buffer_mut();
    for row in 0..grid.rows() {
        for col in 0..grid.cols() {
            let cell = &grid[(row, col)];
            let pos = Position::new(
                map_area.x.saturating_add(col as u16),
                map_area.y.saturating_add(row as u16),
            );
            if map_area.contains(pos) {
                buf[pos].set_symbol(&cell.char);
                if let Some(style) = cell.style {
                    buf[pos].set_style(style);
                }
            }
        }
    }

    let label_pos = Position::new(
        map_area.x.saturating_add(label.col as u16),
        map_area.y.saturating_add(label.row as u16),
    );
    if map_area.contains(label_pos) {
        buf.set_string(label_pos.x, label_pos.y, &label.text, label.style);
    }
}
