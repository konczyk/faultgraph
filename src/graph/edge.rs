use crate::graph::node::NodeId;

pub struct EdgeId(pub usize);

impl EdgeId {
    pub fn index(self) -> usize {
        self.0
    }
}

pub struct Edge {
    id: EdgeId,
    from: NodeId,
    to: NodeId,
    /// multiplier >= 0.0
    multiplier: f64,
}

impl Edge {
    pub fn new(id: EdgeId, from: NodeId, to: NodeId, multiplier: f64) -> Self {
        Self { id, from, to, multiplier }
    }
}