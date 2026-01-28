use crate::analysis::groups::{Group, GroupSet};
use crate::graph::edge::{Edge, EdgeId};
use crate::graph::graph::Graph;
use crate::graph::node::{Node, NodeId};
use crate::scenario::scenario::Scenario;
use crate::simulation::modifiers::CapacityModifier;
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
            Node::new(NodeId(0), "api-1".into(), 200.0, 1.8),
            Node::new(NodeId(1), "api-2".into(), 200.0, 1.6),
            Node::new(NodeId(2), "auth".into(), 80.0, 1.0),
            Node::new(NodeId(3), "orders-1".into(), 100.0, 1.2),
            Node::new(NodeId(4), "orders-2".into(), 100.0, 1.2),
            Node::new(NodeId(5), "cache-1".into(), 300.0, 0.7),
            Node::new(NodeId(6), "cache-2".into(), 300.0, 0.7),
            Node::new(NodeId(7), "cache-3".into(), 300.0, 0.7),
            Node::new(NodeId(8), "cache-4".into(), 300.0, 0.7),
            Node::new(NodeId(9), "db-1".into(), 60.0, 0.0),
            Node::new(NodeId(10), "db-2".into(), 60.0, 0.0),
            Node::new(NodeId(11), "db-3".into(), 60.0, 0.0),
        ];

        let mut edges = Vec::new();
        let mut eid = 0;

        for api in [0, 1] {
            edges.push(Edge::new(EdgeId(eid), NodeId(api), NodeId(2), 1.0));
            eid += 1;
        }

        for api in [0, 1] {
            for orders in [3, 4] {
                edges.push(Edge::new(EdgeId(eid), NodeId(api), NodeId(orders), 1.0));
                eid += 1;
            }
        }

        for api in [0, 1] {
            for cache in [5, 6, 7, 8] {
                edges.push(Edge::new(EdgeId(eid), NodeId(api), NodeId(cache), 4.0));
                eid += 1;
            }
        }

        for cache in [5, 6, 7, 8] {
            edges.push(Edge::new(EdgeId(eid), NodeId(cache), NodeId(9), 1.0));
            eid += 1;
            edges.push(Edge::new(EdgeId(eid), NodeId(cache), NodeId(10), 1.0));
            eid += 1;
            edges.push(Edge::new(EdgeId(eid), NodeId(cache), NodeId(11), 0.8));
            eid += 1;
        }

        for orders in [3, 4] {
            edges.push(Edge::new(EdgeId(eid), NodeId(orders), NodeId(9), 1.0));
            eid += 1;
            edges.push(Edge::new(EdgeId(eid), NodeId(orders), NodeId(10), 1.0));
            eid += 1;
            edges.push(Edge::new(EdgeId(eid), NodeId(orders), NodeId(11), 1.0));
            eid += 1;
        }

        let graph = Graph::new(nodes, edges);

        let groups = GroupSet::new(vec![
            Group::new("Ingress".into(), vec![NodeId(0), NodeId(1)]),
            Group::new("Auth".into(), vec![NodeId(2)]),
            Group::new("Orders".into(), vec![NodeId(3), NodeId(4)]),
            Group::new(
                "Cache".into(),
                vec![NodeId(5), NodeId(6), NodeId(7), NodeId(8)],
            ),
            Group::new("Database".into(), vec![NodeId(9), NodeId(10), NodeId(11)]),
        ]);

        let node_states = graph
            .nodes()
            .iter()
            .map(|_| NodeState::new(0.0, 0.0, 0.0, 1.0))
            .collect();

        let edge_states = graph.edges().iter().map(|_| EdgeState::new(true)).collect();

        let capacity_mods = groups
            .groups()
            .iter()
            .map(|_| CapacityModifier::new())
            .collect();

        let snapshot = Snapshot::new(0, node_states, edge_states, capacity_mods);

        let scenario = BasicScenario {
            entry: vec![NodeId(0), NodeId(1)],
            base_load: 20.0,
            ramp_per_turn: 5.0,
            max_load: 400.0,
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
