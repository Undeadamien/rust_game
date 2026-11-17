use crate::event::{AppEvent, Event, EventHandler};
use crate::game::die::DicePool;
use crate::game::map::Terrain;
use crate::game::map::{Direction, Map};
use crate::game::player::Player;
use crate::game::{Command, Game, Phase};
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
#[derive(Debug, Clone)]
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
    Menu(Menu),
    Game(Game),
}

fn new_game() -> Game {
    let mut map = Map::new();

    let city = map.add_node(String::from("City"), Terrain::City, 0, 0);
    let forest = map.add_node(String::from("Forest"), Terrain::Forest, 1, 0);
    let ruin = map.add_node(String::from("Ruin"), Terrain::Ruin, 0, 1);

    map.add_edge(city, forest);
    map.add_edge(city, ruin);

    debug!("{:#?}", map);

    Game::new(
        Player::new(String::from("Hero")),
        Phase::Travel,
        DicePool::new(vec![]),
        map,
        forest,
    )
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
            state: State::Menu(Menu::default()),
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

        let mut next_state = None;
        match &mut self.state {
            State::Menu(menu) => match key_event.code {
                KeyCode::Char('j') => menu.next(),
                KeyCode::Char('k') => menu.prev(),
                KeyCode::Enter => match menu.action() {
                    MenuAction::Quit => self.events.send(AppEvent::Quit),
                    MenuAction::NewGame => next_state = Some(State::Game(new_game())),
                },
                _ => {}
            },
            State::Game(game) => match key_event.code {
                KeyCode::Esc => next_state = Some(State::Menu(Menu::default())),
                KeyCode::Char('k') => game.update(Command::Travel(Direction::North)),
                KeyCode::Char('j') => game.update(Command::Travel(Direction::South)),
                KeyCode::Char('h') => game.update(Command::Travel(Direction::West)),
                KeyCode::Char('l') => game.update(Command::Travel(Direction::East)),
                _ => {}
            },
        }
        if let Some(state) = next_state {
            self.state = state;
        }
        Ok(())
    }

    //todo: check how animation should be handled
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}
