use crate::analysis::analysis::aggregate_groups;
use crate::analysis::groups::{GroupHealth, GroupSummary, GroupTrend};
use crate::graph::node::NodeId;
use crate::tui::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color::{LightGreen, White};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Padding, Paragraph, Row, Table};

pub fn draw_app(frame: &mut Frame, app: &App) {
    let main = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    let topbar = Layout::horizontal([
        Constraint::Ratio(2, 5),
        Constraint::Ratio(1, 5),
        Constraint::Ratio(2, 5),
    ])
    .split(main[1]);

    let body = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(main[2]);

    frame.render_widget(build_title(app), topbar[0]);
    frame.render_widget(build_turn(app), topbar[1]);
    frame.render_widget(build_indicators(app), topbar[2]);

    frame.render_widget(build_group_table(app), body[0]);
    frame.render_widget(build_node_table(app), body[2]);

    frame.render_widget(build_status(app), main[4]);
}

fn build_title(_app: &'_ App) -> Paragraph<'_> {
    Paragraph::new(" FAULTGRAPH ").bold().cyan()
}

fn build_turn(app: &'_ App) -> Paragraph<'_> {
    Paragraph::new(Line::from(vec![
        Span::from(" Turn "),
        Span::from(format!("{}", app.engine.current_snapshot().turn())).bold(),
        Span::from(" | "),
        Span::from(" Ops "),
        Span::from(format!("{}", app.engine.remaining_ops())).bold(),
    ]))
    .centered()
}

fn build_indicators(app: &'_ App) -> Paragraph<'_> {
    let nodes = app.engine.graph().nodes();
    let states = app.engine.current_snapshot().node_states();
    let entry_nodes = app.engine.scenario().entry_nodes();

    let avg_load = if entry_nodes.len() > 0 {
        entry_nodes
            .iter()
            .map(|id| states[id.index()].load())
            .sum::<f64>()
            / entry_nodes.len() as f64
    } else {
        0.0
    };

    let avg_util = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .enumerate()
        .map(|(i, s)| s.load() / nodes[i].capacity())
        .sum::<f64>()
        / nodes.len() as f64;

    let avg_health = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .map(|s| s.health())
        .sum::<f64>()
        / nodes.len() as f64;

    let health_style = if avg_health < 0.4 {
        Style::default().fg(Color::Red)
    } else if avg_health < 0.7 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    Paragraph::new(Line::from(vec![
        Span::from(" L ").bold(),
        Span::from(format!(" {}rps ", avg_load as usize)),
        Span::from(" | ").dim(),
        Span::from(" U ").bold(),
        Span::from(format!(" {}%  ", (avg_util * 100.0) as usize)),
        Span::from(" | ").dim(),
        Span::from(" ♥ ").bold().style(health_style),
        Span::from(format!(" {}% ", (avg_health * 100.0) as usize)).style(health_style),
    ]))
    .right_aligned()
}

fn build_status(_app: &'_ App) -> Paragraph<'_> {
    Paragraph::new(Line::from(vec![
        Span::from(" [Q]"),
        Span::from(" Quit ").bold(),
        Span::from(" [Space]"),
        Span::from(" Step ").bold(),
        Span::from(" [T]"),
        Span::from(" Throttle ").bold(),
    ]))
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

fn dots(turns: u8) -> String {
    match turns {
        4 => "⢸",
        3 => "⢰",
        2 => "⢠",
        1 => "⢀",
        _ => "",
    }
    .to_string()
}

fn mods(app: &'_ App, group_id: usize) -> Line<'_> {
    let mut mods = Line::default();
    let throttle = app.engine.throttle(group_id);
    if throttle.is_active() {
        let turns = dots(throttle.remaining_turns());
        let span;
        if throttle.is_just_applied() {
            span = Span::from(format!(" Tx{}{} ", throttle.factor(), turns))
                .bg(LightGreen)
                .bold();
        } else {
            span = Span::from(format!(" Tx{}{} ", throttle.factor(), turns)).dim();
        }
        mods.spans.push(span);
    }
    mods
}

fn build_group_table(app: &'_ App) -> Table<'_> {
    let mut aggregations = aggregate_groups(
        app.engine.groups(),
        app.engine.current_snapshot(),
        app.engine.previous_snapshot(),
        app.engine.graph(),
    )
    .into_iter()
    .enumerate()
    .collect::<Vec<(usize, GroupSummary)>>();

    aggregations.sort_by(|a, b| a.1.raw_health().partial_cmp(&b.1.raw_health()).unwrap());

    Table::new(
        aggregations.iter().map(|(g_id, summary)| {
            let util_trend = match summary.utilization_trend() {
                GroupTrend::Up => " ↗",
                GroupTrend::Down => " ↘",
                GroupTrend::Flat => " →",
            };

            let health_trend = match summary.health_trend() {
                GroupTrend::Up => " ↗",
                GroupTrend::Down => " ↘",
                GroupTrend::Flat => " →",
            };

            let health_style = match summary.health() {
                GroupHealth::Ok => Style::default().green(),
                GroupHealth::Degraded => Style::default().yellow(),
                GroupHealth::Critical => Style::default().light_red(),
                GroupHealth::Failed => Style::default().red().bold(),
            };

            Row::new(vec![
                Cell::from(summary.name().to_owned()),
                Cell::from(Line::from(vec![
                    Span::styled(
                        format!("{:>7.1}", summary.avg_utilization() * 100.0),
                        util_style(summary.avg_utilization()),
                    ),
                    Span::from(util_trend),
                ])),
                Cell::from(format!("{:>5}", summary.node_count())),
                Cell::from(Line::from(vec![
                    Span::styled(format!("{:>9}", summary.health()), health_style),
                    Span::from(health_trend),
                ])),
                Cell::from(mods(app, *g_id)),
            ])
        }),
        [
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(15),
            Constraint::Fill(1),
        ],
    )
    .header(
        Row::new([
            Cell::from("Group"),
            Cell::from("   Util %"),
            Cell::from(" Nodes"),
            Cell::from("    Status"),
            Cell::from("Mods"),
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
                Cell::from(format!("{:>7.1}", utilization * 100.0)).style(util_style(*utilization)),
                Cell::from(format!("{:>8.1}", state.load())),
                Cell::from(format!("{:>6.1}", node.capacity())),
                Cell::from(format!("{:>6.1}", state.health() * 100.0)),
                Cell::from(mods(app, app.engine.group_by_node_id(*i))),
            ])
        }),
        [
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new([
            Cell::from("ID"),
            Cell::from("Name"),
            Cell::from("  Util %"),
            Cell::from(" Load rps"),
            Cell::from("  Cap"),
            Cell::from("Health %"),
            Cell::from("Mods"),
        ])
        .style(Style::default().bg(Color::DarkGray).fg(White)),
    )
    .block(
        Block::bordered()
            .title(" Nodes ".bold())
            .padding(Padding::horizontal(1)),
    )
}
