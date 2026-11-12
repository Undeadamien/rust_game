use crate::event::{AppEvent, Event, EventHandler};
use crate::ui;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use ratatui::DefaultTerminal;
use tracing::debug;

#[derive(Clone, Copy, Debug)]
pub struct MenuItem {
    pub label: &'static str,
    pub action: MenuAction,
}
#[derive(Debug)]
//todo: try to drop pub?
pub struct Menu {
    pub selected: usize,
    pub items: Vec<MenuItem>,
}
impl Menu {
    fn next(&mut self) {
        self.selected = std::cmp::min(self.selected + 1, self.items.len() - 1)
    }
    fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1)
    }
    fn action(&self) -> MenuAction {
        self.items[self.selected].action
    }
}
impl Default for Menu {
    fn default() -> Self {
        Self {
            selected: 0,
            items: vec![
                MenuItem {
                    label: "New Game",
                    action: MenuAction::NewGame,
                },
                MenuItem {
                    label: "Quit",
                    action: MenuAction::Quit,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MenuAction {
    NewGame,
    Quit,
}

#[derive(Clone, Debug)]
pub enum State {
    Menu,
    Game, //todo: add variant?
}

#[derive(Debug)]
pub struct App {
    pub running: bool,

    pub events: EventHandler,
    pub state: State,
    pub menu: Menu,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            state: State::Menu,
            menu: Menu::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| ui::render(&self, frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                CrosstermEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
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

    pub fn handle_key_event_menu(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') => self.state = State::Game, //debug
            KeyCode::Char('j') => self.menu.next(),
            KeyCode::Char('k') => self.menu.prev(),
            KeyCode::Enter => match self.menu.action() {
                MenuAction::Quit => self.events.send(AppEvent::Quit),
                MenuAction::NewGame => self.state = State::Game,
            },
            _ => {}
        }
    }

    pub fn handle_key_event_game(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.state = State::Menu, //todo: replace with a menu?
            _ => {}
        }
    }

    //todo: find a better way to handle state scoped keybinds
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        debug!(?key_event);
        if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }

        //todo: remove the hardcoded values
        let (width, height) = terminal::size().unwrap();
        if width < 80 || height < 24 {
            return Ok(());
        }

        match self.state {
            State::Menu => self.handle_key_event_menu(key_event),
            State::Game => self.handle_key_event_game(key_event),
        }
        Ok(())
    }

    //todo: check how animation should be handled
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}
