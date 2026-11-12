use rand::seq::SliceRandom;

pub struct Deck<T> {
    pub draw_pile: Vec<T>,
    pub discard_pile: Vec<T>,
}
impl<T> Deck<T> {
    pub fn new() -> Self {
        Self {
            draw_pile: vec![],
            discard_pile: vec![],
        }
    }
    pub fn shuffle(&mut self) {
        self.draw_pile.shuffle(&mut rand::rng());
    }

    pub fn draw(&mut self) -> Option<T> {
        if self.draw_pile.is_empty() {
            if self.discard_pile.is_empty() {
                return None;
            }
            self.draw_pile.append(&mut self.discard_pile);
            self.shuffle();
        }
        self.draw_pile.pop()
    }

    pub fn discard(&mut self, card: T) {
        self.discard_pile.push(card);
    }
}

impl<T> Default for Deck<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty_deck() {
        let mut deck = Deck::<u16>::new();
        assert!(deck.draw().is_none());
        assert!(!deck.draw().is_some());
    }

    #[test]
    fn test_draw() {
        let mut deck = Deck::<u16>::new();
        deck.draw_pile = vec![1];
        assert!(deck.draw_pile == vec![1]);
        assert!(deck.discard_pile == vec![]);
        match deck.draw() {
            Some(card) => deck.discard(card),
            None => {}
        }
        assert!(deck.discard_pile == vec![1]);
        assert!(deck.draw_pile == vec![]);
    }
}
