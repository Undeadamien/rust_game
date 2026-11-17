#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East,
    North,
    South,
    West,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Terrain {
    City,
    Forest,
    Ruin,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::East => Self::West,
            Self::West => Self::East,
            Self::North => Self::South,
            Self::South => Self::North,
        }
    }
}

fn direction(from: &Node, to: &Node) -> Option<Direction> {
    use std::cmp::Ordering;
    match (to.x.cmp(&from.x), to.y.cmp(&from.y)) {
        (Ordering::Greater, Ordering::Equal) => Some(Direction::East),
        (Ordering::Less, Ordering::Equal) => Some(Direction::West),
        (Ordering::Equal, Ordering::Greater) => Some(Direction::South),
        (Ordering::Equal, Ordering::Less) => Some(Direction::North),
        _ => None, // diagonal or same node
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    counter: u16,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub id: u16, //todo: should be made private
    pub name: String,
    pub terrain: Terrain,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Edge {
    nodes: (u16, u16), //note: Tuple or HashSet?
}

impl Map {
    pub fn new() -> Self {
        Self {
            counter: 0,
            nodes: vec![],
            edges: vec![],
        }
    }
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn node(&self, id: u16) -> &Node {
        &self.nodes[id as usize]
    }

    pub fn add_node(&mut self, name: String, terrain: Terrain, x: u16, y: u16) -> u16 {
        let id = self.counter;
        self.counter += 1;
        self.nodes.push(Node {
            id,
            name,
            terrain,
            x,
            y,
        });
        id
    }

    pub fn add_edge(&mut self, a: u16, b: u16) {
        let Some(dir) = direction(self.node(a), self.node(b)) else {
            return; // reject diagonals
        };
        if self.neighbor(a, dir).is_some() || self.neighbor(b, dir.opposite()).is_some() {
            return; // one neighbor per direction
        }
        let edge = Edge { nodes: (a, b) };
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }

    pub fn neighbor(&self, id: u16, dir: Direction) -> Option<u16> {
        self.edges.iter().find_map(|e| {
            let (a, b) = e.nodes();
            match (a == id, b == id) {
                (true, _) if direction(self.node(a), self.node(b)) == Some(dir) => Some(b),
                (_, true) if direction(self.node(b), self.node(a)) == Some(dir) => Some(a),
                _ => None,
            }
        })
    }
}
impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    pub fn new(id: u16, name: String, terrain: Terrain, x: u16, y: u16) -> Self {
        Self {
            id,
            name,
            terrain,
            x,
            y,
        }
    }
}

impl Edge {
    pub fn new(nodes: (u16, u16)) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> (u16, u16) {
        self.nodes
    }
}
