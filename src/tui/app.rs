use crate::analysis::groups::{GroupSet, GroupSummary};
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
    groups: GroupSet,
}

impl App {
    pub fn new(engine: SimulationEngine, groups: GroupSet) -> Self {
        Self {
            engine,
            running: true,
            sort_mode: SortMode::Utilization,
            groups,
        }
    }

    pub fn groups(&self) -> &GroupSet {
        &self.groups
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
