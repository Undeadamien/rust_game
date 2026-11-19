pub mod card;
pub mod combat;
pub mod effect;
pub mod player;

use crate::game::player::Player;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Phase {
    Travel,
    Encounter,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub player: Player,
}

impl Game {
    //todo: find a better way to init
    pub fn new(player: Player) -> Self {
        Self { player }
    }
    pub fn update(&mut self) {
        //
    }
}
