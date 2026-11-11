#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub actions: Vec<Action>,

    pub gold: i16,
}

#[derive(Debug)]
pub struct Action {
    pub name: String,
    pub effect: Effect,
}

#[derive(Debug)]
pub enum Effect {
    TakeGold(u16),
    TakeDamage(u16),
}

impl Player {
    pub fn new(name: String) -> Self {
        assert!(!name.is_empty());
        Self {
            name: name,
            actions: vec![],
            gold: 0,
        }
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;
}
