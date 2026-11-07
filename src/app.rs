use crate::event::{AppEvent, Event, EventHandler};
use crate::game::{Game, LogKind, Mode, ResolveChoice, random_seed};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

pub struct App {
    pub running: bool,
    pub game: Game,
    pub events: EventHandler,
}

impl App {
    pub fn new(seed: Option<u64>, load: Option<String>) -> Self {
        let game = match load {
            Some(path) => match Game::load(&path) {
                Ok(game) => game,
                Err(error) => {
                    let mut game = Game::new(seed.unwrap_or_else(random_seed));
                    game.push_log(
                        LogKind::Bad,
                        format!("Could not load {path}: {error}. A new world was grown."),
                    );
                    game
                }
            },
            None => Game::new(seed.unwrap_or_else(random_seed)),
        };

        Self {
            running: true,
            game,
            events: EventHandler::new(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running && !self.game.should_quit {
            terminal.draw(|frame| crate::ui::draw(frame, &self.game))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => {
                if let crossterm::event::Event::Key(key_event) = event
                    && key_event.kind == KeyEventKind::Press
                {
                    self.handle_key_event(key_event);
                }
            }
            Event::App(AppEvent::Quit) => self.quit(),
        }
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key_event.code, KeyCode::Char('c' | 'C'))
        {
            self.quit();
            return;
        }

        match self.game.mode {
            Mode::Explore => self.handle_explore_key(key_event),
            Mode::Command => self.handle_command_key(key_event),
            Mode::Board => self.handle_board_key(key_event),
            Mode::Resolve => self.handle_resolve_key(key_event),
            Mode::Help => {
                if matches!(
                    key_event.code,
                    KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter
                ) {
                    self.game.mode = Mode::Explore;
                }
            }
            Mode::Dead => match key_event.code {
                KeyCode::Char('r') => self.game.restart(),
                KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                _ => {}
            },
        }
    }

    fn handle_explore_key(&mut self, key: KeyEvent) {
        // Vim-style count prefix: 5j, 12l, 3n.
        if let KeyCode::Char(c) = key.code
            && let Some(digit) = c.to_digit(10)
        {
            if self.game.count.is_none() && digit == 0 {
                return;
            }
            let count = self
                .game
                .count
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add(digit)
                .clamp(1, 999);
            self.game.count = Some(count);
            self.game.pending_g = false;
            return;
        }

        let count = self.game.count.take().unwrap_or(1);

        if !matches!(key.code, KeyCode::Char('g')) {
            self.game.pending_g = false;
        }

        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.game.move_player(-1, 0, count),
            KeyCode::Char('j') | KeyCode::Down => self.game.move_player(0, 1, count),
            KeyCode::Char('k') | KeyCode::Up => self.game.move_player(0, -1, count),
            KeyCode::Char('l') | KeyCode::Right => self.game.move_player(1, 0, count),
            KeyCode::Char('n') => self.game.cycle_target(true, count),
            KeyCode::Char('N') => self.game.cycle_target(false, count),
            KeyCode::Char('g') => {
                if self.game.pending_g {
                    self.game.pending_g = false;
                    self.game.mark_nearest_settlement();
                } else {
                    self.game.pending_g = true;
                }
            }
            KeyCode::Enter | KeyCode::Char('e') => self.game.interact(),
            KeyCode::Char('c') => self.game.open_board(),
            KeyCode::Char('R') => self.game.rest(),
            KeyCode::Char('t') => {
                self.game.show_labels = !self.game.show_labels;
                let message = if self.game.show_labels {
                    "Place-name labels shown."
                } else {
                    "Place-name labels hidden. The tracked place still shows."
                };
                self.game.push_log(LogKind::System, message.to_string());
            }
            KeyCode::Char(':') => {
                self.game.mode = Mode::Command;
                self.game.command.clear();
            }
            KeyCode::Char('?') => self.game.mode = Mode::Help,
            KeyCode::Esc => {
                if self.game.target.is_some() {
                    self.game.clear_target();
                    self.game.pending_g = false;
                    self.game.count = None;
                    self.game
                        .push_log(LogKind::Travel, "The trail is forgotten.".to_string());
                }
            }
            KeyCode::Char('q') => self.quit(),
            _ => {}
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.game.command.clear();
                self.game.mode = Mode::Explore;
            }
            KeyCode::Enter => self.game.execute_command(),
            KeyCode::Backspace => {
                self.game.command.pop();
            }
            KeyCode::Char(c) if !c.is_control() && self.game.command.chars().count() < 160 => {
                self.game.command.push(c);
            }
            _ => {}
        }
    }

    fn handle_board_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.game.move_board_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.game.move_board_selection(-1),
            KeyCode::Enter => self.game.accept_board_selection(),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') => {
                self.game.mode = Mode::Explore;
            }
            KeyCode::Char('?') => self.game.mode = Mode::Help,
            _ => {}
        }
    }

    fn handle_resolve_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('f') | KeyCode::Enter => self.game.resolve(ResolveChoice::Fight),
            KeyCode::Char('s') => self.game.resolve(ResolveChoice::Spare),
            KeyCode::Esc | KeyCode::Char('q') => self.game.resolve(ResolveChoice::Leave),
            _ => {}
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}
