#[derive(Clone, Copy, PartialEq)]
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
    /// gain >= 0.0
    gain: f64,
}

impl Node {
    pub fn new(id: NodeId, name: String, capacity: f64, gain: f64) -> Self {
        Self {
            id,
            name: name.into(),
            capacity,
            gain,
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name.as_str()
    }

    pub fn capacity(&self) -> f64 {
        self.capacity
    }

    pub fn gain(&self) -> f64 {
        self.gain
    }
}
