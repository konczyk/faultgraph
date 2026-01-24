use crate::analysis::groups::{Group, GroupRisk, GroupSet, GroupSummary, GroupTrend};
use crate::graph::graph::Graph;
use crate::state::snapshot::Snapshot;

fn calc_util(snapshot: &Snapshot, group: &Group, graph: &Graph) -> f64 {
    let states = snapshot.node_states();
    let (agg_util, cnt) = group
        .nodes()
        .iter()
        .map(|id| {
            let capacity = graph.node_by_id(*id).capacity();
            if capacity > 0.0 {
                states[id.index()].load() / capacity
            } else {
                0.0
            }
        })
        .fold((0.0, 0), |(sum, cnt), u| (sum + u, cnt + 1));
    if cnt > 0 { agg_util / cnt as f64 } else { 0.0 }
}

fn calc_worst_health(snapshot: &Snapshot, group: &Group) -> f64 {
    let states = snapshot.node_states();
    group
        .nodes()
        .iter()
        .map(|id| states[id.index()])
        .map(|s| s.health())
        .filter(|h| !h.is_nan())
        .reduce(f64::min)
        .unwrap_or(0.0)
}

pub fn aggregate_groups(
    group_set: &GroupSet,
    current_snapshot: &Snapshot,
    previous_snapshot: &Snapshot,
    graph: &Graph,
) -> Vec<GroupSummary> {
    let epsilon = 0.02;
    group_set
        .groups()
        .iter()
        .map(|g| {
            let prev_avg_util = calc_util(&previous_snapshot, &g, &graph);
            let curr_avg_util = calc_util(&current_snapshot, &g, &graph);
            let util_diff = curr_avg_util - prev_avg_util;

            let trend = if util_diff > epsilon {
                GroupTrend::Up
            } else if util_diff < -epsilon {
                GroupTrend::Down
            } else {
                GroupTrend::Flat
            };

            let worst_health = calc_worst_health(&current_snapshot, &g);

            let risk = match worst_health {
                n if n > 0.7 => GroupRisk::Low,
                n if n <= 0.7 && n > 0.4 => GroupRisk::Medium,
                n if n <= 0.4 && n > 0.1 => GroupRisk::High,
                _ => GroupRisk::Critical,
            };

            GroupSummary::new(
                g.name().to_string(),
                curr_avg_util,
                trend,
                g.nodes().len(),
                worst_health,
                risk,
            )
        })
        .collect()
}
