use crate::analysis::analysis::aggregate_groups;
use crate::analysis::groups::{GroupRisk, GroupTrend};
use crate::graph::node::NodeId;
use crate::tui::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Color::White;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Padding, Row, Table};
use ratatui::Frame;

pub fn draw_app(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length((app.groups().groups().len() + 3) as u16),
            Constraint::Length(1),
            Constraint::Min(5),
        ])
        .split(frame.area());

    frame.render_widget(build_header(app), chunks[0]);
    frame.render_widget(build_group_table(app), chunks[2]);
    frame.render_widget(build_node_table(app), chunks[4]);
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

fn build_header(app: &'_ App) -> Block<'_> {
    Block::new()
        .title(Line::from(vec![
            Span::raw(" Faultgraph ").style(Style::default().bold().cyan()),
            Span::raw("—").style(Style::default().add_modifier(Modifier::DIM)),
            Span::raw(" Turn: ").style(Style::default().add_modifier(Modifier::DIM)),
            Span::raw(format!("{}", app.engine.current_snapshot().turn()))
                .style(Style::default().bold()),
            Span::raw(" "),
        ]))
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
                Cell::from(format!("{}", trend)).style(Style::default().bold()),
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
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::from(" Groups ").style(Style::default().bold()),
            ]))
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
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::from(" Nodes ").style(Style::default().bold()),
            ]))
            .padding(Padding::horizontal(1)),
    )
}
