use crate::analysis::groups::{Group, GroupSet};
use crate::graph::edge::{Edge, EdgeId};
use crate::graph::graph::Graph;
use crate::graph::node::{Node, NodeId};
use crate::scenario::scenario::Scenario;
use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;
use crate::state::snapshot::Snapshot;

pub struct BasicScenario {
    entry: Vec<NodeId>,
    base_load: f64,
    ramp_per_turn: f64,
    max_load: f64,
}

impl BasicScenario {
    pub fn build() -> (Graph, GroupSet, Snapshot, Box<dyn Scenario>) {
        let nodes = vec![
            Node::new(NodeId(0), "api-gateway".into(), 120.0, 1.0),
            Node::new(NodeId(1), "auth-service".into(), 80.0, 1.0),
            Node::new(NodeId(2), "orders-service".into(), 90.0, 1.1),
            Node::new(NodeId(3), "payments-service".into(), 70.0, 1.2),
            Node::new(NodeId(4), "redis-cache".into(), 60.0, 1.4),
            Node::new(NodeId(5), "postgres-primary".into(), 100.0, 0.7),
            Node::new(NodeId(6), "postgres-replica".into(), 100.0, 0.5),
        ];

        let edges = vec![
            Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0),
            Edge::new(EdgeId(1), NodeId(0), NodeId(2), 1.0),
            Edge::new(EdgeId(2), NodeId(2), NodeId(3), 2.0),
            Edge::new(EdgeId(3), NodeId(1), NodeId(4), 1.2),
            Edge::new(EdgeId(4), NodeId(2), NodeId(5), 1.4),
            Edge::new(EdgeId(5), NodeId(5), NodeId(6), 0.6),
        ];

        let graph = Graph::new(nodes, edges);

        let groups = GroupSet::new(vec![
            Group::new("Ingress".into(), vec![NodeId(0)]),
            Group::new(
                "Core Services".into(),
                vec![NodeId(1), NodeId(2), NodeId(3)],
            ),
            Group::new("Cache".into(), vec![NodeId(4)]),
            Group::new("Database".into(), vec![NodeId(5), NodeId(6)]),
        ]);

        let node_states = graph
            .nodes()
            .iter()
            .map(|_| NodeState::new(0.0, 0.0, 0.0, 1.0))
            .collect();

        let edge_states = graph.edges().iter().map(|_| EdgeState::new(true)).collect();

        let snapshot = Snapshot::new(0, node_states, edge_states);

        let scenario = BasicScenario {
            entry: vec![NodeId(0)],
            base_load: 15.0,
            ramp_per_turn: 3.0,
            max_load: 250.0,
        };

        (graph, groups, snapshot, Box::new(scenario))
    }
}

impl Scenario for BasicScenario {
    fn load(&self, node_id: NodeId, turn: usize) -> f64 {
        if self.entry.contains(&node_id) {
            let load = self.base_load + self.ramp_per_turn * turn as f64;
            load.min(self.max_load)
        } else {
            0.0
        }
    }

    fn entry_nodes(&self) -> &[NodeId] {
        &self.entry
    }

    fn ops_per_turn(&self) -> u8 {
        1
    }
}
