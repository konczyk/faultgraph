use crate::analysis::groups::{Group, GroupSet};
use crate::graph::edge::{Edge, EdgeId};
use crate::graph::graph::Graph;
use crate::graph::node::{Node, NodeId};
use crate::scenario::scenario::Scenario;
use crate::simulation::modifiers::CapacityModifier;
use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;
use crate::state::snapshot::Snapshot;

pub struct StressScenario {
    entry: Vec<NodeId>,
    base_load: f64,
    ramp_per_turn: f64,
    max_load: f64,
}

impl StressScenario {
    pub fn build() -> (Graph, GroupSet, Snapshot, Box<dyn Scenario>) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let mut nid = 0;
        let mut eid = 0;

        let ingress: Vec<NodeId> = (0..20)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("api-{}", nid), 300.0, 1.8));
                nid += 1;
                id
            })
            .collect();

        let gateways: Vec<NodeId> = (0..30)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("gw-{}", nid), 250.0, 1.4));
                nid += 1;
                id
            })
            .collect();

        let auth: Vec<NodeId> = (0..20)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("auth-{}", nid), 120.0, 1.0));
                nid += 1;
                id
            })
            .collect();

        let cache_l1: Vec<NodeId> = (0..30)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("cache-l1-{}", nid), 600.0, 0.8));
                nid += 1;
                id
            })
            .collect();

        let cache_l2: Vec<NodeId> = (0..30)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("cache-l2-{}", nid), 800.0, 0.6));
                nid += 1;
                id
            })
            .collect();

        let services: Vec<NodeId> = (0..40)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("svc-{}", nid), 180.0, 1.2));
                nid += 1;
                id
            })
            .collect();

        let workers: Vec<NodeId> = (0..20)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("worker-{}", nid), 140.0, 1.1));
                nid += 1;
                id
            })
            .collect();

        let dbs: Vec<NodeId> = (0..20)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("db-{}", nid), 90.0, 0.0));
                nid += 1;
                id
            })
            .collect();

        for i in &ingress {
            for g in gateways.iter().take(6) {
                edges.push(Edge::new(EdgeId(eid), *i, *g, 1.0));
                eid += 1;
            }
        }

        for g in &gateways {
            for a in auth.iter().take(4) {
                edges.push(Edge::new(EdgeId(eid), *g, *a, 1.0));
                eid += 1;
            }
        }

        for a in &auth {
            for c in cache_l1.iter().take(5) {
                edges.push(Edge::new(EdgeId(eid), *a, *c, 3.0));
                eid += 1;
            }
        }

        for c1 in &cache_l1 {
            for c2 in cache_l2.iter().take(3) {
                edges.push(Edge::new(EdgeId(eid), *c1, *c2, 0.9));
                eid += 1;
            }
        }

        for c in &cache_l2 {
            for s in services.iter().take(2) {
                edges.push(Edge::new(EdgeId(eid), *c, *s, 0.8));
                eid += 1;
            }
        }

        for s in &services {
            for w in workers.iter().take(2) {
                edges.push(Edge::new(EdgeId(eid), *s, *w, 1.1));
                eid += 1;
            }
        }

        for w in &workers {
            for d in dbs.iter().take(3) {
                edges.push(Edge::new(EdgeId(eid), *w, *d, 1.0));
                eid += 1;
            }
        }

        let graph = Graph::new(nodes, edges);

        let groups = GroupSet::new(vec![
            Group::new("Ingress".into(), ingress.clone()),
            Group::new("Gateways".into(), gateways.clone()),
            Group::new("Auth".into(), auth.clone()),
            Group::new("Cache L1".into(), cache_l1.clone()),
            Group::new("Cache L2".into(), cache_l2.clone()),
            Group::new("Services".into(), services.clone()),
            Group::new("Workers".into(), workers.clone()),
            Group::new("Database".into(), dbs.clone()),
        ]);

        let node_states = graph
            .nodes()
            .iter()
            .map(|_| NodeState::new(0.0, 0.0, 0.0, 1.0))
            .collect();

        let edge_states = graph
            .edges()
            .iter()
            .map(|_| EdgeState::new(true))
            .collect();

        let capacity_mods = groups
            .groups()
            .iter()
            .map(|_| CapacityModifier::new())
            .collect();

        let snapshot = Snapshot::new(0, node_states, edge_states, capacity_mods);

        let scenario = StressScenario {
            entry: ingress,
            base_load: 120.0,
            ramp_per_turn: 6.0,
            max_load: 5_000.0,
        };

        (graph, groups, snapshot, Box::new(scenario))
    }
}

impl Scenario for StressScenario {
    fn load(&self, node_id: NodeId, turn: usize) -> f64 {
        if !self.entry.contains(&node_id) {
            return 0.0;
        }

        let base = self.base_load + self.ramp_per_turn * turn as f64;
        let wave = (turn % 20) as f64;
        let oscillation = if wave < 10.0 { wave } else { 20.0 - wave };
        let spike = if turn % 30 <= 2 { 80.0 } else { 0.0 };

        (base + oscillation * 4.0 + spike).min(self.max_load)
    }

    fn entry_nodes(&self) -> &[NodeId] {
        &self.entry
    }

    fn ops_per_turn(&self) -> u8 {
        1
    }
}
