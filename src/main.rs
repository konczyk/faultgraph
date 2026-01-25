use crate::scenario::basic::BasicScenario;
use crate::simulation::engine::SimulationEngine;
use crate::tui::app::App;
use crate::tui::draw::draw_app;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

mod analysis;
mod graph;
mod scenario;
mod simulation;
mod state;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let (graph, groups, initial_snapshot, scenario) = BasicScenario::build();
    let engine = SimulationEngine::new(graph, groups, initial_snapshot, scenario);

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
                    app.engine.step();
                }
                Event::Key(key)
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('t') =>
                {
                    if app.engine.try_throttle_group(0) {
                        let _ = terminal.draw(|frame| draw_app(frame, &app));
                    }
                }
                _ => continue,
            }
        }
    }
    Ok(())
}
