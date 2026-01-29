use crate::analysis::groups::{Group, GroupSet};
use crate::graph::edge::{Edge, EdgeId};
use crate::graph::graph::Graph;
use crate::graph::node::{Node, NodeId};
use crate::scenario::scenario::Scenario;
use crate::simulation::modifiers::CapacityModifier;
use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;
use crate::state::snapshot::Snapshot;
use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct RandomStressScenario {
    entry: Vec<NodeId>,
    base_load: f64,
    ramp_per_turn: f64,
    max_load: f64,
}

impl RandomStressScenario {
    pub fn build(seed: u64) -> (Graph, GroupSet, Snapshot, Box<dyn Scenario>) {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut nid = 0usize;
        let mut eid = 0usize;

        let sizes = [40.0, 80.0, 160.0];

        let make_nodes = |nodes: &mut Vec<Node>,
                          nid: &mut usize,
                          count: usize,
                          prefix: &str,
                          gain: f64|
         -> Vec<NodeId> {
            let mut ids = Vec::new();
            for _ in 0..count {
                let cap = sizes[*nid % sizes.len()];
                let id = NodeId(*nid);
                nodes.push(Node::new(
                    id,
                    format!("{}-{}", prefix, id.index()),
                    cap,
                    gain,
                ));
                ids.push(id);
                *nid += 1;
            }
            ids
        };

        let mut lb_ids = Vec::new();
        for _ in 0..3 {
            let id = NodeId(nid);
            nodes.push(Node::new(id, format!("lb-{}", id.index()), 320.0, 1.0));
            lb_ids.push(id);
            nid += 1;
        }
        let api_ids = make_nodes(&mut nodes, &mut nid, 10, "api", 1.05);
        let auth_ids = make_nodes(&mut nodes, &mut nid, 5, "auth", 1.3);
        let router_ids = make_nodes(&mut nodes, &mut nid, 5, "router", 1.0);
        let cache_ids = make_nodes(&mut nodes, &mut nid, 40, "cache", 0.7);
        let orders_ids = make_nodes(&mut nodes, &mut nid, 20, "orders", 1.4);
        let worker_ids = make_nodes(&mut nodes, &mut nid, 30, "worker", 1.6);
        let db_ids = make_nodes(&mut nodes, &mut nid, 10, "db", 0.0);

        let all_nodes: Vec<NodeId> = (0..nid).map(NodeId).collect();

        let cap = |id: NodeId| nodes[id.index()].capacity();

        let mut has_edge = vec![vec![false; nid]; nid];

        let mut add_edge = |from: NodeId, to: NodeId| {
            if from == to || has_edge[from.index()][to.index()] {
                return;
            }
            has_edge[from.index()][to.index()] = true;
            edges.push(Edge::new(EdgeId(eid), from, to, cap(to)));
            eid += 1;
        };

        for api in &api_ids {
            add_edge(lb_ids[0], *api);
        }

        for db in &db_ids {
            add_edge(lb_ids[1], *db);
        }

        let mut reachable = vec![false; nid];
        for lb in &lb_ids {
            reachable[lb.index()] = true;
        }

        let mut frontier = lb_ids.clone();
        while reachable.iter().any(|r| !r) {
            let from = frontier[rng.gen_range(0..frontier.len())];
            let to = all_nodes[rng.gen_range(0..all_nodes.len())];
            if !reachable[to.index()] {
                add_edge(from, to);
                reachable[to.index()] = true;
                frontier.push(to);
            }
        }

        let extra_edges = nid * 3;
        for _ in 0..extra_edges {
            let from = all_nodes[rng.gen_range(0..all_nodes.len())];
            let to = all_nodes[rng.gen_range(0..all_nodes.len())];
            add_edge(from, to);
        }

        let graph = Graph::new(nodes, edges);

        let groups = GroupSet::new(vec![
            Group::new("LoadBalancers".into(), lb_ids.clone()),
            Group::new("Ingress".into(), api_ids),
            Group::new("Auth".into(), auth_ids),
            Group::new("Routers".into(), router_ids),
            Group::new("Cache".into(), cache_ids),
            Group::new("Orders".into(), orders_ids),
            Group::new("Workers".into(), worker_ids),
            Group::new("Database".into(), db_ids),
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

        let scenario = RandomStressScenario {
            entry: lb_ids,
            base_load: 100.0,
            ramp_per_turn: 25.0,
            max_load: 6000.0,
        };

        (graph, groups, snapshot, Box::new(scenario))
    }
}

impl Scenario for RandomStressScenario {
    fn load(&self, node_id: NodeId, turn: usize) -> f64 {
        if self.entry.contains(&node_id) {
            let spike = if turn % 17 == 0 {
                1.4
            } else if turn % 11 == 0 {
                0.7
            } else {
                1.0
            };
            let t = turn as f64;
            let ramp = self.ramp_per_turn * (t + 1.0).ln();
            ((self.base_load + ramp) * spike).min(self.max_load)
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
