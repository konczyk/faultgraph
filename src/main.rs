use crate::graph::edge::{Edge, EdgeId};
use crate::graph::graph::Graph;
use crate::graph::node::{Node, NodeId};
use crate::scenario::scenario::Scenario;
use crate::simulation::engine::SimulationEngine;
use crate::state::edge_state::EdgeState;
use crate::state::node_state::NodeState;
use crate::state::snapshot::Snapshot;
use crate::tui::app::App;
use crate::tui::draw::draw_app;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

mod graph;
mod scenario;
mod simulation;
mod state;
mod tui;

pub fn build_graph() -> Graph {
    let nodes = vec![
        Node::new(NodeId(0), "api-gateway".to_string(), 100.0),
        Node::new(NodeId(1), "auth-service".to_string(), 60.0),
        Node::new(NodeId(2), "orders-service".to_string(), 80.0),
        Node::new(NodeId(3), "redis-cache".to_string(), 50.0),
        Node::new(NodeId(4), "postgres-db".to_string(), 70.0),
    ];

    let edges = vec![
        Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0), // api → auth
        Edge::new(EdgeId(1), NodeId(0), NodeId(2), 1.0), // api → orders
        Edge::new(EdgeId(2), NodeId(1), NodeId(3), 1.2), // auth → redis
        Edge::new(EdgeId(3), NodeId(2), NodeId(4), 1.5), // orders → postgres
    ];

    Graph::new(nodes, edges)
}

pub struct BasicScenario {
    entry: Vec<NodeId>,
    base_load: f64,
    ramp_per_turn: f64,
    max_load: f64,
}

impl BasicScenario {
    pub fn new(entry: Vec<NodeId>) -> Self {
        Self {
            entry,
            base_load: 10.0,
            ramp_per_turn: 2.5,
            max_load: 200.0,
        }
    }
}

impl Scenario for BasicScenario {
    fn load(&self, node_id: NodeId, turn: usize) -> f64 {
        if self.entry.contains(&node_id) {
            let load = self.base_load + self.ramp_per_turn * turn as f64;
            load.min(self.max_load)
        } else {
            0.0
        }
    }

    fn entry_nodes(&self) -> &[NodeId] {
        &self.entry
    }
}

pub fn build_engine() -> SimulationEngine {
    let graph = build_graph();
    let scenario = BasicScenario::new(vec![NodeId(0)]);
    let initial_snapshot = Snapshot::new(
        0,
        graph
            .nodes()
            .iter()
            .map(|_| NodeState::new(0.0, 1.0))
            .collect(),
        graph.edges().iter().map(|_| EdgeState::new(true)).collect(),
    );

    SimulationEngine::new(graph, initial_snapshot, Box::new(scenario))
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let engine = build_engine();
    let mut app = App::new(engine);

    loop {
        let _ = terminal.draw(|frame| draw_app(frame, &app));

        if crossterm::event::poll(Duration::from_millis(16))? {
            match crossterm::event::read()? {
                Event::Key(key)
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
                {
                    break;
                }
                Event::Key(key)
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') =>
                {
                    app.engine.step()
                }
                _ => continue,
            }
        }
    }
    Ok(())
}
