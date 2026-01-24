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
use crate::scenario::basic::BasicScenario;

mod analysis;
mod graph;
mod scenario;
mod simulation;
mod state;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let (graph, groups, initial_snapshot, scenario) = BasicScenario::build();
    let engine = SimulationEngine::new(graph, initial_snapshot, scenario);
    let mut app = App::new(engine, groups);

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
