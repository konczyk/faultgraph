#[derive(Clone)]
pub struct NodeState {
    /// demand >= 0.0
    demand: f64,
    /// 0.0 <= server <= node capacity
    served: f64,
    /// backlog >= 0.0
    backlog: f64,
    /// health [0.0, 1.0]
    health: f64,
}

impl NodeState {
    pub fn new(demand: f64, served: f64, backlog: f64, health: f64) -> Self {
        Self {
            demand,
            served,
            backlog,
            health,
        }
    }

    pub fn demand(&self) -> f64 {
        self.demand
    }

    pub fn served(&self) -> f64 {
        self.served
    }

    pub fn backlog(&self) -> f64 {
        self.backlog
    }

    pub fn health(&self) -> f64 {
        self.health
    }

    pub fn set_demand(&mut self, load: f64) {
        self.demand = load;
    }

    pub fn set_served(&mut self, load: f64) {
        self.served = load;
    }

    pub fn set_backlog(&mut self, load: f64) {
        self.backlog = load.max(0.0);
    }

    pub fn set_health(&mut self, health: f64) {
        self.health = health.clamp(0.0, 1.0)
    }

    pub fn is_healthy(&self) -> bool {
        self.health > 0.0
    }
}
