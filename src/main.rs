use crate::scenario::basic::BasicScenario;
use crate::scenario::random::RandomStressScenario;
use crate::scenario::stress::StressScenario;
use crate::simulation::engine::SimulationEngine;
use crate::tui::app::App;
use crate::tui::draw::draw_app;
use clap::{Parser, ValueEnum};
use crossterm::event::KeyCode::{Down, Up};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

mod analysis;
mod graph;
mod scenario;
mod simulation;
mod state;
mod tui;

#[derive(Clone, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
enum ScenarioKind {
    Basic,
    Random,
    Stress,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, short, value_enum, default_value_t = ScenarioKind::Basic)]
    scenario: ScenarioKind,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut terminal = ratatui::init();
    let (graph, groups, initial_snapshot, scenario) = match args.scenario {
        ScenarioKind::Basic => BasicScenario::build(),
        ScenarioKind::Random => RandomStressScenario::build(12345),
        ScenarioKind::Stress => StressScenario::build(),
    };
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
                    app.refresh_groups();
                    app.engine.step();
                }
                Event::Key(key)
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('t') =>
                {
                    app.engine.try_throttle_group(app.selected_group_id());
                }
                Event::Key(key)
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('b') =>
                {
                    app.engine.try_boost_group(app.selected_group_id());
                }
                Event::Key(key) if key.kind == KeyEventKind::Press && key.code == Up => {
                    app.select_previous_group();
                }
                Event::Key(key) if key.kind == KeyEventKind::Press && key.code == Down => {
                    app.select_next_group();
                }
                _ => continue,
            }
        }
    }
    Ok(())
}
