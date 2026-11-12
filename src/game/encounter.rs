use crate::game::die::{self, Face};
use crate::game::effect;

pub enum Kind {
    Combat,
    Social,
}

pub struct Encounter {
    pub name: String,
    pub description: String,

    pub kind: Kind,
    pub checks: Vec<Check>,
}

pub struct Check {
    pub kind: die::Face,
    pub difficulty: u8,
    pub on_success: Vec<effect::Effect>,
    pub on_failure: Vec<effect::Effect>,
}

impl Check {
    pub fn resolve(&self, dice: &[Face]) -> bool {
        let rolled = dice.iter().filter(|f| **f == self.kind).count();
        rolled >= self.difficulty.into()
    }
}
