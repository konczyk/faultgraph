use crate::graph::edge::{Edge, EdgeId};
use crate::graph::node::{Node, NodeId};

pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    adj: Vec<Vec<EdgeId>>,
    sinks: Vec<NodeId>,
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut adj: Vec<Vec<EdgeId>> = vec![Vec::new(); nodes.len()];
        edges.iter().for_each(|e| {
            adj[e.from().index()].push(e.id());
        });
        let sinks = nodes
            .iter()
            .filter(|n| adj[n.id().index()].is_empty())
            .map(|n| *n.id())
            .collect::<Vec<NodeId>>();
        Self {
            nodes,
            edges,
            adj,
            sinks,
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
        &self.adj[id.index()]
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn sinks(&self) -> &[NodeId] {
        &self.sinks
    }
}
