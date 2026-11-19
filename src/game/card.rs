use rand::seq::SliceRandom;

pub struct Card {
    pub cost: u16,
}
impl Card {
    pub fn new(cost: u16) -> Self {
        Self { cost }
    }
}

pub struct Deck {
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
}
impl Deck {
    pub fn new() -> Self {
        Self {
            draw_pile: vec![],
            discard_pile: vec![],
        }
    }
    pub fn shuffle(&mut self) {
        self.draw_pile.shuffle(&mut rand::rng()); //todo: replace with `make_rng` for a seeded generator
    }
    pub fn draw(&mut self) -> Option<Card> {
        if self.draw_pile.is_empty() {
            if self.discard_pile.is_empty() {
                return None;
            }
            self.draw_pile.append(&mut self.discard_pile);
            self.shuffle();
        }
        self.draw_pile.pop()
    }
    pub fn discard(&mut self, card: Card) {
        self.discard_pile.push(card);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_deck() {
        let mut deck = Deck::new();
        assert!(deck.draw().is_none());
        assert!(!deck.draw().is_some());
    }
}
