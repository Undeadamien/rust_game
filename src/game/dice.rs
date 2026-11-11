use itertools::Itertools;
use rand::seq::IteratorRandom;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Face {
    Attack,
    Defense,
    Empty,
}

#[derive(Debug, Clone)]
pub struct Die {
    faces: Vec<Face>,
}

impl Die {
    pub fn new(faces: Vec<Face>) -> Self {
        assert!(faces.iter().len() >= 2);
        Self { faces: faces }
    }
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
    pub fn roll(&self) -> Face {
        self.faces.iter().choose(&mut rand::rng()).cloned().unwrap()
    }
    pub fn frequences(&self) -> HashMap<Face, usize> {
        self.faces.iter().cloned().counts()
    }
}

pub struct DiePool {
    dices: Vec<Die>,
}

impl DiePool {
    pub fn new(dices: Vec<Die>) -> Self {
        Self { dices: dices }
    }
    pub fn dices(&self) -> &[Die] {
        &self.dices
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
        Die::new(vec![Face::Attack]);
    }

    #[test]
    fn test_frequences() {
        let die = Die::new(vec![Face::Attack, Face::Attack, Face::Defense, Face::Empty]);
        let frequences = die.frequences();
        let expected = HashMap::from([(Face::Attack, 2), (Face::Defense, 1), (Face::Empty, 1)]);
        assert_eq!(expected, frequences)
    }
}
