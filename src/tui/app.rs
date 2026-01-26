use crate::analysis::analysis::aggregate_groups;
use crate::analysis::groups::GroupSummary;
use crate::simulation::engine::SimulationEngine;

pub struct App {
    pub engine: SimulationEngine,
    pub aggregations: Vec<(usize, GroupSummary)>,
    pub selected_index: usize,
}

impl App {
    pub fn new(engine: SimulationEngine) -> Self {
        let mut app = Self {
            engine,
            aggregations: vec![],
            selected_index: 0,
        };
        app.refresh_groups();
        app
    }

    pub fn refresh_groups(&mut self) {
        self.aggregations = aggregate_groups(
            self.engine.groups(),
            self.engine.current_snapshot(),
            self.engine.previous_snapshot(),
            self.engine.graph(),
        )
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, GroupSummary)>>();

        self.aggregations
            .sort_by(|a, b| a.1.raw_health().partial_cmp(&b.1.raw_health()).unwrap());
    }

    pub fn select_next_group(&mut self) {
        if self.selected_index + 1 < self.engine.groups().groups().len() {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn select_previous_group(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.engine.groups().groups().len() - 1;
        }
    }

    pub fn selected_group_id(&self) -> usize {
        self.aggregations[self.selected_index].0
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
