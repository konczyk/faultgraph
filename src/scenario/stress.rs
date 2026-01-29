use crate::analysis::groups::{Group, GroupSet};
use crate::graph::edge::{Edge, EdgeId};
use crate::graph::graph::Graph;
use crate::graph::node::{Node, NodeId};
use crate::scenario::scenario::Scenario;
use crate::simulation::modifiers::CapacityModifier;
use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;
use crate::state::snapshot::Snapshot;

pub struct StressScenario {
    entry: Vec<NodeId>,
    base_load: f64,
    ramp_per_turn: f64,
    max_load: f64,
}

impl StressScenario {
    pub fn build() -> (Graph, GroupSet, Snapshot, Box<dyn Scenario>) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut nid = 0usize;
        let mut eid = 0usize;

        let api_cnt = 10;
        let auth_cnt = 5;
        let router_cnt = 5;
        let cache_cnt = 40;
        let orders_cnt = 20;
        let worker_cnt = 30;
        let db_cnt = 10;

        let sizes = [40.0, 80.0, 160.0];

        let mut make_nodes = |count: usize, prefix: &str, gain: f64| -> Vec<NodeId> {
            (0..count)
                .map(|i| {
                    let cap = sizes[(nid + i) % sizes.len()];
                    let id = NodeId(nid);
                    nodes.push(Node::new(id, format!("{}-{}", prefix, nid), cap, gain));
                    nid += 1;
                    id
                })
                .collect()
        };

        let api_ids = make_nodes(api_cnt, "api", 1.05);
        let auth_ids = make_nodes(auth_cnt, "auth", 1.3);
        let router_ids = make_nodes(router_cnt, "router", 1.0);
        let cache_ids = make_nodes(cache_cnt, "cache", 0.7);
        let orders_ids = make_nodes(orders_cnt, "orders", 1.4);
        let worker_ids = make_nodes(worker_cnt, "worker", 1.6);
        let db_ids = make_nodes(db_cnt, "db", 0.0);

        let cap = |id: NodeId| nodes[id.index()].capacity();

        for api in &api_ids {
            for auth in &auth_ids {
                edges.push(Edge::new(EdgeId(eid), *api, *auth, cap(*auth)));
                eid += 1;
            }
            for router in &router_ids {
                edges.push(Edge::new(EdgeId(eid), *api, *router, cap(*router)));
                eid += 1;
            }
            for cache in cache_ids.iter().take(6) {
                edges.push(Edge::new(EdgeId(eid), *api, *cache, cap(*cache)));
                eid += 1;
            }
        }

        for router in &router_ids {
            for cache in &cache_ids {
                edges.push(Edge::new(EdgeId(eid), *router, *cache, cap(*cache)));
                eid += 1;
            }
            for orders in orders_ids.iter().take(6) {
                edges.push(Edge::new(EdgeId(eid), *router, *orders, cap(*orders)));
                eid += 1;
            }
        }

        for cache in &cache_ids {
            for orders in orders_ids.iter().take(10) {
                edges.push(Edge::new(EdgeId(eid), *cache, *orders, cap(*orders)));
                eid += 1;
            }
        }

        for window in cache_ids.windows(2) {
            edges.push(Edge::new(EdgeId(eid), window[0], window[1], cap(window[1])));
            eid += 1;
        }

        for orders in &orders_ids {
            for worker in worker_ids.iter().take(8) {
                edges.push(Edge::new(EdgeId(eid), *orders, *worker, cap(*worker)));
                eid += 1;
            }
            for db in &db_ids {
                edges.push(Edge::new(EdgeId(eid), *orders, *db, cap(*db)));
                eid += 1;
            }
        }

        for worker in &worker_ids {
            for db in db_ids.iter().take(5) {
                edges.push(Edge::new(EdgeId(eid), *worker, *db, cap(*db)));
                eid += 1;
            }
        }

        let graph = Graph::new(nodes, edges);

        let groups = GroupSet::new(vec![
            Group::new("Ingress".into(), api_ids.clone()),
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

        let scenario = StressScenario {
            entry: api_ids,
            base_load: 50.0,
            ramp_per_turn: 8.0,
            max_load: 2000.0,
        };

        (graph, groups, snapshot, Box::new(scenario))
    }
}

impl Scenario for StressScenario {
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
