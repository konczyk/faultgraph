use crate::analysis::groups::GroupSet;
use crate::graph::graph::Graph;
use crate::graph::node::NodeId;
use crate::scenario::scenario::Scenario;
use crate::state::snapshot::Snapshot;
use std::mem;

pub struct SimulationEngine {
    graph: Graph,
    groups: GroupSet,
    previous_snapshot: Option<Snapshot>,
    current_snapshot: Snapshot,
    scenario: Box<dyn Scenario>,
    remaining_ops: u8,
}

impl SimulationEngine {
    pub fn new(
        graph: Graph,
        groups: GroupSet,
        initial_snapshot: Snapshot,
        scenario: Box<dyn Scenario>,
    ) -> Self {
        let remaining_ops = scenario.ops_per_turn();
        Self {
            graph,
            groups,
            previous_snapshot: None,
            current_snapshot: initial_snapshot,
            scenario,
            remaining_ops,
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

    pub fn step(&mut self) {
        self.current_snapshot.tick();
        let node_states = self.current_snapshot.node_states();
        let edge_states = self.current_snapshot.edge_states();
        let mut prop = vec![0.0; self.graph.node_count()];

        node_states
            .iter()
            .enumerate()
            .map(|(n_id, _)| self.graph.node_by_id(NodeId(n_id)))
            .filter(|n| node_states[n.id().index()].is_healthy())
            .for_each(|n| {
                self.graph
                    .outgoing(*n.id())
                    .iter()
                    .map(|e_id| self.graph.edge_by_id(*e_id))
                    .for_each(|e| {
                        let t_id = e.to().index();
                        prop[t_id] += self.current_snapshot.edge_load(e.id(), self.graph());
                    })
            });

        self.scenario.entry_nodes().iter().for_each(|id| {
            prop[id.index()] += self.scenario.load(*id, self.current_snapshot.turn())
        });

        let mut new_node_states = node_states.clone();
        new_node_states.iter_mut().enumerate().for_each(|(i, n)| {
            n.set_demand(prop[i]);
            if !n.is_healthy() {
                n.set_served(0.0);
                n.set_backlog(0.0);
                return;
            }

            let throttle = self
                .current_snapshot
                .capacity_mod(self.groups.group_by_node_id(i))
                .factor();
            let capacity = self.graph.node_by_id(NodeId(i)).capacity() * throttle;
            let outgoing_edges = self.graph.outgoing(NodeId(i));
            let total = prop[i] + n.backlog();

            n.set_served(capacity.min(total));

            let has_active_edge = outgoing_edges
                .iter()
                .find(|e_id| edge_states[e_id.index()].is_enabled())
                .is_some();

            if outgoing_edges.len() > 0 && !has_active_edge {
                n.set_backlog(total);
            } else {
                n.set_backlog(total - n.served());
            }

            if capacity == 0.0 {
                return;
            }
            let pressure = total / capacity;
            let k = 0.1;
            if pressure > 1.0 {
                let damage = k * (pressure - 1.0);
                n.set_health(n.health() - damage);
            } else if pressure < 1.0 && n.backlog() == 0.0 {
                n.set_health(n.health() + 0.01);
            }
        });

        let turn = self.current_snapshot.turn() + 1;
        let new_edge_states = edge_states.clone();
        let new_capacity_mods = self.current_snapshot.capacity_mods().clone();

        let old_snapshot = mem::replace(
            &mut self.current_snapshot,
            Snapshot::new(turn, new_node_states, new_edge_states, new_capacity_mods),
        );

        self.previous_snapshot = Some(old_snapshot);
        self.remaining_ops = self.scenario.ops_per_turn();
    }

    pub fn current_snapshot(&self) -> &Snapshot {
        &self.current_snapshot
    }

    pub fn previous_snapshot(&self) -> &Snapshot {
        self.previous_snapshot
            .as_ref()
            .unwrap_or(&self.current_snapshot)
    }

    fn try_capacity_modifier(&mut self, group_id: usize, factor: f64) {
        if self.remaining_ops > 0 && self.current_snapshot.update_capacity(group_id, factor) {
            self.remaining_ops -= 1;
        }
    }

    pub fn try_throttle_group(&mut self, group_id: usize) {
        self.try_capacity_modifier(group_id, 0.5);
    }

    pub fn try_boost_group(&mut self, group_id: usize) {
        self.try_capacity_modifier(group_id, 1.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::groups::Group;
    use crate::graph::edge::{Edge, EdgeId};
    use crate::graph::node::Node;
    use crate::simulation::modifiers::CapacityModifier;
    use crate::state::edge_state::EdgeState;
    use crate::state::node_state::NodeState;
    use approx::assert_relative_eq;

    pub struct TestScenario {
        entry: Vec<NodeId>,
        loads: Vec<f64>,
    }

    impl TestScenario {
        pub fn new(entry: Vec<NodeId>, loads: Vec<f64>) -> Self {
            Self { entry, loads }
        }
    }

    impl Scenario for TestScenario {
        fn load(&self, node_id: NodeId, turn: usize) -> f64 {
            if self.entry.contains(&node_id) {
                self.loads[turn]
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

    fn snapshot(graph: &Graph, group_cnt: usize) -> Snapshot {
        Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .map(|_| NodeState::new(0.0, 0.0, 0.0, 1.0))
                .collect(),
            graph.edges().iter().map(|_| EdgeState::new(true)).collect(),
            vec![CapacityModifier::new(); group_cnt],
        )
    }

    #[test]
    fn test_one_hop_propagation() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph, 1);
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![10.0, 20.0, 30.0])),
        );

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(10.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(10.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(20.0, node_states[0].demand());
        assert_relative_eq!(10.0, node_states[1].demand());

        assert_relative_eq!(20.0, node_states[0].served());
        assert_relative_eq!(10.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());
    }

    #[test]
    fn test_edge_multiplier_correctness() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 2.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph, 1);
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![10.0, 20.0, 30.0])),
        );
        engine.step();
        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(20.0, node_states[0].served());
        assert_relative_eq!(20.0, node_states[1].served());
    }

    #[test]
    fn test_disabled_edge_block_propagation() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 2.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .map(|_| NodeState::new(0.0, 0.0, 0.0, 1.0))
                .collect(),
            graph
                .edges()
                .iter()
                .map(|_| EdgeState::new(false))
                .collect(),
            vec![CapacityModifier::new(); 1],
        );
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![10.0, 20.0, 30.0])),
        );
        engine.step();
        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(10.0, node_states[0].served());
        assert_relative_eq!(10.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].served());
        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(30.0, node_states[0].served());
        assert_relative_eq!(30.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].served());
    }

    #[test]
    fn test_unhealthy_nodes_do_not_propagate_load() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 60.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 2.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .enumerate()
                .map(|(i, _)| NodeState::new(0.0, 0.0, 0.0, if i == 0 { 0.0 } else { 1.0 }))
                .collect(),
            graph.edges().iter().map(|_| EdgeState::new(true)).collect(),
            vec![CapacityModifier::new(); 1],
        );
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![10.0, 20.0, 30.0])),
        );

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(10.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(20.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());
    }

    #[test]
    fn test_backlog_accumulates_when_over_capacity() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 40.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph, 1);
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![50.0, 50.0, 50.0])),
        );

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(50.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(50.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(50.0, node_states[0].demand());
        assert_relative_eq!(50.0, node_states[1].demand());

        assert_relative_eq!(50.0, node_states[0].served());
        assert_relative_eq!(40.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(10.0, node_states[1].backlog());
    }

    #[test]
    fn test_backlog_drains_when_below_capacity() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 40.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph, 1);
        let groups = GroupSet::new(vec![Group::new(
            "group1".to_string(),
            vec![NodeId(0), NodeId(1)],
        )]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![50.0, 20.0, 10.0])),
        );

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(50.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(50.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(20.0, node_states[0].demand());
        assert_relative_eq!(50.0, node_states[1].demand());

        assert_relative_eq!(20.0, node_states[0].served());
        assert_relative_eq!(40.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(10.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(10.0, node_states[0].demand());
        assert_relative_eq!(20.0, node_states[1].demand());

        assert_relative_eq!(10.0, node_states[0].served());
        assert_relative_eq!(30.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());
    }

    #[test]
    fn test_throttle() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 40.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph, 2);
        let groups = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0)]),
            Group::new("group2".to_string(), vec![NodeId(1)]),
        ]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![100.0, 80.0, 20.0])),
        );
        engine.try_throttle_group(0);

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(100.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(50.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(50.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(80.0, node_states[0].demand());
        assert_relative_eq!(50.0, node_states[1].demand());

        assert_relative_eq!(50.0, node_states[0].served());
        assert_relative_eq!(40.0, node_states[1].served());

        assert_relative_eq!(80.0, node_states[0].backlog());
        assert_relative_eq!(10.0, node_states[1].backlog());
    }

    #[test]
    fn test_boost() {
        let api = Node::new(NodeId(0), "api".to_string(), 100.0, 1.0);
        let db = Node::new(NodeId(1), "db".to_string(), 40.0, 1.0);
        let link = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let graph = Graph::new(vec![api, db], vec![link]);
        let initial_snapshot = snapshot(&graph, 2);
        let groups = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0)]),
            Group::new("group2".to_string(), vec![NodeId(1)]),
        ]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(vec![NodeId(0)], vec![200.0, 110.0, 50.0])),
        );
        engine.try_boost_group(0);

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(0.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(200.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());

        assert_relative_eq!(150.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());

        assert_relative_eq!(50.0, node_states[0].backlog());
        assert_relative_eq!(0.0, node_states[1].backlog());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(110.0, node_states[0].demand());
        assert_relative_eq!(150.0, node_states[1].demand());

        assert_relative_eq!(150.0, node_states[0].served());
        assert_relative_eq!(40.0, node_states[1].served());

        assert_relative_eq!(10.0, node_states[0].backlog());
        assert_relative_eq!(110.0, node_states[1].backlog());
    }

    #[test]
    fn test_load_splitting() {
        let api1 = Node::new(NodeId(0), "api1".to_string(), 100.0, 1.0);
        let api2 = Node::new(NodeId(1), "api2".to_string(), 100.0, 1.0);
        let db1 = Node::new(NodeId(2), "db1".to_string(), 40.0, 1.0);
        let db2 = Node::new(NodeId(3), "db2".to_string(), 40.0, 1.0);
        let db3 = Node::new(NodeId(4), "db3".to_string(), 40.0, 1.0);

        let link1 = Edge::new(EdgeId(0), NodeId(0), NodeId(2), 1.0);
        let link2 = Edge::new(EdgeId(1), NodeId(0), NodeId(3), 3.0);
        let link3 = Edge::new(EdgeId(2), NodeId(0), NodeId(4), 5.0);

        let link4 = Edge::new(EdgeId(3), NodeId(1), NodeId(2), 1.0);
        let link5 = Edge::new(EdgeId(4), NodeId(1), NodeId(3), 1.0);

        let graph = Graph::new(
            vec![api1, api2, db1, db2, db3],
            vec![link1, link2, link3, link4, link5],
        );
        let initial_snapshot = Snapshot::new(
            0,
            graph
                .nodes()
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    NodeState::new(0.0, 0.0, 0.0, if i == 1 || i == 3 { 0.0 } else { 1.0 })
                })
                .collect(),
            graph
                .edges()
                .iter()
                .enumerate()
                .map(|(i, _)| EdgeState::new(i != 2))
                .collect(),
            vec![CapacityModifier::new(); 2],
        );
        let groups = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0), NodeId(1)]),
            Group::new("group2".to_string(), vec![NodeId(2), NodeId(3), NodeId(4)]),
        ]);

        let mut engine = SimulationEngine::new(
            graph,
            groups,
            initial_snapshot,
            Box::new(TestScenario::new(
                vec![NodeId(0), NodeId(1)],
                vec![10.0, 20.0, 30.0],
            )),
        );

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(0.0, node_states[0].demand());
        assert_relative_eq!(0.0, node_states[1].demand());
        assert_relative_eq!(0.0, node_states[2].demand());
        assert_relative_eq!(0.0, node_states[3].demand());
        assert_relative_eq!(0.0, node_states[4].demand());

        assert_relative_eq!(0.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());
        assert_relative_eq!(0.0, node_states[2].served());
        assert_relative_eq!(0.0, node_states[3].served());
        assert_relative_eq!(0.0, node_states[4].served());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(10.0, node_states[0].demand());
        assert_relative_eq!(10.0, node_states[1].demand());
        assert_relative_eq!(0.0, node_states[2].demand());
        assert_relative_eq!(0.0, node_states[3].demand());
        assert_relative_eq!(0.0, node_states[4].demand());

        assert_relative_eq!(10.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());
        assert_relative_eq!(0.0, node_states[2].served());
        assert_relative_eq!(0.0, node_states[3].served());
        assert_relative_eq!(0.0, node_states[4].served());

        engine.step();

        let node_states = engine.current_snapshot.node_states();
        assert_relative_eq!(20.0, node_states[0].demand());
        assert_relative_eq!(20.0, node_states[1].demand());
        assert_relative_eq!(2.5, node_states[2].demand());
        assert_relative_eq!(7.5, node_states[3].demand());
        assert_relative_eq!(0.0, node_states[4].demand());

        assert_relative_eq!(20.0, node_states[0].served());
        assert_relative_eq!(0.0, node_states[1].served());
        assert_relative_eq!(2.5, node_states[2].served());
        assert_relative_eq!(0.0, node_states[3].served());
        assert_relative_eq!(0.0, node_states[4].served());
    }
}
