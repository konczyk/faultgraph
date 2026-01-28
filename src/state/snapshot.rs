use crate::simulation::modifiers::CapacityModifier;
use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;

pub struct Snapshot {
    turn: usize,
    node_states: Vec<NodeState>,
    edge_states: Vec<EdgeState>,
    capacity_mods: Vec<CapacityModifier>,
}

impl Snapshot {
    pub fn new(
        turn: usize,
        node_states: Vec<NodeState>,
        edge_states: Vec<EdgeState>,
        capacity_mods: Vec<CapacityModifier>,
    ) -> Self {
        Self {
            turn,
            node_states,
            edge_states,
            capacity_mods,
        }
    }

    pub fn tick(&mut self) {
        self.capacity_mods.iter_mut().for_each(|m| m.tick())
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

    pub fn capacity_mods(&self) -> &Vec<CapacityModifier> {
        &self.capacity_mods
    }

    pub fn capacity_mod(&self, group_id: usize) -> &CapacityModifier {
        &self.capacity_mods[group_id]
    }

    pub fn update_capacity(&mut self, group_id: usize, factor: f64) -> bool {
        self.capacity_mods[group_id].apply(factor)
    }
}
