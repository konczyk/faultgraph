use crate::analysis::groups::GroupSet;
use crate::graph::edge::EdgeId;
use crate::graph::graph::Graph;
use crate::graph::node::NodeId;
use crate::scenario::scenario::Scenario;
use crate::simulation::modifiers::Throttle;
use crate::state::snapshot::Snapshot;
use std::mem;

pub struct SimulationEngine {
    graph: Graph,
    groups: GroupSet,
    previous_snapshot: Option<Snapshot>,
    current_snapshot: Snapshot,
    scenario: Box<dyn Scenario>,
    remaining_ops: u8,
    throttles: Vec<Throttle>,
    node_to_group: Vec<usize>,
}

impl SimulationEngine {
    pub fn new(
        graph: Graph,
        groups: GroupSet,
        initial_snapshot: Snapshot,
        scenario: Box<dyn Scenario>,
    ) -> Self {
        let remaining_ops = scenario.ops_per_turn();
        let groups_cnt = groups.groups().len();
        let node_to_group = graph
            .nodes()
            .iter()
            .enumerate()
            .map(|(n_id, _)| {
                groups
                    .groups()
                    .iter()
                    .enumerate()
                    .find_map(|(g_id, group)| {
                        group
                            .nodes()
                            .iter()
                            .find(|n| n.index() == n_id)
                            .map(|_| g_id)
                    })
                    .unwrap()
            })
            .collect();
        Self {
            graph,
            groups,
            previous_snapshot: None,
            current_snapshot: initial_snapshot,
            scenario,
            remaining_ops,
            throttles: vec![Throttle::new(); groups_cnt],
            node_to_group,
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn groups(&self) -> &GroupSet {
        &self.groups
    }

    pub fn scenario(&self) -> &Box<dyn Scenario> {
        &self.scenario
    }

    pub fn remaining_ops(&self) -> u8 {
        self.remaining_ops
    }

    pub fn group_by_node_id(&self, node_id: usize) -> usize {
        self.node_to_group[node_id]
    }

    pub fn step(&mut self) {
        let states = self.current_snapshot.node_states();
        let mut prop = vec![0.0; self.graph.node_count()];

        self.current_snapshot
            .edge_states()
            .iter()
            .enumerate()
            .filter(|(_, state)| state.is_enabled())
            .map(|(i, _)| (i, self.graph.edge_by_id(EdgeId(i))))
            .filter(|(_, e)| states[e.from().index()].is_healthy())
            .for_each(|(_, e)| {
                let f_id = e.from().index();
                let t_id = e.to().index();
                let throttle = self.throttles[self.node_to_group[f_id]].factor();
                prop[t_id] += states[f_id].load() * e.multiplier() * throttle;
            });

        self.scenario.entry_nodes().iter().for_each(|id| {
            prop[id.index()] += self.scenario.load(*id, self.current_snapshot.turn())
        });

        let mut new_node_states = self.current_snapshot.node_states().clone();
        new_node_states.iter_mut().enumerate().for_each(|(i, n)| {
            n.inject_load(prop[i]);
            let utilization = n.load() / self.graph.node_by_id(NodeId(i)).capacity();
            let k = 0.1;
            if utilization > 1.0 {
                let damage = k * (utilization - 1.0);
                n.set_health(n.health() - damage);
            }
        });

        let turn = self.current_snapshot.turn() + 1;
        let new_edge_states = self.current_snapshot.edge_states().clone();

        let old_snapshot = mem::replace(
            &mut self.current_snapshot,
            Snapshot::new(turn, new_node_states, new_edge_states),
        );

        self.previous_snapshot = Some(old_snapshot);

        self.remaining_ops = self.scenario.ops_per_turn();
        self.throttles.iter_mut().for_each(|t| t.deactivate());
    }

    pub fn current_snapshot(&self) -> &Snapshot {
        &self.current_snapshot
    }

    pub fn previous_snapshot(&self) -> &Snapshot {
        self.previous_snapshot
            .as_ref()
            .unwrap_or(&self.current_snapshot)
    }

    pub fn try_throttle_group(&mut self, group_id: usize) -> bool {
        if self.remaining_ops == 0 {
            false
        } else {
            self.remaining_ops -= 1;
            self.throttles[group_id].apply(0.5);
            true
        }
    }

    pub fn throttle(&self, group_id: usize) -> &Throttle {
        &self.throttles[group_id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::groups::Group;
    use crate::graph::edge::Edge;
    use crate::graph::node::Node;
    use crate::state::edge_state::EdgeState;
    use crate::state::node_state::NodeState;
    use approx::assert_relative_eq;

    pub struct TestScenario {
        entry: Vec<NodeId>,
        base_load: f64,
    }

    impl TestScenario {
        pub fn new(entry: Vec<NodeId>) -> Self {
            Self {
                entry,
                base_load: 10.0,
            }
        }
    }

    impl Scenario for TestScenario {
        fn load(&self, node_id: NodeId, turn: usize) -> f64 {
            if self.entry.contains(&node_id) {
                self.base_load * (turn + 1) as f64
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

    fn snapshot(graph: &Graph) -> Snapshot {
        Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .map(|_| NodeState::new(0.0, 1.0))
                .collect(),
            graph.edges().iter().map(|_| EdgeState::new(true)).collect(),
        )
    }

    #[test]
    fn test_one_hop_propagation() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph);
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)])),
        );

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].load());
        assert_relative_eq!(0.0, node_states[1].load());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(10.0, node_states[0].load());
        assert_relative_eq!(0.0, node_states[1].load());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(30.0, node_states[0].load());
        assert_relative_eq!(10.0, node_states[1].load());
    }

    #[test]
    fn test_edge_multiplier_correctness() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 2.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph);
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)])),
        );
        engine.step();
        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(30.0, node_states[0].load());
        assert_relative_eq!(20.0, node_states[1].load());
    }

    #[test]
    fn test_disabled_edge_block_propagation() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 2.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .map(|_| NodeState::new(0.0, 1.0))
                .collect(),
            graph
                .edges()
                .iter()
                .map(|_| EdgeState::new(false))
                .collect(),
        );
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)])),
        );
        engine.step();
        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(30.0, node_states[0].load());
        assert_relative_eq!(0.0, node_states[1].load());
    }

    #[test]
    fn test_unhealthy_nodes_do_not_propagate_load() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 2.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .enumerate()
                .map(|(i, _)| NodeState::new(0.0, if i == 0 { 0.0 } else { 1.0 }))
                .collect(),
            graph.edges().iter().map(|_| EdgeState::new(true)).collect(),
        );
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)])),
        );
        engine.step();
        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(30.0, node_states[0].load());
        assert_relative_eq!(0.0, node_states[1].load());
    }
}
