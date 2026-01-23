use crate::graph::graph::Graph;
use crate::scenario::scenario::Scenario;
use crate::state::node_state::NodeState;
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
        self.current_snapshot = Snapshot::new(
            self.current_snapshot.turn() + 1,
            self.current_snapshot.node_states().clone(),
            self.current_snapshot.edge_states().clone(),
        );

        self.current_snapshot.reset_loads();
        self.scenario.entry_nodes().iter().for_each(|id| {
            self.current_snapshot.inject_load(*id, self.scenario.load(*id, self.current_snapshot.turn() - 1));
        });
    }

    pub fn current_snapshot(&self) -> &Snapshot {
        &self.current_snapshot
    }
}