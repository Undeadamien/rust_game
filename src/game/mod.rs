pub mod deck;
pub mod die;
pub mod effect;
pub mod encounter;
pub mod map;
pub mod player;

use crate::game::die::DicePool;
use crate::game::map::{Map, Node};
use crate::game::player::Player;

pub enum Command {
    Travel(Node),
}

#[derive(Debug, Copy, Clone)]
pub enum Phase {
    Travel,
    Encounter,
}

pub struct Game {
    pub player: Player,
    pub phase: Phase,
    pub dice: DicePool,

    pub map: Map,
}

impl Game {
    //todo: find a better way to init
    pub fn new(player: Player, phase: Phase, dice: DicePool, map: Map) -> Self {
        Self {
            player,
            phase,
            dice,
            map,
        }
    }
    pub fn update(&mut self, command: Command) {
        match (self.phase, command) {
            (Phase::Travel, Command::Travel(_)) => todo!(),
            _ => todo!(),
        }
    }
}

/*
A
├───┐
B   └────┐
         C
*/
