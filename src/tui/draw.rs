use crate::analysis::groups::{GroupHealth, GroupTrend};
use crate::graph::node::NodeId;
use crate::tui::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color::{Black, Gray, LightGreen, White};
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

    let incoming_load = entry_nodes
        .iter()
        .map(|id| {
            app.engine
                .scenario()
                .load(*id, app.engine.current_snapshot().turn())
        })
        .sum::<f64>();

    let (agg_util, cnt) = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_healthy())
        .map(|(i, s)| {
            let capacity_mod = app
                .engine
                .current_snapshot()
                .capacity_mod(app.engine.group_by_node_id(i));
            s.served() / (nodes[i].capacity() * capacity_mod.factor())
        })
        .fold((0.0, 0), |acc, u| (acc.0 + u, acc.1 + 1));

    let avg_util = if cnt == 0 { 0.0 } else { agg_util / cnt as f64 };

    let (agg_health, cnt) = app
        .engine
        .current_snapshot()
        .node_states()
        .iter()
        .map(|s| s.health())
        .fold((0.0, 0), |acc, h| (acc.0 + h, acc.1 + 1));

    let avg_health = if cnt == 0 {
        0.0
    } else {
        agg_health / cnt as f64
    };

    let health_style = if avg_health < 0.4 {
        Style::default().fg(Color::Red)
    } else if avg_health < 0.7 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    Paragraph::new(Line::from(vec![
        Span::from(" L ").bold(),
        Span::from(format!(" {}rps ", incoming_load as usize)),
        Span::from(" | ").dim(),
        Span::from(" U ").bold(),
        Span::from(format!(" {}%  ", (avg_util * 100.0).round() as usize)),
        Span::from(" | ").dim(),
        Span::from(" ♥ ").bold().style(health_style),
        Span::from(format!(" {}% ", (avg_health * 100.0).round() as usize)).style(health_style),
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
        Span::from(" [B]"),
        Span::from(" Boost ").bold(),
    ]))
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
    let capacity_mod = app.engine.current_snapshot().capacity_mod(group_id);
    if capacity_mod.is_active() {
        let turns = dots(capacity_mod.remaining_turns());
        let span;
        let mod_type = if capacity_mod.factor() > 1.0 {
            "B"
        } else {
            "T"
        };
        if capacity_mod.is_just_applied() {
            span = Span::from(format!(" {mod_type}x{}{} ", capacity_mod.factor(), turns))
                .bg(LightGreen)
                .bold();
        } else {
            span = Span::from(format!(" {mod_type}x{}{} ", capacity_mod.factor(), turns)).dim();
        }
        mods.spans.push(span);
    }
    mods
}

fn build_group_table(app: &'_ App) -> Table<'_> {
    Table::new(
        app.aggregations.iter().map(|(g_id, summary)| {
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

            let row_style = if *g_id == app.selected_group_id() {
                Style::default().bg(Gray).fg(Black)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(summary.name().to_owned()),
                Cell::from(Line::from(vec![
                    Span::from(format!("{:>7.1}", summary.avg_utilization() * 100.0)),
                    Span::from(util_trend),
                ])),
                Cell::from(format!(
                    "{:>8}",
                    format!("{} / {}", summary.healthy_nodes(), summary.node_count())
                )),
                Cell::from(Line::from(vec![
                    Span::styled(format!("{:>9}", summary.health()), health_style),
                    Span::from(health_trend),
                ])),
                Cell::from(mods(app, *g_id)),
            ])
            .style(row_style)
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
            Cell::from("   Nodes"),
            Cell::from("    Status"),
            Cell::from(" Mods"),
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
    let node_states = app.engine.current_snapshot().node_states();
    let graph = app.engine.graph();

    let mut rows = node_states
        .iter()
        .enumerate()
        .map(|(i, state)| {
            let node = graph.node_by_id(NodeId(i));
            (
                i,
                if node.capacity() > 0.0 {
                    state.served() / node.capacity()
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
            let state = &node_states[*i];

            Row::new(vec![
                Cell::from(i.to_string()),
                Cell::from(node.name()),
                Cell::from(format!("{:>7.1}", utilization * 100.0)),
                Cell::from(format!("{:>8.1}", state.demand())),
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
            Cell::from(" Mods"),
        ])
        .style(Style::default().bg(Color::DarkGray).fg(White)),
    )
    .block(
        Block::bordered()
            .title(" Nodes ".bold())
            .padding(Padding::horizontal(1)),
    )
}
