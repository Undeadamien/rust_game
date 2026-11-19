pub mod card;
pub mod effect;
pub mod map;
pub mod player;

use crate::game::map::{Direction, Map};
use crate::game::player::Player;

pub enum Command {
    Travel(Direction),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Phase {
    Travel,
    Encounter,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub player: Player,
    pub phase: Phase,
    pub map: Map,
    pub current: u16,
}

impl Game {
    //todo: find a better way to init
    pub fn new(player: Player, phase: Phase, map: Map, current: u16) -> Self {
        Self {
            player,
            phase,
            map,
            current,
        }
    }
    pub fn update(&mut self, command: Command) {
        let Command::Travel(dir) = command;
        if self.phase == Phase::Travel {
            if let Some(next) = self.map.neighbor(self.current, dir) {
                self.current = next;
            }
        }
    }
}
