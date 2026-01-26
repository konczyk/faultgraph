use crate::graph::node::NodeId;
use std::fmt::{Display, Formatter};

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
pub enum GroupHealth {
    Ok,
    Degraded,
    Critical,
    Failed,
}

impl Display for GroupHealth {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GroupHealth::Ok => "Ok",
            GroupHealth::Degraded => "Degraded",
            GroupHealth::Critical => "Critical",
            GroupHealth::Failed => "Failed",
        };
        f.pad(s)
    }
}

pub struct GroupSummary {
    name: String,
    avg_utilization: f64,
    utilization_trend: GroupTrend,
    node_count: usize,
    raw_health: f64,
    health: GroupHealth,
    health_trend: GroupTrend,
}

impl GroupSummary {
    pub fn new(
        name: String,
        avg_utilization: f64,
        utilization_trend: GroupTrend,
        node_count: usize,
        raw_health: f64,
        health: GroupHealth,
        health_trend: GroupTrend,
    ) -> Self {
        Self {
            name,
            avg_utilization,
            utilization_trend,
            node_count,
            raw_health,
            health,
            health_trend,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn avg_utilization(&self) -> f64 {
        self.avg_utilization
    }

    pub fn utilization_trend(&self) -> &GroupTrend {
        &self.utilization_trend
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }

    pub fn raw_health(&self) -> f64 {
        self.raw_health
    }

    pub fn health(&self) -> &GroupHealth {
        &self.health
    }

    pub fn health_trend(&self) -> &GroupTrend {
        &self.health_trend
    }
}
