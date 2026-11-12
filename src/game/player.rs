#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub actions: Vec<Action>,

    pub gold: i16,
}

#[derive(Debug)]
pub struct Action {
    pub name: String,
    pub effects: Vec<Effect>,
}

#[derive(Debug)]
pub enum Effect {
    //todo: add more
    ChangeGold(i16),
    ChangeHealth(i16),
}

impl Player {
    pub fn new(name: String) -> Self {
        assert!(!name.is_empty());
        Self {
            name,
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
