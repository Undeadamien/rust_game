use crate::game::effect::{Effect, PlayerChange};

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub actions: Vec<Action>,

    pub gold: i16,
    pub health: u16, //todo: allow negative health?
    pub health_max: u16,
}

#[derive(Debug, Clone)]
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
            health_max: 3,
        }
    }
    pub fn apply(&mut self, change: PlayerChange) {
        match change {
            //todo: replace the right sides with methods
            PlayerChange::Gold(n) => self.gold += n,
            PlayerChange::Health(n) => self.health = self.health.saturating_sub(n as u16),
        };
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;
}
