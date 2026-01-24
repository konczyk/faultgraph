use crate::graph::node::NodeId;

pub struct Group {
    name: String,
    nodes: Vec<NodeId>,
}

impl Group {
    pub fn new(name: String, nodes: Vec<NodeId>) -> Self {
        Self { name, nodes }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }
}

pub struct GroupSet {
    groups: Vec<Group>,
}

impl GroupSet {
    pub fn new(groups: Vec<Group>) -> Self {
        Self { groups }
    }

    pub fn groups(&self) -> &[Group] {
        &self.groups
    }
}

pub enum GroupTrend {
    Up,
    Down,
    Flat,
}

pub enum GroupRisk {
    Low,
    Medium,
    High,
    Critical,
}

pub struct GroupSummary {
    name: String,
    avg_utilization: f64,
    trend: GroupTrend,
    node_count: usize,
    worst_health: f64,
    risk: GroupRisk,
}

impl GroupSummary {
    pub fn new(
        name: String,
        avg_utilization: f64,
        trend: GroupTrend,
        node_count: usize,
        worst_health: f64,
        risk: GroupRisk,
    ) -> Self {
        Self {
            name,
            avg_utilization,
            trend,
            node_count,
            worst_health,
            risk,
        }
    }
}
