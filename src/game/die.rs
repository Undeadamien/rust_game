use itertools::Itertools;
use rand::seq::IteratorRandom;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Face {
    Sword,
    Shield,
    Magic,
    Blank,
}

#[derive(Debug, Clone)]
pub struct Die {
    faces: Vec<Face>,
}

impl Die {
    pub fn new(faces: Vec<Face>) -> Self {
        assert!(faces.iter().len() >= 2);
        Self { faces }
    }
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
    pub fn roll(&self) -> Face {
        //todo: replace with `make_rng` for a seeded generator
        self.faces.iter().choose(&mut rand::rng()).cloned().unwrap()
    }
    pub fn frequences(&self) -> HashMap<Face, usize> {
        self.faces.iter().cloned().counts()
    }
}

#[derive(Debug, Clone)]
pub struct DicePool {
    dice: Vec<Die>,
}

impl DicePool {
    pub fn new(dice: Vec<Die>) -> Self {
        Self { dice }
    }
    pub fn dice(&self) -> &[Die] {
        &self.dice
    }
    // note: rename for `roll_all`?
    pub fn roll(&self) -> Vec<Face> {
        self.dice.iter().map(|d| d.roll()).collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_empty_die() {
        Die::new(vec![]);
    }

    #[test]
    #[should_panic]
    fn test_one_sided_die() {
        Die::new(vec![Face::Sword]);
    }

    #[test]
    fn test_frequences() {
        let die = Die::new(vec![Face::Sword, Face::Sword, Face::Shield, Face::Blank]);
        let frequences = die.frequences();
        let expected = HashMap::from([(Face::Sword, 2), (Face::Shield, 1), (Face::Blank, 1)]);
        assert_eq!(expected, frequences)
    }
}
