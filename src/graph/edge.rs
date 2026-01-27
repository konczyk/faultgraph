use crate::graph::node::NodeId;

#[derive(Clone, Copy)]
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
    /// weight >= 0.0
    weight: f64,
}

impl Edge {
    pub fn new(id: EdgeId, from: NodeId, to: NodeId, weight: f64) -> Self {
        Self {
            id,
            from,
            to,
            weight,
        }
    }

    pub fn id(&self) -> EdgeId {
        self.id
    }

    pub fn from(&self) -> NodeId {
        self.from
    }

    pub fn to(&self) -> NodeId {
        self.to
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }
}
