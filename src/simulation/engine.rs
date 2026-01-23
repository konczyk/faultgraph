use crate::graph::graph::Graph;
use crate::state::snapshot::Snapshot;

pub struct SimulationEngine {
    graph: Graph,
    current_snapshot: Snapshot,
}

impl SimulationEngine {
    pub fn new(graph: Graph, initial_snapshot: Snapshot) -> Self {
        Self { graph, current_snapshot: initial_snapshot }
    }

    pub fn step(&mut self) {
        self.current_snapshot = Snapshot::new(
            self.current_snapshot.turn() + 1,
            self.current_snapshot.node_states().clone(),
            self.current_snapshot.edge_states().clone(),
        )
    }

    pub fn current_snapshot(&self) -> &Snapshot {
        &self.current_snapshot
    }
}