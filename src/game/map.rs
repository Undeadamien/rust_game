#[derive(Debug, Clone)]
pub struct Map {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Edge {
    nodes: (Node, Node), //note: Tuple or HashSet?
}

impl Map {
    pub fn new() -> Self {
        Self {
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

    pub fn add_node(&mut self, node: Node) {
        if self.nodes.contains(&node) {
            return;
        }
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        if self.edges.contains(&edge) {
            return;
        }
        let (node_a, node_b) = edge.nodes();

        if !self.nodes.contains(&node_a) {
            self.nodes.push(node_a);
        }
        if !self.nodes.contains(&node_b) {
            self.nodes.push(node_b);
        }

        self.edges.push(edge);
    }
}

impl Node {
    pub fn new(name: String) -> Self {
        Self { name: name }
    }
}

impl Edge {
    pub fn new(nodes: (Node, Node)) -> Self {
        Self { nodes: nodes }
    }

    pub fn nodes(&self) -> (Node, Node) {
        self.nodes.clone()
    }
}
