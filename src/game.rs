//! Game state: the wayfarer, contracts, travel, combat, saving and loading.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::rng::Rng;
use crate::world::{Point, PoiKind, Settlement, World};

pub const SAVE_VERSION: u32 = 1;
pub const DEFAULT_SAVE: &str = "wayfarer_save.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Explore,
    Board,
    Resolve,
    Help,
    Dead,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogKind {
    Info,
    Good,
    Bad,
    Contract,
    Travel,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub text: String,
    pub kind: LogKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Poi(usize),
    Settlement(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub pos: Point,
    pub hp: i32,
    pub max_hp: i32,
    pub crowns: u32,
    pub steel: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractStatus {
    Available,
    Accepted,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub id: usize,
    pub settlement: usize,
    pub poi: usize,
    pub title: String,
    pub patron: String,
    pub monster: String,
    pub danger: u8,
    pub reward: u32,
    pub status: ContractStatus,
    pub brief: String,
    pub secret: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveChoice {
    Fight,
    Spare,
    Leave,
}

#[derive(Debug)]
pub struct Game {
    pub world: World,
    pub player: Player,
    pub contracts: Vec<Contract>,
    pub turn: u64,
    pub log: Vec<LogEntry>,
    pub target: Option<Target>,
    pub route: Option<Vec<Point>>,
    pub mode: Mode,
    pub board_sel: usize,
    pub command: String,
    pub show_labels: bool,
    pub pending_g: bool,
    pub count: Option<u32>,
    pub should_quit: bool,
    pub rng: Rng,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let world = World::generate(seed);
        let start = world.start_position();
        let player = Player {
            pos: start,
            hp: 12,
            max_hp: 12,
            crowns: 30,
            steel: 3,
        };
        let mut rng = Rng::new(seed ^ 0xDEAD_BEEF_CAFE_F00D);
        let contracts = generate_contracts(&world, &mut rng);
        let start_name = world.settlements[0].name.clone();

        let mut game = Self {
            world,
            player,
            contracts,
            turn: 0,
            log: Vec::new(),
            target: None,
            route: None,
            mode: Mode::Explore,
            board_sel: 0,
            command: String::new(),
            show_labels: true,
            pending_g: false,
            count: None,
            should_quit: false,
            rng,
        };

        game.push_log(
            LogKind::System,
            format!(
                "You arrive in {start_name} with {} crowns and a notched sword.",
                game.player.crowns
            ),
        );
        game.push_log(
            LogKind::Info,
            "Every settlement keeps a contract board. Stand in one and press Enter.".to_string(),
        );
        game.push_log(
            LogKind::Travel,
            "Press n / N to survey points of interest. Press ? for the key map.".to_string(),
        );
        game
    }

    // ------------------------------------------------------------------
    // Logging & time
    // ------------------------------------------------------------------

    pub fn push_log(&mut self, kind: LogKind, text: String) {
        self.log.push(LogEntry { text, kind });
        if self.log.len() > 120 {
            self.log.remove(0);
        }
    }

    pub fn day(&self) -> u64 {
        self.turn / 24 + 1
    }

    pub fn hour(&self) -> u64 {
        self.turn % 24
    }

    pub fn phase(&self) -> &'static str {
        match self.hour() {
            5..=8 => "Dawn",
            9..=15 => "Day",
            16..=19 => "Dusk",
            _ => "Night",
        }
    }

    pub fn phase_glyph(&self) -> &'static str {
        match self.phase() {
            "Dawn" | "Day" => "☀",
            _ => "☾",
        }
    }

    fn phase_change(&self, old: &'static str) -> Option<&'static str> {
        let now = self.phase();
        if now == old {
            return None;
        }
        Some(match now {
            "Dawn" => "Dawn breaks over the wilds.",
            "Day" => "The sun climbs higher.",
            "Dusk" => "Dusk gathers. The roads grow unfriendly.",
            _ => "Night falls. Somewhere, something howls.",
        })
    }

    // ------------------------------------------------------------------
    // Locations & targets
    // ------------------------------------------------------------------

    pub fn current_settlement(&self) -> Option<&Settlement> {
        self.world.settlement_at(self.player.pos)
    }

    pub fn current_poi(&self) -> Option<&crate::world::Poi> {
        self.world.poi_at(self.player.pos)
    }

    pub fn target_pos(&self) -> Option<Point> {
        match self.target? {
            Target::Poi(i) => self.world.pois.get(i).map(|p| p.pos),
            Target::Settlement(i) => self.world.settlements.get(i).map(|s| s.pos),
        }
    }

    pub fn target_name(&self) -> Option<&str> {
        match self.target? {
            Target::Poi(i) => self.world.pois.get(i).map(|p| p.name.as_str()),
            Target::Settlement(i) => self.world.settlements.get(i).map(|s| s.name.as_str()),
        }
    }

    pub fn target_kind(&self) -> Option<&'static str> {
        match self.target? {
            Target::Poi(i) => self.world.pois.get(i).map(|p| p.kind.label()),
            Target::Settlement(i) => self.world.settlements.get(i).map(|s| s.kind.label()),
        }
    }

    pub fn target_danger(&self) -> Option<u8> {
        match self.target? {
            Target::Poi(i) => self.world.pois.get(i).map(|p| p.danger),
            Target::Settlement(_) => None,
        }
    }

    pub fn target_distance(&self) -> Option<usize> {
        if let Some(route) = &self.route {
            return Some(route.len().saturating_sub(1));
        }
        let target = self.target_pos()?;
        let dx = (target.x - self.player.pos.x).unsigned_abs() as usize;
        let dy = (target.y - self.player.pos.y).unsigned_abs() as usize;
        Some(dx + dy)
    }

    pub fn target_leagues(&self) -> Option<usize> {
        self.target_distance().map(|d| (d / 3).max(1))
    }

    pub fn target_direction(&self) -> Option<&'static str> {
        let target = self.target_pos()?;
        let dx = target.x - self.player.pos.x;
        let dy = target.y - self.player.pos.y;
        Some(direction_name(dx, dy))
    }

    pub fn location_description(&self) -> String {
        if let Some(settlement) = self.current_settlement() {
            return format!("{} · {}", settlement.name, settlement.kind.label());
        }
        if let Some(poi) = self.current_poi() {
            let suffix = if poi.cleared { " · cleared" } else { "" };
            return format!("{}{suffix}", poi.name);
        }
        let terrain = self.world.terrain(self.player.pos);
        if self.world.is_road(self.player.pos) {
            format!("On the road · {}", terrain.name())
        } else {
            let mut name = terrain.name().to_string();
            if let Some(first) = name.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            name
        }
    }

    pub fn refresh_route(&mut self) {
        let route = match self.target_pos() {
            Some(target) => self.world.find_path(self.player.pos, target),
            None => None,
        };
        self.route = route;
    }

    pub fn clear_target(&mut self) {
        self.target = None;
        self.route = None;
    }

    pub fn cycle_target(&mut self, forward: bool, count: u32) {
        if self.world.pois.is_empty() {
            return;
        }
        let len = self.world.pois.len() as i64;
        let current = match self.target {
            Some(Target::Poi(i)) => Some(i as i64),
            _ => None,
        };
        let mut index = match current {
            Some(i) => i,
            None if forward => -1,
            None => 0,
        };
        for _ in 0..count.max(1) {
            index = if forward {
                (index + 1).rem_euclid(len)
            } else {
                (index - 1).rem_euclid(len)
            };
        }
        let poi_id = index as usize;
        let name = self.world.pois[poi_id].name.clone();
        let kind = self.world.pois[poi_id].kind.label();
        self.target = Some(Target::Poi(poi_id));
        self.refresh_route();
        let leagues = self.target_leagues().unwrap_or(1);
        let direction = self.target_direction().unwrap_or("here");
        self.push_log(
            LogKind::Travel,
            format!("Marked {name} ({kind}, ~{leagues} leagues {direction})."),
        );
    }

    pub fn mark_nearest_settlement(&mut self) {
        let here = self.player.pos;
        let nearest = self
            .world
            .settlements
            .iter()
            .filter(|s| s.pos != here)
            .min_by(|a, b| {
                a.pos
                    .distance_to(here)
                    .partial_cmp(&b.pos.distance_to(here))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .or_else(|| self.world.nearest_settlement(here))
            .map(|s| s.id);
        if let Some(id) = nearest {
            let name = self.world.settlements[id].name.clone();
            self.target = Some(Target::Settlement(id));
            self.refresh_route();
            self.push_log(LogKind::Travel, format!("You mark {name} on your map."));
        }
    }

    // ------------------------------------------------------------------
    // Travel
    // ------------------------------------------------------------------

    pub fn move_player(&mut self, dx: i32, dy: i32, count: u32) {
        let phase_before = self.phase();
        let mut moved = 0usize;

        for _ in 0..count.max(1) {
            let next = self.player.pos.offset(dx, dy);
            if !self.world.is_passable(next) {
                if moved == 0 {
                    let terrain = self.world.terrain(next);
                    self.push_log(
                        LogKind::Info,
                        format!("The way is blocked by {}.", terrain.name()),
                    );
                }
                break;
            }
            self.player.pos = next;
            self.turn += 1;
            moved += 1;

            if let Some(poi) = self.world.poi_at(next) {
                let name = poi.name.clone();
                let cleared = poi.cleared;
                let has_contract = self
                    .contracts
                    .iter()
                    .any(|c| c.poi == poi.id && c.status == ContractStatus::Accepted);
                if has_contract {
                    self.push_log(
                        LogKind::Contract,
                        format!("You have reached {name}. Press Enter to confront the mark."),
                    );
                } else if cleared {
                    self.push_log(
                        LogKind::Travel,
                        format!("You pass the quiet ruins of {name}."),
                    );
                } else {
                    self.push_log(LogKind::Travel, format!("You arrive at {name}."));
                }
            } else if let Some(settlement) = self.world.settlement_at(next) {
                let name = settlement.name.clone();
                self.push_log(LogKind::Travel, format!("You arrive at {name}."));
            }
        }

        if moved > 0 {
            self.refresh_route();
            if let Some(message) = self.phase_change(phase_before) {
                self.push_log(LogKind::Travel, message.to_string());
            }
        }
    }

    pub fn rest(&mut self) {
        let Some(settlement) = self.current_settlement() else {
            self.push_log(
                LogKind::Info,
                "You need a roof and a hearth to rest properly.".to_string(),
            );
            return;
        };
        let name = settlement.name.clone();
        if self.player.hp >= self.player.max_hp {
            self.push_log(
                LogKind::Info,
                format!("You are already rested. {name} has nothing more to offer tonight."),
            );
            return;
        }
        let phase_before = self.phase();
        self.player.hp = self.player.max_hp;
        self.turn += 8;
        self.push_log(
            LogKind::Good,
            format!("You rest at {name}. Wounds close, and the road calls again."),
        );
        if let Some(message) = self.phase_change(phase_before) {
            self.push_log(LogKind::Travel, message.to_string());
        }
    }

    // ------------------------------------------------------------------
    // Interaction
    // ------------------------------------------------------------------

    pub fn interact(&mut self) {
        if self.current_settlement().is_some() {
            self.open_board();
            return;
        }

        if let Some(poi) = self.current_poi() {
            let poi_id = poi.id;
            let name = poi.name.clone();
            let detail = poi.detail.clone();
            let cleared = poi.cleared;
            let kind = poi.kind;
            let was_explored = poi.explored;

            if self.accepted_contract_for(poi_id).is_some() {
                self.mode = Mode::Resolve;
                self.push_log(
                    LogKind::Contract,
                    format!("You draw your sword at {name}."),
                );
                return;
            }

            if !was_explored {
                if let Some(poi) = self.world.pois.iter_mut().find(|p| p.id == poi_id) {
                    poi.explored = true;
                }
                if kind == PoiKind::Shrine {
                    let healed = 3.min(self.player.max_hp - self.player.hp);
                    self.player.hp += healed;
                    self.push_log(
                        LogKind::Good,
                        format!("You leave a coin at {name}. Warmth returns. (+{healed} HP)"),
                    );
                } else {
                    self.push_log(LogKind::Info, format!("You survey {name}. {detail}"));
                }
            } else if cleared {
                self.push_log(
                    LogKind::Info,
                    format!("Only claw marks and cold ash remain at {name}."),
                );
            } else {
                self.push_log(LogKind::Info, format!("{name}: {detail}"));
            }
            return;
        }

        self.push_log(
            LogKind::Info,
            "There is nothing here but wind, grass, and old stone.".to_string(),
        );
    }

    // ------------------------------------------------------------------
    // Contract board
    // ------------------------------------------------------------------

    pub fn open_board(&mut self) {
        if let Some(settlement) = self.current_settlement() {
            let name = settlement.name.clone();
            self.mode = Mode::Board;
            self.board_sel = 0;
            self.push_log(
                LogKind::Contract,
                format!("You read the contract board at {name}."),
            );
        } else {
            self.push_log(
                LogKind::Info,
                "You must stand in a settlement to read its contract board.".to_string(),
            );
        }
    }

    pub fn board_contracts(&self) -> Vec<usize> {
        let Some(settlement) = self.current_settlement() else {
            return Vec::new();
        };
        self.contracts
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.settlement == settlement.id && c.status == ContractStatus::Available
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn move_board_selection(&mut self, delta: i32) {
        let count = self.board_contracts().len();
        if count == 0 {
            return;
        }
        let next = self.board_sel as i32 + delta;
        self.board_sel = next.clamp(0, count as i32 - 1) as usize;
    }

    pub fn accept_board_selection(&mut self) {
        let contracts = self.board_contracts();
        let Some(&contract_id) = contracts.get(self.board_sel) else {
            self.push_log(LogKind::Info, "There is nothing to accept.".to_string());
            return;
        };

        self.contracts[contract_id].status = ContractStatus::Accepted;
        let poi_id = self.contracts[contract_id].poi;
        let title = self.contracts[contract_id].title.clone();
        let patron = self.contracts[contract_id].patron.clone();
        let reward = self.contracts[contract_id].reward;
        let danger = self.contracts[contract_id].danger;
        let poi_name = self.world.pois[poi_id].name.clone();

        self.target = Some(Target::Poi(poi_id));
        self.refresh_route();
        self.mode = Mode::Explore;
        self.push_log(
            LogKind::Contract,
            format!(
                "Contract accepted: {title} — {danger} danger, {reward} crowns, posted by {patron}."
            ),
        );
        self.push_log(
            LogKind::Travel,
            format!("The trail to {poi_name} is marked on your map."),
        );
    }

    pub fn accepted_contract_for(&self, poi_id: usize) -> Option<usize> {
        self.contracts
            .iter()
            .position(|c| c.poi == poi_id && c.status == ContractStatus::Accepted)
    }

    pub fn active_contracts(&self) -> Vec<usize> {
        self.contracts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.status == ContractStatus::Accepted)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn completed_count(&self) -> usize {
        self.contracts
            .iter()
            .filter(|c| c.status == ContractStatus::Completed)
            .count()
    }

    pub fn resolve_has_secret(&self) -> bool {
        self.current_poi()
            .and_then(|poi| self.accepted_contract_for(poi.id))
            .is_some_and(|i| self.contracts[i].secret.is_some())
    }

    pub fn resolve_contract_index(&self) -> Option<usize> {
        self.current_poi()
            .and_then(|poi| self.accepted_contract_for(poi.id))
    }

    // ------------------------------------------------------------------
    // Contract resolution
    // ------------------------------------------------------------------

    pub fn resolve(&mut self, choice: ResolveChoice) {
        let Some(contract_id) = self.resolve_contract_index() else {
            self.mode = Mode::Explore;
            return;
        };

        if choice == ResolveChoice::Leave {
            let name = self.world.pois[self.contracts[contract_id].poi].name.clone();
            self.push_log(LogKind::Info, format!("You back away from {name}."));
            self.mode = Mode::Explore;
            return;
        }

        let poi_id = self.contracts[contract_id].poi;
        let title = self.contracts[contract_id].title.clone();
        let monster = self.contracts[contract_id].monster.clone();
        let danger = self.contracts[contract_id].danger as i32;
        let reward = self.contracts[contract_id].reward;
        let patron = self.contracts[contract_id].patron.clone();
        let poi_name = self.world.pois[poi_id].name.clone();
        let secret = self.contracts[contract_id].secret.clone();

        if choice == ResolveChoice::Spare {
            let Some(secret) = secret else {
                self.push_log(
                    LogKind::Info,
                    "There is nothing here that wants to parley.".to_string(),
                );
                return;
            };
            let half = reward / 2;
            self.player.crowns += half;
            self.contracts[contract_id].status = ContractStatus::Completed;
            self.contracts[contract_id].outcome = Some("Spared".to_string());
            self.mark_poi_cleared(poi_id);
            self.clear_target();
            self.mode = Mode::Explore;
            self.push_log(LogKind::Good, format!("You lower your blade. {secret}"));
            self.push_log(
                LogKind::Contract,
                format!("You take {half} crowns for your silence. {poi_name} is quiet now."),
            );
            return;
        }

        // Fight.
        let roll = self.rng.range(1, 21) as i32;
        let steel = self.player.steel * 2;
        let dc = 5 + danger * 3;
        let total = roll + steel;

        if total >= dc {
            self.player.crowns += reward;
            self.contracts[contract_id].status = ContractStatus::Completed;
            self.contracts[contract_id].outcome = Some("Slain".to_string());
            self.mark_poi_cleared(poi_id);
            self.clear_target();
            self.mode = Mode::Explore;
            self.push_log(
                LogKind::Good,
                format!("You slay {monster} at {poi_name}. (d20 {roll} + steel {steel} vs {dc})"),
            );
            self.push_log(
                LogKind::Contract,
                format!("Contract complete: {title}. {reward} crowns paid by {patron}."),
            );

            if self.completed_count().is_multiple_of(3) {
                self.player.steel += 1;
                self.push_log(
                    LogKind::System,
                    "Your sword arm hardens. Steel +1.".to_string(),
                );
            }
        } else {
            let damage = danger + self.rng.range(0, 3) as i32;
            self.player.hp -= damage;
            self.push_log(
                LogKind::Bad,
                format!(
                    "{monster} gets inside your guard. You lose {damage} HP. \
                     (d20 {roll} + steel {steel} vs {dc})"
                ),
            );
            self.mode = Mode::Explore;
            if self.player.hp <= 0 {
                self.player.hp = 0;
                self.mode = Mode::Dead;
                self.push_log(
                    LogKind::Bad,
                    "Your blood soaks into the old ground. The road ends here.".to_string(),
                );
            }
        }
    }

    fn mark_poi_cleared(&mut self, poi_id: usize) {
        if let Some(poi) = self.world.pois.iter_mut().find(|p| p.id == poi_id) {
            poi.cleared = true;
            poi.explored = true;
        }
    }

    pub fn restart(&mut self) {
        let seed = { self.rng.next_u64() };
        *self = Game::new(seed);
    }

    // ------------------------------------------------------------------
    // Commands & persistence
    // ------------------------------------------------------------------

    pub fn execute_command(&mut self) {
        let raw = std::mem::take(&mut self.command);
        let raw = raw.trim();
        self.mode = Mode::Explore;
        if raw.is_empty() {
            return;
        }

        let mut parts = raw.split_whitespace();
        let command = parts.next().unwrap_or_default();
        match command {
            "q" | "quit" => self.should_quit = true,
            "w" | "write" => {
                let path = parts.next().unwrap_or(DEFAULT_SAVE);
                self.save(path);
            }
            "wq" | "x" => {
                let path = parts.next().unwrap_or(DEFAULT_SAVE);
                if self.save(path) {
                    self.should_quit = true;
                }
            }
            "e" | "edit" | "load" => {
                let path = parts.next().unwrap_or(DEFAULT_SAVE);
                match Game::load(path) {
                    Ok(mut loaded) => {
                        loaded.push_log(LogKind::System, format!("Loaded {path}."));
                        *self = loaded;
                    }
                    Err(error) => {
                        self.push_log(LogKind::Bad, format!("Could not load: {error}"));
                    }
                }
            }
            "new" | "seed" => {
                let seed = parts
                    .next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(random_seed);
                *self = Game::new(seed);
                self.push_log(
                    LogKind::System,
                    format!("A new world grows from seed {seed}."),
                );
            }
            "help" | "h" => self.mode = Mode::Help,
            "rest" => self.rest(),
            _ => self.push_log(
                LogKind::Bad,
                format!("Unknown command: {command}. Try :w :e :seed :new :help :q"),
            ),
        }
    }

    pub fn save(&mut self, path: &str) -> bool {
        let data = SaveGame {
            version: SAVE_VERSION,
            world: self.world.clone(),
            player: self.player.clone(),
            contracts: self.contracts.clone(),
            turn: self.turn,
            log: self.log.clone(),
            target: self.target,
            rng: self.rng.clone(),
        };

        let result = (|| -> std::io::Result<()> {
            let path = Path::new(path);
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            let bytes = serde_json::to_vec(&data).map_err(std::io::Error::other)?;
            let tmp = path.with_extension("tmp");
            fs::write(&tmp, bytes)?;
            fs::rename(&tmp, path)?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.push_log(LogKind::System, format!("Saved to {path}."));
                true
            }
            Err(error) => {
                self.push_log(LogKind::Bad, format!("Could not save: {error}"));
                false
            }
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let data: SaveGame = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
        if data.version != SAVE_VERSION {
            return Err(format!(
                "save version {} is not supported (expected {SAVE_VERSION})",
                data.version
            ));
        }
        if data.world.settlements.is_empty() {
            return Err("the save contains no settlements".to_string());
        }
        if !data.world.is_passable(data.player.pos) {
            return Err("the saved position is not on passable ground".to_string());
        }

        let target = match data.target {
            Some(Target::Poi(i)) if i >= data.world.pois.len() => None,
            Some(Target::Settlement(i)) if i >= data.world.settlements.len() => None,
            other => other,
        };

        let mut game = Self {
            world: data.world,
            player: data.player,
            contracts: data.contracts,
            turn: data.turn,
            log: data.log,
            target,
            route: None,
            mode: Mode::Explore,
            board_sel: 0,
            command: String::new(),
            show_labels: true,
            pending_g: false,
            count: None,
            should_quit: false,
            rng: data.rng,
        };
        game.refresh_route();
        Ok(game)
    }
}

#[derive(Serialize, Deserialize)]
struct SaveGame {
    version: u32,
    world: World,
    player: Player,
    contracts: Vec<Contract>,
    turn: u64,
    log: Vec<LogEntry>,
    target: Option<Target>,
    rng: Rng,
}

fn direction_name(dx: i32, dy: i32) -> &'static str {
    let ns = if dy < 0 {
        "N"
    } else if dy > 0 {
        "S"
    } else {
        ""
    };
    let ew = if dx < 0 {
        "W"
    } else if dx > 0 {
        "E"
    } else {
        ""
    };
    match (ns, ew) {
        ("N", "E") => "NE",
        ("N", "W") => "NW",
        ("S", "E") => "SE",
        ("S", "W") => "SW",
        ("N", "") => "N",
        ("S", "") => "S",
        ("", "E") => "E",
        ("", "W") => "W",
        _ => "here",
    }
}

fn generate_contracts(world: &World, rng: &mut Rng) -> Vec<Contract> {
    let mut poi_ids: Vec<usize> = (0..world.pois.len()).collect();
    rng.shuffle(&mut poi_ids);
    let mut cursor = 0usize;
    let mut contracts = Vec::new();

    for settlement in &world.settlements {
        let count = rng.range(2, 4);
        for _ in 0..count {
            if cursor >= poi_ids.len() {
                break;
            }
            let poi = &world.pois[poi_ids[cursor]];
            cursor += 1;

            let role = rng.choose(PATRON_ROLES);
            let patron = format!("the {role} of {}", settlement.name);
            let victim = rng.choose(VICTIMS);
            let reward = 35 + poi.danger as u32 * 30 + rng.range(0, 41);
            let brief = format!(
                "Locals say {} has been preying on {} near {}. {patron} offers {reward} crowns for proof of its end.",
                poi.monster, victim, poi.name
            );
            let secret = if rng.chance(0.6) {
                Some(rng.choose(SECRETS).to_string())
            } else {
                None
            };

            contracts.push(Contract {
                id: contracts.len(),
                settlement: settlement.id,
                poi: poi.id,
                title: poi.name.clone(),
                patron,
                monster: poi.monster.clone(),
                danger: poi.danger,
                reward,
                status: ContractStatus::Available,
                brief,
                secret,
                outcome: None,
            });
        }
    }

    contracts
}

pub fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x5EED)
}

const PATRON_ROLES: &[&str] = &[
    "reeve",
    "elder",
    "priest",
    "innkeeper",
    "merchant",
    "headman",
    "smith",
];

const VICTIMS: &[&str] = &[
    "sheep",
    "fishermen",
    "travellers",
    "children",
    "cattle",
    "woodcutters",
    "gravediggers",
];

const SECRETS: &[&str] = &[
    "The beast is a villager under a curse. The patron knows.",
    "The creature is guarding something buried beneath the ruins.",
    "The patron's own son was seen walking into the nest.",
    "It only kills those who drink from the mill pond.",
    "A sorceress bound the spirit here, and the binding is failing.",
    "The contract is a trap. The patron wants a witness silenced.",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_has_contracts_and_log() {
        let game = Game::new(999);
        assert!(!game.contracts.is_empty());
        assert!(!game.log.is_empty());
        assert!(game.player.hp > 0);
        assert_eq!(game.mode, Mode::Explore);
    }

    #[test]
    fn save_round_trip() {
        let mut game = Game::new(4242);
        game.move_player(1, 0, 3);
        game.turn = 12;
        let path = std::env::temp_dir().join(format!("wayfarer_test_{}.json", std::process::id()));
        let path_str = path.to_string_lossy().to_string();
        assert!(game.save(&path_str));

        let loaded = Game::load(&path_str).expect("load should succeed");
        assert_eq!(loaded.world.seed, game.world.seed);
        assert_eq!(loaded.player.pos, game.player.pos);
        assert_eq!(loaded.turn, game.turn);
        assert_eq!(loaded.contracts.len(), game.contracts.len());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn target_cycles_through_pois() {
        let mut game = Game::new(7);
        let count = game.world.pois.len();
        game.cycle_target(true, 1);
        assert!(matches!(game.target, Some(Target::Poi(0))));
        for _ in 0..count {
            game.cycle_target(true, 1);
        }
        assert!(matches!(game.target, Some(Target::Poi(i)) if i == 0));
    }
}
