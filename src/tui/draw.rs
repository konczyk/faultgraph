use crate::graph::node::NodeId;
use crate::tui::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn draw_app(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(frame.area());

    frame.render_widget(build_header(app), chunks[0]);
    frame.render_widget(build_node_table(app), chunks[1]);
}

fn build_header(app: &'_ App) -> Block<'_> {
    Block::new()
        .borders(Borders::ALL)
        .title(format!(" Faultgraph — Turn: {} ", app.engine.current_snapshot().turn()))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
}

fn build_node_table(app: &'_ App) -> Table<'_> {
    let states = app.engine.current_snapshot().node_states();
    let graph = app.engine.graph();

    let mut rows = states
        .iter()
        .enumerate()
        .map(|(i, state)| {
            let node = graph.node_by_id(NodeId(i));
            (
                i,
                if node.capacity() > 0.0 {
                    state.load() / node.capacity()
                } else {
                    0.0
                }
            )
        })
        .collect::<Vec<(usize, f64)>>();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Table::new(
        rows.iter()
            .map(|(i, utilization)| {
                let node = graph.node_by_id(NodeId(*i));
                let state = &states[*i];

                Row::new(vec![
                    Cell::from(i.to_string()),
                    Cell::from(node.name()),
                    Cell::from(format!("{:.1}", state.load())),
                    Cell::from(format!("{:.1}", node.capacity())),
                    Cell::from(format!("{:.2}", state.health())),
                    Cell::from(format!("{:.2}", utilization)),
                ])
            }),
        [
            Constraint::Length(4),
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(Row::new([
        Cell::from("ID"),
        Cell::from("Name"),
        Cell::from("Load"),
        Cell::from("Cap"),
        Cell::from("Health"),
        Cell::from("Util"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Nodes"))
}

