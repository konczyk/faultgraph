#[derive(Clone, Copy)]
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

    pub fn load(&self) -> f64 {
        self.load
    }

    pub fn health(&self) -> f64 {
        self.health
    }

    pub fn inject_load(&mut self, load: f64) {
        self.load += load;
    }

    pub fn set_health(&mut self, health: f64) {
        self.health = health.max(0.0)
    }

    pub fn is_healthy(&self) -> bool {
        self.health > 0.0
    }

}

