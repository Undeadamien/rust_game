use crate::game::effect::{Effect, PlayerChange};

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub actions: Vec<Action>,

    pub gold: i16,
    pub health: i16, //todo: allow negative health?
}

#[derive(Debug)]
pub struct Action {
    pub name: String,
    pub effects: Vec<Effect>,
}

impl Player {
    pub fn new(name: String) -> Self {
        assert!(!name.is_empty());
        Self {
            name,
            actions: vec![],
            gold: 0,
            health: 3,
        }
    }
    pub fn apply(&mut self, change: PlayerChange) {
        match change {
            //todo: replace the right sides with methods
            PlayerChange::Gold(n) => self.gold += n,
            PlayerChange::Health(n) => self.health += n,
        };
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;
}
