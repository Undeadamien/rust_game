//! All terminal rendering.

use std::collections::HashSet;

use ratatui::layout::{Alignment, Constraint, Layout, Rect, Spacing};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border::PLAIN;
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};
use ratatui::{Frame, buffer::Buffer};

use crate::game::{Game, LogKind, Mode};
use crate::world::{Point, PoiKind, Terrain};

const GOLD: Color = Color::Rgb(214, 178, 94);
const PARCHMENT: Color = Color::Rgb(214, 206, 184);
const DIM: Color = Color::Rgb(118, 113, 104);
const CYAN: Color = Color::Rgb(118, 206, 206);
const RED: Color = Color::Rgb(205, 88, 78);
const GREEN: Color = Color::Rgb(122, 190, 122);
const MAGENTA: Color = Color::Rgb(186, 128, 190);

pub fn draw(frame: &mut Frame, game: &Game) {
    let area = frame.area();
    if area.width < 60 || area.height < 16 {
        let message = Paragraph::new("The road is too narrow.\nPlease widen the terminal.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(PARCHMENT));
        frame.render_widget(message, area);
        return;
    }

    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Length(8)])
        .spacing(Spacing::Overlap(1));
    let [main, bottom] = area.layout(&vertical);
    let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Length(38)])
        .spacing(Spacing::Overlap(1));
    let [map_area, side_area] = main.layout(&horizontal);

    draw_map(frame, game, map_area);
    draw_sidebar(frame, game, side_area);
    draw_chronicle(frame, game, bottom);

    match game.mode {
        Mode::Board => draw_board(frame, game),
        Mode::Resolve => draw_resolve(frame, game),
        Mode::Help => draw_help(frame),
        Mode::Dead => draw_dead(frame, game),
        Mode::Explore | Mode::Command => {}
    }
}

// ----------------------------------------------------------------------
// Map
// ----------------------------------------------------------------------

fn draw_map(frame: &mut Frame, game: &Game, area: Rect) {
    let title = game.world.name.clone();
    let block = panel(title, Some("t: labels".to_string()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 3 || inner.height < 3 {
        return;
    }

    let view_w = inner.width as i32;
    let view_h = inner.height as i32;
    let world_w = game.world.width as i32;
    let world_h = game.world.height as i32;
    let cam_x = (game.player.pos.x - view_w / 2).clamp(0, (world_w - view_w).max(0));
    let cam_y = (game.player.pos.y - view_h / 2).clamp(0, (world_h - view_h).max(0));

    let route: HashSet<(i32, i32)> = game
        .route
        .as_ref()
        .map(|path| path.iter().map(|p| (p.x, p.y)).collect())
        .unwrap_or_default();
    let target_pos = game.target_pos();

    let buf = frame.buffer_mut();
    for sy in 0..inner.height {
        for sx in 0..inner.width {
            let p = Point::new(cam_x + sx as i32, cam_y + sy as i32);
            if !game.world.in_bounds(p) {
                continue;
            }
            let (symbol, style) = map_cell(game, p, route.contains(&(p.x, p.y)), target_pos);
            if let Some(cell) = buf.cell_mut((inner.x + sx, inner.y + sy)) {
                cell.set_symbol(symbol);
                cell.set_style(style);
            }
        }
    }

    // Bracket the tracked location.
    if let Some(pos) = target_pos {
        let sx = pos.x - cam_x;
        let sy = pos.y - cam_y;
        if sx >= 0 && sy >= 0 && sx < view_w && sy < view_h {
            let style = Style::default().fg(CYAN).add_modifier(Modifier::BOLD);
            if sx > 0 {
                set_cell(buf, inner, sx - 1, sy, "[", style);
            }
            if sx + 1 < view_w {
                set_cell(buf, inner, sx + 1, sy, "]", style);
            }
        }
    }

    // The wayfarer always sits on top.
    let px = game.player.pos.x - cam_x;
    let py = game.player.pos.y - cam_y;
    if px >= 0 && py >= 0 && px < view_w && py < view_h {
        set_cell(
            buf,
            inner,
            px,
            py,
            "@",
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(45, 45, 60))
                .add_modifier(Modifier::BOLD),
        );
    }

    draw_labels(buf, game, inner, cam_x, cam_y, target_pos);
}

fn map_cell(
    game: &Game,
    p: Point,
    on_route: bool,
    target_pos: Option<Point>,
) -> (&'static str, Style) {
    if p == game.player.pos {
        return ("@", Style::default());
    }
    if let Some(poi) = game.world.poi_at(p) {
        let mut style = Style::default().fg(poi_color(poi.kind));
        if poi.cleared {
            style = style.fg(DIM);
        }
        if target_pos == Some(p) {
            style = style.fg(Color::White).add_modifier(Modifier::BOLD);
        }
        let glyph = if poi.cleared { "x" } else { poi_glyph(poi.kind) };
        return (glyph, style);
    }
    if game.world.settlement_at(p).is_some() {
        let mut style = Style::default().fg(GOLD);
        if target_pos == Some(p) {
            style = style.fg(Color::White).add_modifier(Modifier::BOLD);
        }
        return ("⌂", style);
    }
    if on_route {
        return (
            "*",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        );
    }
    if game.world.is_road(p) {
        return ("=", Style::default().fg(Color::Rgb(146, 118, 78)));
    }

    match game.world.terrain(p) {
        Terrain::Water => {
            if game.world.texture(p) > 0.86 {
                ("≈", Style::default().fg(Color::Rgb(82, 126, 176)))
            } else {
                ("~", Style::default().fg(Color::Rgb(62, 102, 150)))
            }
        }
        Terrain::Swamp => ("\"", Style::default().fg(Color::Rgb(102, 118, 76)).dim()),
        Terrain::Grass => {
            if game.world.texture(p) > 0.88 {
                (",", Style::default().fg(Color::Rgb(88, 136, 88)))
            } else {
                (".", Style::default().fg(Color::Rgb(96, 148, 96)))
            }
        }
        Terrain::Forest => ("♣", Style::default().fg(Color::Rgb(56, 116, 78))),
        Terrain::Hills => ("∩", Style::default().fg(Color::Rgb(150, 140, 96))),
        Terrain::Mountain => ("▲", Style::default().fg(Color::Rgb(140, 140, 152))),
    }
}

fn poi_glyph(kind: PoiKind) -> &'static str {
    match kind {
        PoiKind::Nest => "☠",
        PoiKind::Ruin => "◈",
        PoiKind::Cave => "◌",
        PoiKind::Shrine => "✦",
        PoiKind::Camp => "⚑",
        PoiKind::Grove => "♠",
    }
}

fn poi_color(kind: PoiKind) -> Color {
    match kind {
        PoiKind::Nest => RED,
        PoiKind::Ruin => Color::Rgb(180, 140, 100),
        PoiKind::Cave => Color::Rgb(150, 145, 140),
        PoiKind::Shrine => GOLD,
        PoiKind::Camp => Color::Rgb(210, 130, 90),
        PoiKind::Grove => GREEN,
    }
}

fn draw_labels(
    buf: &mut Buffer,
    game: &Game,
    inner: Rect,
    cam_x: i32,
    cam_y: i32,
    target_pos: Option<Point>,
) {
    let view_w = inner.width as i32;
    let view_h = inner.height as i32;
    if view_w <= 4 {
        return;
    }

    // Reserve every glyph so a label never hides a place.
    let mut occupied: HashSet<(i32, i32)> = HashSet::new();
    for settlement in &game.world.settlements {
        occupied.insert((settlement.pos.x - cam_x, settlement.pos.y - cam_y));
    }
    for poi in &game.world.pois {
        occupied.insert((poi.pos.x - cam_x, poi.pos.y - cam_y));
    }
    occupied.insert((game.player.pos.x - cam_x, game.player.pos.y - cam_y));

    struct Label {
        screen: (i32, i32),
        text: String,
        style: Style,
        priority: u8,
    }

    let mut labels = Vec::new();
    for poi in &game.world.pois {
        let selected = target_pos == Some(poi.pos);
        if !game.show_labels && !selected {
            continue;
        }
        let sx = poi.pos.x - cam_x;
        let sy = poi.pos.y - cam_y;
        if sx < 0 || sy < 0 || sx >= view_w || sy >= view_h {
            continue;
        }
        let (text, style) = if selected {
            (
                format!("» {}", poi.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
        } else if poi.cleared {
            (
                poi.name.clone(),
                Style::default().fg(Color::Rgb(90, 88, 82)),
            )
        } else {
            (
                poi.name.clone(),
                Style::default().fg(poi_color(poi.kind)).dim(),
            )
        };
        labels.push(Label {
            screen: (sx, sy),
            text,
            style,
            priority: if selected { 0 } else { 2 },
        });
    }

    for settlement in &game.world.settlements {
        let selected = target_pos == Some(settlement.pos);
        if !game.show_labels && !selected {
            continue;
        }
        let sx = settlement.pos.x - cam_x;
        let sy = settlement.pos.y - cam_y;
        if sx < 0 || sy < 0 || sx >= view_w || sy >= view_h {
            continue;
        }
        let (text, style) = if selected {
            (
                format!("» {}", settlement.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (settlement.name.clone(), Style::default().fg(GOLD).dim())
        };
        labels.push(Label {
            screen: (sx, sy),
            text,
            style,
            priority: if selected { 0 } else { 1 },
        });
    }

    labels.sort_by_key(|label| {
        let distance = (label.screen.0 - view_w / 2).abs() + (label.screen.1 - view_h / 2).abs();
        (label.priority, distance)
    });

    for label in labels {
        let (gx, gy) = label.screen;
        let width = label.text.chars().count() as i32;
        if gy < 0 || gy >= view_h || width == 0 {
            continue;
        }

        let right_start = gx + 2;
        let right_ok = right_start + width <= view_w
            && (right_start..right_start + width).all(|x| !occupied.contains(&(x, gy)));
        let left_start = gx - 2 - width;
        let left_ok = left_start >= 0
            && (left_start..left_start + width).all(|x| !occupied.contains(&(x, gy)));

        let start = if right_ok {
            right_start
        } else if left_ok {
            left_start
        } else {
            continue;
        };

        for x in start..start + width {
            occupied.insert((x, gy));
        }
        buf.set_stringn(
            inner.x + start as u16,
            inner.y + gy as u16,
            &label.text,
            width as usize,
            label.style,
        );
    }
}

fn set_cell(buf: &mut Buffer, inner: Rect, x: i32, y: i32, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((inner.x + x as u16, inner.y + y as u16)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

// ----------------------------------------------------------------------
// Sidebar
// ----------------------------------------------------------------------

fn draw_sidebar(frame: &mut Frame, game: &Game, area: Rect) {
    let block = panel("Wayfarer", None);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            "♥".repeat(game.player.hp.max(0) as usize),
            Style::default().fg(RED),
        ),
        Span::styled(
            "♡".repeat((game.player.max_hp - game.player.hp).max(0) as usize),
            Style::default().fg(Color::Rgb(90, 70, 70)),
        ),
        Span::styled(
            format!("  {}/{}", game.player.hp, game.player.max_hp),
            Style::default().fg(PARCHMENT),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Crowns ", Style::default().fg(DIM)),
        Span::styled(
            format!("{}", game.player.crowns),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   Steel ", Style::default().fg(DIM)),
        Span::styled(
            format!("{}", game.player.steel),
            Style::default().fg(PARCHMENT),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(game.phase_glyph(), Style::default().fg(GOLD)),
        Span::styled(
            format!(" Day {} · {}", game.day(), game.phase()),
            Style::default().fg(DIM),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(section("Tracked"));
    if let Some(name) = game.target_name() {
        lines.push(Line::from(Span::styled(
            name.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        let danger = game
            .target_danger()
            .map(|d| format!(" · {}", danger_text(d)))
            .unwrap_or_default();
        lines.push(Line::from(Span::styled(
            format!(
                "{} ~{} leagues {}{}",
                game.target_kind().unwrap_or("Place"),
                game.target_leagues().unwrap_or(1),
                game.target_direction().unwrap_or("here"),
                danger
            ),
            Style::default().fg(CYAN),
        )));
        lines.push(Line::from(Span::styled(
            "route marked in cyan",
            Style::default().fg(DIM),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "n / N  survey points of interest",
            Style::default().fg(DIM),
        )));
        lines.push(Line::from(Span::styled(
            "gg     mark nearest settlement",
            Style::default().fg(DIM),
        )));
    }
    lines.push(Line::from(""));

    lines.push(section("Contracts"));
    let active = game.active_contracts();
    if active.is_empty() {
        lines.push(Line::from(Span::styled(
            "No active contracts.",
            Style::default().fg(DIM),
        )));
    } else {
        for &id in active.iter().take(3) {
            let contract = &game.contracts[id];
            lines.push(Line::from(Span::styled(
                format!("▸ {}", contract.title),
                Style::default().fg(PARCHMENT),
            )));
            let destination = game.world.pois[contract.poi].name.clone();
            lines.push(Line::from(Span::styled(
                format!("  {} · {}c", destination, contract.reward),
                Style::default().fg(DIM),
            )));
        }
        if active.len() > 3 {
            lines.push(Line::from(Span::styled(
                format!("  +{} more", active.len() - 3),
                Style::default().fg(DIM),
            )));
        }
    }
    if game.completed_count() > 0 {
        lines.push(Line::from(Span::styled(
            format!("Completed: {}", game.completed_count()),
            Style::default().fg(GREEN).dim(),
        )));
    }
    lines.push(Line::from(""));

    lines.push(section("Here"));
    lines.push(Line::from(Span::styled(
        game.location_description(),
        Style::default().fg(PARCHMENT),
    )));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_uppercase(),
        Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
    ))
}

// ----------------------------------------------------------------------
// Chronicle
// ----------------------------------------------------------------------

fn draw_chronicle(frame: &mut Frame, game: &Game, area: Rect) {
    let block = panel("Chronicle", Some(": command · ? help".to_string()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let command_mode = game.mode == Mode::Command;
    let reserved = usize::from(command_mode);
    let visible = inner.height as usize - reserved;
    let width = inner.width.saturating_sub(2).max(1) as usize;
    let start = game.log.len().saturating_sub(visible);

    let mut lines: Vec<Line<'static>> = Vec::new();
    for entry in &game.log[start..] {
        let (glyph, color) = log_style(entry.kind);
        lines.push(Line::from(vec![
            Span::styled(format!("{glyph} "), Style::default().fg(color)),
            Span::styled(
                clip(&entry.text, width),
                Style::default().fg(if entry.kind == LogKind::System {
                    DIM
                } else {
                    PARCHMENT
                }),
            ),
        ]));
    }

    if command_mode {
        let prompt_width = inner.width.saturating_sub(1) as usize;
        let command = clip_tail(&game.command, prompt_width);
        lines.push(Line::from(vec![
            Span::styled(":", Style::default().fg(GOLD)),
            Span::styled(command, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(DIM)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn log_style(kind: LogKind) -> (&'static str, Color) {
    match kind {
        LogKind::Info => ("·", PARCHMENT),
        LogKind::Good => ("+", GREEN),
        LogKind::Bad => ("!", RED),
        LogKind::Contract => ("§", GOLD),
        LogKind::Travel => ("→", CYAN),
        LogKind::System => (":", MAGENTA),
    }
}

fn clip(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let count = text.chars().count();
    if count <= width {
        return text.to_string();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn clip_tail(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return text.to_string();
    }
    chars[chars.len() - width..].iter().collect()
}

// ----------------------------------------------------------------------
// Overlays
// ----------------------------------------------------------------------

fn draw_board(frame: &mut Frame, game: &Game) {
    let area = popup_area(frame.area(), 78, 21);
    frame.render_widget(Clear, area);
    let settlement = game
        .current_settlement()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "the road".to_string());
    let block = panel(format!("Contract Board · {settlement}"), None);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let contracts = game.board_contracts();
    let width = inner.width.saturating_sub(2).max(1) as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();

    if contracts.is_empty() {
        lines.push(Line::from(Span::styled(
            "The board is bare.",
            Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Come back after the roads have swallowed someone.",
            Style::default().fg(DIM),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} available", contracts.len()),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  ·  j/k choose  ·  Enter accept",
                Style::default().fg(DIM),
            ),
        ]));
        lines.push(Line::from(""));

        for (position, &id) in contracts.iter().enumerate() {
            let contract = &game.contracts[id];
            let selected = position == game.board_sel;
            let marker = if selected { "▸ " } else { "  " };
            let base = if selected {
                Style::default().bg(Color::Rgb(48, 44, 34))
            } else {
                Style::default()
            };
            let title_style = if selected {
                base.fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                base.fg(PARCHMENT)
            };
            lines.push(Line::from(vec![
                Span::styled(marker, base.fg(GOLD)),
                Span::styled(clip(&contract.title, width.saturating_sub(2)), title_style),
                Span::styled(format!("  {}", danger_text(contract.danger)), base.fg(RED)),
                Span::styled(format!("  {}c", contract.reward), base.fg(GOLD)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  ", base),
                Span::styled(
                    clip(&contract.brief, width.saturating_sub(2)),
                    base.fg(if selected { PARCHMENT } else { DIM }),
                ),
            ]));
            lines.push(Line::from(""));
        }

        if let Some(&id) = contracts.get(game.board_sel) {
            let contract = &game.contracts[id];
            let poi = &game.world.pois[contract.poi];
            lines.push(Line::from(vec![
                Span::styled("Patron  ", Style::default().fg(DIM)),
                Span::styled(contract.patron.clone(), Style::default().fg(PARCHMENT)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Mark    ", Style::default().fg(DIM)),
                Span::styled(
                    format!("{} ({})", poi.name, poi.kind.label()),
                    Style::default().fg(CYAN),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Esc close",
        Style::default().fg(DIM),
    )));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_resolve(frame: &mut Frame, game: &Game) {
    let area = popup_area(frame.area(), 78, 19);
    frame.render_widget(Clear, area);
    let Some(contract_id) = game.resolve_contract_index() else {
        return;
    };
    let contract = &game.contracts[contract_id];
    let block = panel(format!("Contract · {}", contract.title), None);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width.saturating_sub(2).max(1) as usize;
    let dc = 5 + contract.danger as i32 * 3;
    let steel = game.player.steel * 2;
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            contract.monster.clone(),
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   {}", danger_text(contract.danger)),
            Style::default().fg(RED),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        clip(&contract.brief, width),
        Style::default().fg(PARCHMENT),
    )));
    lines.push(Line::from(""));
    if let Some(secret) = &contract.secret {
        lines.push(Line::from(vec![
            Span::styled("Rumor  ", Style::default().fg(MAGENTA)),
            Span::styled(
                clip(secret, width.saturating_sub(7)),
                Style::default().fg(MAGENTA).add_modifier(Modifier::ITALIC),
            ),
        ]));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(vec![
        Span::styled("Your steel  ", Style::default().fg(DIM)),
        Span::styled(format!("{steel}"), Style::default().fg(PARCHMENT)),
        Span::styled(format!("   Challenge  {dc}"), Style::default().fg(DIM)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "[f] Fight",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ", Style::default()),
        if contract.secret.is_some() {
            Span::styled(
                "[s] Spare",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("[s] Spare (nothing listens)", Style::default().fg(DIM))
        },
        Span::styled("   [Esc] Withdraw", Style::default().fg(DIM)),
    ]));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_help(frame: &mut Frame) {
    let area = popup_area(frame.area(), 82, 26);
    frame.render_widget(Clear, area);
    let block = panel("The Wayfarer's Field Guide", None);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let keys: &[(&str, &str)] = &[
        ("hjkl / arrows", "travel (counts work: 5j, 12l)"),
        ("n / N", "mark next / previous point of interest"),
        ("gg", "mark the nearest settlement"),
        ("Enter", "read the contract board / inspect a place"),
        ("c", "open the contract board (in a settlement)"),
        ("R", "rest at a settlement and recover"),
        ("t", "toggle place-name labels"),
        (":", "command line (:w :e :seed 123 :new :help :q)"),
        ("q", "quit"),
    ];
    let mut lines: Vec<Line<'static>> = Vec::new();
    for &(key, description) in keys {
        lines.push(Line::from(vec![
            Span::styled(format!("{key:<14}"), Style::default().fg(GOLD)),
            Span::styled(description, Style::default().fg(PARCHMENT)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "In a fight:  [f] fight   [s] spare (when there is a secret)   [Esc] withdraw",
        Style::default().fg(DIM),
    )));
    lines.push(Line::from(""));
    lines.push(section("Map legend"));
    lines.push(Line::from(Span::styled(
        "@ you   ⌂ settlement   ☠ nest   ◈ ruin   ◌ cave   ✦ shrine   ⚑ camp   ♠ grove",
        Style::default().fg(PARCHMENT),
    )));
    lines.push(Line::from(Span::styled(
        ". grass   ♣ forest   ∩ hills   ▲ mountain   ~ water   \" marsh   = road   * marked route",
        Style::default().fg(DIM),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Saves are written to wayfarer_save.json unless you give a path: :w saves/old.json",
        Style::default().fg(DIM),
    )));
    lines.push(Line::from(Span::styled(
        "Esc closes this guide.",
        Style::default().fg(DIM),
    )));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_dead(frame: &mut Frame, game: &Game) {
    let area = popup_area(frame.area(), 66, 14);
    frame.render_widget(Clear, area);
    let block = panel("The Road Ends", None);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "YOU DIED",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "Day {} · {} contracts completed · {} crowns",
                game.day(),
                game.completed_count(),
                game.player.crowns
            ),
            Style::default().fg(PARCHMENT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "The wilds keep what they take.",
            Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("r", Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
            Span::styled(" wake in a new world     ", Style::default().fg(PARCHMENT)),
            Span::styled("q", Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
            Span::styled(" quit", Style::default().fg(PARCHMENT)),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        inner,
    );
}

// ----------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------

fn panel(title: impl Into<String>, hint: Option<String>) -> Block<'static> {
    let mut title_spans = vec![
        Span::raw(PLAIN.horizontal_top.to_string()),
        Span::styled("[", Style::default().fg(Color::White)),
        Span::styled(title.into(), Style::default().fg(GOLD)),
        Span::styled("]", Style::default().fg(Color::White)),
    ];
    if let Some(hint) = hint {
        title_spans.push(Span::styled(format!(" {hint}"), Style::default().fg(DIM)));
    }
    Block::bordered()
        .border_type(BorderType::Plain)
        .merge_borders(MergeStrategy::Fuzzy)
        .title_alignment(Alignment::Left)
        .title(title_spans)
}

fn danger_text(danger: u8) -> String {
    let filled = "◆".repeat(danger.min(5) as usize);
    let empty = "◇".repeat((5 - danger.min(5)) as usize);
    format!("{filled}{empty}")
}

fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4)).max(24);
    let height = height.min(area.height.saturating_sub(4)).max(6);
    Rect::new(
        area.x + (area.width.saturating_sub(width)) / 2,
        area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ContractStatus;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_at(game: &Game, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, game)).unwrap();
        let buffer = terminal.backend().buffer();
        buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn every_mode_renders_without_panicking() {
        let mut game = Game::new(123);
        game.cycle_target(true, 3);

        for (width, height) in [(60, 16), (80, 24), (120, 40), (200, 50)] {
            for mode in [
                Mode::Explore,
                Mode::Board,
                Mode::Help,
                Mode::Dead,
                Mode::Command,
            ] {
                game.mode = mode;
                let text = render_at(&game, width, height);
                assert!(!text.trim().is_empty(), "blank screen at {width}x{height}");
            }
        }
    }

    #[test]
    fn resolve_overlay_renders() {
        let mut game = Game::new(456);
        let contract = game
            .contracts
            .iter_mut()
            .find(|contract| contract.secret.is_some())
            .expect("a secret contract");
        contract.status = ContractStatus::Accepted;
        let poi = contract.poi;
        game.player.pos = game.world.pois[poi].pos;
        game.mode = Mode::Resolve;

        let text = render_at(&game, 100, 32);
        assert!(text.contains("Fight"));
        assert!(text.contains("Spare"));
    }

    #[test]
    fn tiny_terminal_shows_a_hint() {
        let game = Game::new(1);
        let text = render_at(&game, 40, 10);
        assert!(text.contains("widen"));
    }
}
