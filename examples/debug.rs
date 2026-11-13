#![allow(dead_code)]

use grid::*;
use rust_game::game::die::{DicePool, Die, Face};
use rust_game::game::effect::Effect;
use rust_game::game::effect::PlayerChange::{Gold, Health};
use rust_game::game::encounter::{Check, Encounter, Kind};
use rust_game::game::map::{Edge, Map, Node};
use rust_game::game::player::{Action, Player};
use rust_game::game::{Game, Phase};

fn main() {
    let mut grid: Grid<char> = Grid::from_vec(vec![' '; 16], 4);
    grid.fill('·');
    grid[(3, 3)] = 'X';
    println! {"{:?}", grid}

    for x in 0..grid.rows() {
        println! {"{:?}", grid.iter_row(x).collect::<String>()}
    }
}

fn debug_game() {
    let player = Player::new(String::from("The Hero"));
    let map = Map::new();
    let die: Die = Die::new(vec![
        Face::Sword,
        Face::Sword,
        Face::Sword,
        Face::Shield,
        Face::Shield,
        Face::Blank,
    ]);
    let dice = DicePool::new(vec![die; 4]);

    let _game = Game::new(player, Phase::Travel, dice, map);
}

fn debug_encounter() {
    let encounter = Encounter {
        name: String::from("Goblin Attack"),
        description: String::from("A group of goblin is attacking you, defend youself!"),
        kind: Kind::Combat,
        checks: vec![
            Check {
                kind: Face::Sword,
                difficulty: 3,
                on_success: vec![Effect::Player(Gold(2))],
                on_failure: vec![],
            },
            Check {
                kind: Face::Shield,
                difficulty: 2,
                on_success: vec![],
                on_failure: vec![Effect::Player(Health(2))],
            },
        ],
    };

    let die: Die = Die::new(vec![
        Face::Sword,
        Face::Sword,
        Face::Sword,
        Face::Shield,
        Face::Shield,
        Face::Blank,
    ]);
    let dice = DicePool::new(vec![die; 4]);
    let roll = dice.roll();

    for check in encounter.checks {
        if check.resolve(&roll) {
            println!(
                "Check passed with:\n{:?}->{:?}\n{:?}\n",
                check.kind, check.difficulty, roll
            )
        } else {
            println!(
                "Check not passed with:\n{:?}->{:?}\n{:?}\n",
                check.kind, check.difficulty, roll
            )
        }
    }
}

fn debug_player() {
    let mut player = Player::new(String::from("Hello world"));
    let action = Action {
        name: String::from("TestAction"),
        effects: vec![
            Effect::Player(Gold(10)),
            Effect::Player(Gold(10)),
            Effect::Player(Health(-10)),
        ],
    };
    println!("{}", player.gold);
    for effect in action.effects {
        println!("{:?}", effect);
        match effect {
            Effect::Player(change) => player.apply(change),
            _ => todo!(),
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
    let die = Die::new(vec![
        Face::Sword,
        Face::Sword,
        Face::Sword,
        Face::Shield,
        Face::Shield,
        Face::Blank,
    ]);

    println!("Faces: {:?}", die.faces());
    println!("Frequences: {:?}", die.frequences());
    println!("Frequence: Attack: {:?}", die.frequences()[&Face::Sword]);
    for n in 0..=10 {
        println!("Roll:  {:0>3} {:?}", n, die.roll());
    }

    //todo: debug Dice
}
