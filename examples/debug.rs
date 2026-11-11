#![allow(dead_code)]

use rust_game::game::dice::{Die, Face};

fn main() {}

fn debug_die() {
    let dice = Die::new(vec![
        Face::Attack,
        Face::Attack,
        Face::Attack,
        Face::Defense,
        Face::Defense,
        Face::Empty,
    ]);

    println!("Faces: {:?}", dice.faces());
    println!("Frequences: {:?}", dice.frequences());
    println!("Frequence: Attack: {:?}", dice.frequences()[&Face::Attack]);
    for n in 0..=10 {
        println!("Roll:  {:0>3} {:?}", n, dice.roll());
    }
}
