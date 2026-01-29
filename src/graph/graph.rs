use crate::graph::edge::{Edge, EdgeId};
use crate::graph::node::{Node, NodeId};

pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    outgoing: Vec<Vec<EdgeId>>,
    incoming: Vec<Vec<EdgeId>>,
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut outgoing: Vec<Vec<EdgeId>> = vec![Vec::new(); nodes.len()];
        let mut incoming: Vec<Vec<EdgeId>> = vec![Vec::new(); nodes.len()];
        edges.iter().for_each(|e| {
            outgoing[e.from().index()].push(e.id());
            incoming[e.to().index()].push(e.id());
        });
        Self {
            nodes,
            edges,
            outgoing,
            incoming,
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn node_by_id(&self, id: NodeId) -> &Node {
        &self.nodes[id.index()]
    }

    pub fn edge_by_id(&self, id: EdgeId) -> &Edge {
        &self.edges[id.index()]
    }

    pub fn outgoing(&self, id: NodeId) -> &[EdgeId] {
        &self.outgoing[id.index()]
    }

    pub fn incoming(&self, id: NodeId) -> &[EdgeId] {
        &self.incoming[id.index()]
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
