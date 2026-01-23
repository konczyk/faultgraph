#[derive(Clone, Copy)]
pub struct NodeId(pub usize);

impl NodeId {
    pub fn index(self) -> usize {
        self.0
    }
}

pub struct Node {
    id: NodeId,
    name: String,
    /// capacity > 0.0
    capacity: f64,
}

impl Node {
    pub fn new(id: NodeId, name: impl Into<String>, capacity: f64) -> Self {
        Self {
            id,
            name: name.into(),
            capacity
        }
    }
}