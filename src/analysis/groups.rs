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

#[derive(Debug, PartialEq)]
pub enum GroupTrend {
    Up,
    Down,
    Flat,
}

#[derive(Debug, PartialEq)]
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn avg_utilization(&self) -> f64 {
        self.avg_utilization
    }

    pub fn trend(&self) -> &GroupTrend {
        &self.trend
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }

    pub fn worst_health(&self) -> f64 {
        self.worst_health
    }

    pub fn risk(&self) -> &GroupRisk {
        &self.risk
    }
}
