use crate::graph::node::NodeId;

pub trait Scenario {
    fn load(&self, node_id: NodeId, turn: usize) -> f64;
    fn entry_nodes(&self) -> &[NodeId];
    fn ops_per_turn(&self) -> u8;
}
