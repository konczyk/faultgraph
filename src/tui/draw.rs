use crate::analysis::groups::{GroupHealth, GroupTrend};
use crate::graph::node::{Node, NodeId};
use crate::state::node_state::NodeState;
use crate::tui::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::Color::{Black, Gray, LightGreen, White};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Cell, Padding, Paragraph, Row, Table};

fn find_pressure(app: &App) -> Vec<f64> {
    app.aggregations
        .iter()
        .find(|(i, _)| *i == app.selected_group_id())
        .map_or(vec![], |(_, s)| s.pressure().to_vec())
}

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

    let groups = Layout::vertical([
        Constraint::Length((app.engine.groups().groups().len() + 3) as u16),
        Constraint::Fill(1),
    ])
    .split(body[0]);

    let pressure = find_pressure(app);
    let non_zero = pressure.iter().filter(|p| **p > 0.0).count();

    let details = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length((non_zero + 2).min(4).clamp(3, 6) as u16),
        Constraint::Fill(1),
    ])
    .split(groups[1].inner(Margin::new(2, 0)));

    frame.render_widget(build_title(app), topbar[0]);
    frame.render_widget(build_turn(app), topbar[1]);
    frame.render_widget(build_indicators(app), topbar[2]);

    frame.render_widget(build_group_table(app), groups[0]);
    frame.render_widget(build_details_block(app), groups[1]);
    frame.render_widget(build_details_stats(app), details[0]);
    frame.render_widget(build_details_pressure(app), details[1]);
    frame.render_widget(build_details_most_pressured(app), details[2]);
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
    let entry_nodes = app.engine.scenario().entry_nodes();

    let incoming_load = entry_nodes
        .iter()
        .map(|id| {
            app.engine
                .scenario()
                .load(*id, app.engine.current_snapshot().turn())
        })
        .sum::<f64>();

    let (agg_served, agg_capacity) = app
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
                .capacity_mod(app.engine.groups().group_by_node_id(i));
            (s.served(), nodes[i].capacity() * capacity_mod.factor())
        })
        .fold((0.0, 0.0), |acc, agg| (acc.0 + agg.0, acc.1 + agg.1));

    let avg_util = if agg_capacity == 0.0 {
        0.0
    } else {
        agg_served / agg_capacity
    };

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

    let health_style = if avg_health < 0.3 {
        Style::default().fg(Color::Red)
    } else if avg_health < 0.8 {
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
                    Span::from(format!(
                        "{:>7.1}",
                        (summary.avg_utilization().min(1.0)) * 100.0
                    )),
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

fn build_details_block(_app: &'_ App) -> Block<'_> {
    Block::bordered().title(" Details ".bold())
}

fn build_details_stats(app: &'_ App) -> Paragraph<'_> {
    let group_id = app.selected_group_id();
    let group = &app.engine.groups().groups()[group_id];
    let nodes = group.nodes().iter().count();
    let healthy = group
        .nodes()
        .iter()
        .filter(|n| app.engine.current_snapshot().node_states()[n.index()].is_healthy())
        .count();
    let aggregations = &app
        .aggregations
        .iter()
        .find(|(g_id, _)| *g_id == app.selected_group_id())
        .unwrap()
        .1;

    let mods = app
        .engine
        .current_snapshot()
        .capacity_mods()
        .iter()
        .filter(|m| m.is_active())
        .map(|m| {
            let name = if m.factor() < 1.0 {
                "Throttle"
            } else {
                "Boost"
            };
            Span::from(format!(
                "Mods: {name} x{:.1} ({}t left) ",
                m.factor(),
                m.remaining_turns()
            ))
        })
        .collect::<Vec<Span>>();

    let text: Text = vec![
        "".into(),
        "".into(),
        format!("Group: {}", group.name()).into(),
        "".into(),
        format!("Nodes: {} / {}", healthy, nodes).into(),
        format!(
            "Avg Util: {}%",
            (aggregations.avg_utilization() * 100.0).round() as usize
        )
        .into(),
        format!(
            "Health: {}%",
            (aggregations.raw_health() * 100.0).round() as usize
        )
        .into(),
        if mods.len() > 0 {
            Line::from(mods)
        } else {
            "Mods: None".into()
        },
        "".into(),
        "Incoming Pressure (this turn)".into(),
    ]
    .into();
    Paragraph::new(text)
}

fn build_details_pressure(app: &'_ App) -> Paragraph<'_> {
    let mut lines: Vec<Line> = vec!["".into(), "Incoming Pressure (this turn)".into()];
    let pressure = find_pressure(app);
    let mut non_zero = pressure
        .iter()
        .enumerate()
        .filter(|(_, p)| **p > 0.0)
        .map(|(i, p)| {
            (
                if i == app.selected_group_id() {
                    "Internal"
                } else {
                    app.aggregations
                        .iter()
                        .find(|(g_id, _)| i == *g_id)
                        .map_or("", |(_, s)| s.name())
                },
                i,
                *p,
            )
        })
        .collect::<Vec<(&str, usize, f64)>>();
    if non_zero.is_empty() {
        lines.push("None".into())
    } else {
        let total_pressure = non_zero.iter().map(|(_, _, p)| p).sum::<f64>();

        non_zero.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        let mut top = non_zero[0..non_zero.len().min(3)].to_vec();
        let top_contains_internal = top
            .iter()
            .find(|(_, group_id, _)| *group_id == app.selected_group_id())
            .is_some();
        let internal = non_zero
            .iter()
            .find(|(_, group_id, p)| *group_id == app.selected_group_id() && *p > 0.0);

        if top.len() == 3 && non_zero.len() > 3 && !top_contains_internal && internal.is_some() {
            top.push(*internal.unwrap());
        }

        for (name, _, p) in &top {
            lines.push(Line::from(vec![
                {
                    let filled = (((p / total_pressure) * 16.0).round() as usize).min(16);
                    Span::from(format!(
                        "[{}{}]",
                        "#".repeat(filled),
                        "-".repeat(16 - filled)
                    ))
                },
                Span::from(format!(" {:<20}", name)),
                Span::from(format!(
                    " {:>3}%",
                    ((p / total_pressure) * 100.0).round() as usize
                )),
            ]));
        }
    }

    Paragraph::new(Text::from(lines))
}

fn build_details_most_pressured(app: &'_ App) -> Paragraph<'_> {
    let mut lines: Vec<Line> = vec!["".into(), "Most Pressured Nodes".into()];
    let mut most_pressured = app.engine.groups().groups()[app.selected_group_id()]
        .nodes()
        .iter()
        .map(|n_id| {
            (
                app.engine.graph().node_by_id(*n_id),
                &app.engine.current_snapshot().node_states()[n_id.index()],
            )
        })
        .filter(|(_, state)| state.is_healthy())
        .collect::<Vec<(&Node, &NodeState)>>();

    if most_pressured.is_empty() {
        lines.push("None".into())
    } else {
        let throttle = app
            .engine
            .current_snapshot()
            .capacity_mod(app.selected_group_id())
            .factor();
        most_pressured.sort_by(|a, b| {
            let pressure_a = (a.1.demand() + a.1.backlog()) / (a.0.capacity() * throttle);
            let pressure_b = (b.1.demand() + b.1.backlog()) / (b.0.capacity() * throttle);
            pressure_b.partial_cmp(&pressure_a).unwrap()
        });
        let top = most_pressured[0..most_pressured.len().min(3)].to_vec();
        for (node, state) in &top {
            let util = state.served() / (node.capacity() * throttle);
            lines.push(Line::from(vec![
                Span::from(format!("{:<20}", node.name())),
                Span::from(format!("util: {:>3}%", (util * 100.0).round() as usize)),
                Span::from(format!(
                    "   backlog: {:>4}",
                    state.backlog().round() as usize
                ))
                .dim(),
            ]));
        }
    }

    Paragraph::new(Text::from(lines))
}

fn build_node_table(app: &'_ App) -> Table<'_> {
    let node_states = app.engine.current_snapshot().node_states();
    let graph = app.engine.graph();

    let mut rows = node_states
        .iter()
        .enumerate()
        .filter(|(i, _)| app.engine.groups().group_by_node_id(*i) == app.selected_group_id())
        .map(|(i, state)| {
            let node = graph.node_by_id(NodeId(i));
            let capacity_mod = app
                .engine
                .current_snapshot()
                .capacity_mod(app.engine.groups().group_by_node_id(i));
            let capacity = node.capacity() * capacity_mod.factor();
            (
                i,
                if capacity > 0.0 {
                    (
                        (if state.is_healthy() {
                            state.demand() + state.backlog()
                        } else {
                            0.0
                        }) / capacity,
                        state.served() / capacity,
                    )
                } else {
                    (0.0, 0.0)
                },
            )
        })
        .collect::<Vec<(usize, (f64, f64))>>();
    rows.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap());

    Table::new(
        rows.iter().map(|(i, (_, utilization))| {
            let node = graph.node_by_id(NodeId(*i));
            let state = &node_states[*i];

            Row::new(vec![
                Cell::from(i.to_string()),
                Cell::from(node.name()),
                Cell::from(format!("{:>7.1}", (utilization.min(1.0)) * 100.0)),
                Cell::from(format!("{:>8.1}", state.demand())),
                Cell::from(format!("{:>6.1}", node.capacity())),
                Cell::from(format!("{:>6.1}", state.health() * 100.0)),
                Cell::from(mods(app, app.engine.groups().group_by_node_id(*i))),
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
