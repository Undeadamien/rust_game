use crate::event::{AppEvent, Event, EventHandler};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

#[derive(Clone, Debug)]
pub enum State {
    Menu,
    Game,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub state: State,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            state: State::Menu,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| crate::ui::render(&self, frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    //todo: find a better way to handle state scoped keybinds
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        //debug
        if key_event.code == KeyCode::Esc {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }
        if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }
        match self.state {
            State::Menu => match key_event.code {
                KeyCode::Char(' ') => self.state = State::Game, //debug
                _ => {}
            },
            State::Game => match key_event.code {
                KeyCode::Char(' ') => self.state = State::Menu, //debug
                _ => {}
            },
            // _ => {}
        }
        Ok(())
    }

    //todo: check how animation should be handled
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}
