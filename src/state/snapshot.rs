use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;

pub struct Snapshot {
    turn: usize,
    node_states: Vec<NodeState>,
    edge_states: Vec<EdgeState>
}

impl Snapshot {
    pub fn new(turn: usize, node_states: Vec<NodeState>, edge_states: Vec<EdgeState>) -> Self {
        Self { turn, node_states, edge_states }
    }

    pub fn turn(&self) -> usize {
        self.turn
    }

    pub fn node_states(&self) -> &Vec<NodeState> {
        &self.node_states
    }

    pub fn edge_states(&self) -> &Vec<EdgeState> {
        &self.edge_states
    }

}