use crate::analysis::analysis::aggregate_groups;
use crate::analysis::groups::{GroupRisk, GroupTrend};
use crate::graph::node::NodeId;
use crate::tui::app::App;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Color::White;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Cell, Padding, Row, Table};
use ratatui::Frame;

pub fn draw_app(frame: &mut Frame, app: &App) {
    let main = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(frame.area());

    let sides = Layout::horizontal([
        Constraint::Percentage(70),
        Constraint::Length(3),
        Constraint::Percentage(30),
    ])
    .split(main[1]);

    let left_chunks = Layout::vertical([
        Constraint::Length((app.groups().groups().len() + 3) as u16),
        Constraint::Length(1),
        Constraint::Min(5),
    ])
    .split(sides[0]);

    let right_chunks = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(6),
    ])
    .split(sides[2]);

    frame.render_widget(build_header(app), main[0]);
    frame.render_widget(build_group_table(app), left_chunks[0]);
    frame.render_widget(build_node_table(app), left_chunks[2]);
    frame.render_widget(build_status(app), right_chunks[0]);
    frame.render_widget(build_keys(app), right_chunks[2]);
}

fn util_style(utilization: f64) -> Style {
    if utilization < 0.8 {
        Style::default().fg(Color::Green)
    } else if utilization <= 1.0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Red)
    }
}

fn build_status(app: &'_ App) -> Table<'_> {
    let nodes = app.engine.graph().nodes();
    let avg_util: f64 = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .enumerate()
        .map(|(i, s)| s.load() / nodes[i].capacity())
        .sum();
    let over_cap = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .enumerate()
        .filter(|(i, s)| s.load() > nodes[*i].capacity())
        .count();
    let unhealthy = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .filter(|s| !s.is_healthy())
        .count();
    let worst_health = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .map(|s| s.health())
        .reduce(f64::min)
        .unwrap_or(0.0);

    Table::new(
        [
            Row::new(vec![
                Cell::from("Turn: "),
                Cell::from(format!("{}", app.engine.current_snapshot().turn())),
            ]),
            Row::new(vec![Cell::from("Scenario: "), Cell::from("Basic")]),
            Row::new(vec![Cell::from(""), Cell::from("")]),
            Row::new(vec![
                Cell::from("Avg Util: "),
                Cell::from(format!("{:.2}", avg_util / nodes.len() as f64)),
            ]),
            Row::new(vec![
                Cell::from("Over Cap: "),
                Cell::from(format!("{} / {}", over_cap, nodes.len())),
            ]),
            Row::new(vec![
                Cell::from("Unhealthy: "),
                Cell::from(format!("{} / {}", unhealthy, nodes.len())),
            ]),
            Row::new(vec![
                Cell::from("Worst Health: "),
                Cell::from(format!("{:.2}", worst_health)),
            ]),
        ],
        [Constraint::Ratio(1, 2), Constraint::Fill(1)],
    )
    .block(
        Block::bordered()
            .title(" Status ".bold())
            .padding(Padding::uniform(1)),
    )
}

fn build_keys(app: &'_ App) -> Table<'_> {
    Table::new(
        [
            Row::new(vec![Cell::from("[Space] "), Cell::from("Step")]),
            Row::new(vec![Cell::from("[Q] "), Cell::from("Quit")]),
        ],
        [Constraint::Ratio(1, 2), Constraint::Fill(1)],
    )
    .block(
        Block::bordered()
            .title(" Keys ".bold())
            .padding(Padding::uniform(1)),
    )
}

fn build_header(app: &'_ App) -> Block<'_> {
    Block::new()
        .title(" FAULTGRAPH ".bold())
        .title_alignment(Alignment::Center)
}

fn build_group_table(app: &'_ App) -> Table<'_> {
    let mut aggregations = aggregate_groups(
        app.groups(),
        app.engine.current_snapshot(),
        app.engine.previous_snapshot(),
        app.engine.graph(),
    );

    aggregations.sort_by(|a, b| a.worst_health().partial_cmp(&b.worst_health()).unwrap());

    Table::new(
        aggregations.iter().map(|summary| {
            let trend = match summary.trend() {
                GroupTrend::Up => "  ↗",
                GroupTrend::Down => "  ↘",
                GroupTrend::Flat => "  →",
            };

            let risk_style = match summary.risk() {
                GroupRisk::Low => Style::default().add_modifier(Modifier::DIM),
                GroupRisk::Medium => Style::default().yellow(),
                GroupRisk::High => Style::default().light_red(),
                GroupRisk::Critical => Style::default().red().bold(),
            };

            Row::new(vec![
                Cell::from(summary.name().to_owned()),
                Cell::from(format!("{:>6.2}", summary.avg_utilization()))
                    .style(util_style(summary.avg_utilization())),
                Cell::from(format!("{trend}")).style(Style::default().bold()),
                Cell::from(format!("{:>5}", summary.node_count())),
                Cell::from(format!("{:?}", summary.risk())).style(risk_style),
            ])
        }),
        [
            Constraint::Length(25),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new([
            Cell::from("Group"),
            Cell::from(" Util"),
            Cell::from("Trend"),
            Cell::from("Nodes"),
            Cell::from("Risk"),
        ])
        .style(Style::default().bg(Color::DarkGray).fg(White)),
    )
    .block(
        Block::bordered()
            .title(" Groups ".bold())
            .padding(Padding::horizontal(1)),
    )
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
                },
            )
        })
        .collect::<Vec<(usize, f64)>>();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Table::new(
        rows.iter().map(|(i, utilization)| {
            let node = graph.node_by_id(NodeId(*i));
            let state = &states[*i];

            Row::new(vec![
                Cell::from(i.to_string()),
                Cell::from(node.name()),
                Cell::from(format!("{:>6.2}", utilization)).style(util_style(*utilization)),
                Cell::from(format!("{:>6.1}", state.load())),
                Cell::from(format!("{:>6.1}", node.capacity())),
                Cell::from(format!("{:>6.2}", state.health())),
            ])
        }),
        [
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new([
            Cell::from("ID"),
            Cell::from("Name"),
            Cell::from(" Util"),
            Cell::from(" Load"),
            Cell::from("  Cap"),
            Cell::from("Health"),
        ])
        .style(Style::default().bg(Color::DarkGray).fg(White)),
    )
    .block(
        Block::bordered()
            .title(" Nodes ".bold())
            .padding(Padding::horizontal(1)),
    )
}
