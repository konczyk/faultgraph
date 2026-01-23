use crate::simulation::engine::SimulationEngine;

pub enum SortMode {
    Utilization,
    Load,
    Health,
}

pub struct App {
    pub engine: SimulationEngine,
    pub running: bool,
    pub sort_mode: SortMode,
}

impl App {
    pub fn new(engine: SimulationEngine) -> Self {
        Self { engine, running: true, sort_mode: SortMode::Utilization}
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
