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
        let mut eid = 0usize;
        let mut nid = 0usize;

        let api_cnt = 10;
        let auth_cnt = 5;
        let router_cnt = 5;
        let cache_cnt = 40;
        let orders_cnt = 20;
        let worker_cnt = 30;
        let db_cnt = 10;

        let api_ids = (0..api_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("api-{}", nid), 120.0, 1.05));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        let auth_ids = (0..auth_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("auth-{}", nid), 60.0, 1.3));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        let router_ids = (0..router_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("router-{}", nid), 80.0, 1.0));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        let cache_ids = (0..cache_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("cache-{}", nid), 40.0, 0.7));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        let orders_ids = (0..orders_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("orders-{}", nid), 50.0, 1.4));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        let worker_ids = (0..worker_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("worker-{}", nid), 30.0, 1.6));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        let db_ids = (0..db_cnt)
            .map(|_| {
                let id = NodeId(nid);
                nodes.push(Node::new(id, format!("db-{}", nid), 25.0, 0.0));
                nid += 1;
                id
            })
            .collect::<Vec<_>>();

        for api in &api_ids {
            for auth in &auth_ids {
                edges.push(Edge::new(EdgeId(eid), *api, *auth, 0.8));
                eid += 1;
            }
            for router in &router_ids {
                edges.push(Edge::new(EdgeId(eid), *api, *router, 1.0));
                eid += 1;
            }
            for cache in cache_ids.iter().take(5) {
                edges.push(Edge::new(EdgeId(eid), *api, *cache, 0.6));
                eid += 1;
            }
        }

        for router in &router_ids {
            for cache in &cache_ids {
                edges.push(Edge::new(EdgeId(eid), *router, *cache, 1.2));
                eid += 1;
            }
            for orders in orders_ids.iter().take(5) {
                edges.push(Edge::new(EdgeId(eid), *router, *orders, 0.9));
                eid += 1;
            }
        }

        for cache in &cache_ids {
            for orders in orders_ids.iter().take(10) {
                edges.push(Edge::new(EdgeId(eid), *cache, *orders, 1.1));
                eid += 1;
            }
            for api in api_ids.iter().take(2) {
                edges.push(Edge::new(EdgeId(eid), *cache, *api, 0.15));
                eid += 1;
            }
        }

        for orders in &orders_ids {
            for worker in worker_ids.iter().take(8) {
                edges.push(Edge::new(EdgeId(eid), *orders, *worker, 1.3));
                eid += 1;
            }
            for db in &db_ids {
                edges.push(Edge::new(EdgeId(eid), *orders, *db, 0.8));
                eid += 1;
            }
        }

        for worker in &worker_ids {
            for db in db_ids.iter().take(5) {
                edges.push(Edge::new(EdgeId(eid), *worker, *db, 1.0));
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
            let load = (self.base_load + self.ramp_per_turn * turn as f64) * spike;
            load.min(self.max_load)
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
