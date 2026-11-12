#![allow(dead_code)]

use rust_game::game::die::{Die, Face};
use rust_game::game::map::{Edge, Map, Node};
use rust_game::game::player::{Action, Effect::*, Player};

fn main() {
    // debug_player();
    // debug_map();
    // debug_die();
}

fn debug_player() {
    let mut player = Player::new(String::from("Hello world"));
    let action = Action {
        name: String::from("TestAction"),
        effects: vec![ChangeGold(10), ChangeGold(10), ChangeHealth(-10)],
    };
    println!("{}", player.gold);
    for effect in action.effects {
        println!("{:?}", effect);
        match effect {
            ChangeGold(value) => player.gold += value,
            _ => println!("effect not applied"),
        }
    }
    println!("{}", player.gold);
}

fn debug_map() {
    let mut map = Map::new();
    map.add_edge(Edge::new((
        Node::new(String::from("hello")),
        Node::new(String::from("world")),
    )));
    map.add_edge(Edge::new((
        Node::new(String::from("world")),
        Node::new(String::from("hello")),
    )));
    map.add_edge(Edge::new((
        Node::new(String::from("world")),
        Node::new(String::from("hello")),
    )));
    map.add_edge(Edge::new((
        Node::new(String::from("WORLD")),
        Node::new(String::from("hello")),
    )));

    for node in map.nodes() {
        println!("{:?}", node);
    }
    for edge in map.edges() {
        println!("{:?}", edge);
    }
}

fn debug_die() {
    let dice = Die::new(vec![
        Face::Attack,
        Face::Attack,
        Face::Attack,
        Face::Defense,
        Face::Defense,
        Face::Blank,
    ]);

    println!("Faces: {:?}", dice.faces());
    println!("Frequences: {:?}", dice.frequences());
    println!("Frequence: Attack: {:?}", dice.frequences()[&Face::Attack]);
    for n in 0..=10 {
        println!("Roll:  {:0>3} {:?}", n, dice.roll());
    }
}
