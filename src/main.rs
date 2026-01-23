use crate::graph::graph::Graph;
use crate::graph::node::NodeId;
use crate::scenario::scenario::Scenario;
use crate::simulation::engine::SimulationEngine;
use crate::state::snapshot::Snapshot;
use crate::tui::app::App;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, BorderType, Borders};
use std::io;
use std::time::Duration;

mod graph;
mod state;
mod simulation;
mod scenario;
mod tui;

struct BasicScenario {}
impl Scenario for BasicScenario {
    fn load(&self, node_id: NodeId, turn: usize) -> f64 {
        0.0
    }
    fn entry_nodes(&self) -> &[NodeId] {
        &[]
    }
}

pub fn build_engine() -> SimulationEngine {
    let graph = Graph::new(vec![], vec![]);
    let initial_snapshot = Snapshot::new(0, vec![], vec![]);

    SimulationEngine::new(graph, initial_snapshot, Box::new(BasicScenario{}))

}

fn main() -> io::Result<()>{
    let mut terminal = ratatui::init();
    let engine = build_engine();
    let mut app = App::new(engine);

    loop {
        let _ = terminal.draw(|frame| {
            let block = Block::new()
                .borders(Borders::ALL)
                .title(format!(" Faultgraph — Turn: {} ", app.engine.current_snapshot().turn()))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            frame.render_widget(block, frame.area());
        });

        if crossterm::event::poll(Duration::from_millis(16))? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') => break,
                Event::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') => app.engine.step(),
                _ => continue
            }
        }
    }
     Ok(())
}
