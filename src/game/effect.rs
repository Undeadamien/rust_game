#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Player(PlayerChange),
    Group(GroupChange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerChange {
    Gold(i16),
    Health(i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupChange {
    Health(i16),
}
