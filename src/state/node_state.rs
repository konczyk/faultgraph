pub struct NodeState {
    /// load >= 0.0
    load: f64,
    /// health [0.0, 1.0]
    health: f64,
}

impl NodeState {
    pub fn new(load: f64, health: f64) -> Self {
        Self { load, health }
    }
}

