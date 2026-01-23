use crate::graph::edge::EdgeId;
use crate::graph::graph::Graph;
use crate::scenario::scenario::Scenario;
use crate::state::snapshot::Snapshot;

pub struct SimulationEngine {
    graph: Graph,
    current_snapshot: Snapshot,
    scenario: Box<dyn Scenario>,
}

impl SimulationEngine {
    pub fn new(graph: Graph, initial_snapshot: Snapshot, scenario: Box<dyn Scenario>) -> Self {
        Self { graph, current_snapshot: initial_snapshot, scenario }
    }

    pub fn step(&mut self) {
        let states = self.current_snapshot.node_states();
        let mut prop = vec![0.0; self.graph.node_count()];

        self.current_snapshot.edge_states().iter().enumerate()
            .filter(|(_, state)| state.is_enabled())
            .map(|(i, _)| (i, self.graph.edge_by_id(EdgeId(i))))
            .filter(|(_, e)| states[e.from().index()].is_healthy())
            .for_each(|(_, e)| {
                prop[e.to().index()] += states[e.from().index()].load() * e.multiplier();
            });

        self.scenario.entry_nodes().iter().for_each(|id| {
            prop[id.index()] += self.scenario.load(*id, self.current_snapshot.turn())
        });

        let mut new_node_states = self.current_snapshot.node_states().clone();
        new_node_states.iter_mut().enumerate().for_each(|(i, n)| {
            n.inject_load(prop[i]);
        });

        self.current_snapshot = Snapshot::new(
            self.current_snapshot.turn() + 1,
            new_node_states,
            self.current_snapshot.edge_states().clone(),
        );
    }

    pub fn current_snapshot(&self) -> &Snapshot {
        &self.current_snapshot
    }
}