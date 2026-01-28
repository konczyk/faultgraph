use crate::graph::edge::EdgeId;
use crate::graph::graph::Graph;
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

    pub fn edge_load(&self, edged_id: EdgeId, graph: &Graph) -> f64 {
        let edge = graph.edge_by_id(edged_id);
        let f_id = edge.from().index();
        if !self.node_states[f_id].is_healthy()
            || !self.edge_states[edged_id.index()].is_enabled()
            || self.node_states[f_id].served() == 0.0
        {
            return 0.0;
        }

        let total_weight = graph
            .outgoing(edge.from())
            .iter()
            .filter(|e_id| self.edge_states[e_id.index()].is_enabled())
            .map(|e_id| graph.edge_by_id(*e_id).weight())
            .sum::<f64>();

        if total_weight == 0.0 {
            return 0.0;
        }

        let node = graph.node_by_id(edge.from());
        let served = self.node_states[f_id].served();
        let total_demand = served * node.gain();
        total_demand * (edge.weight() / total_weight)
    }
}
