use crate::analysis::groups::{Group, GroupHealth, GroupSet, GroupSummary, GroupTrend};
use crate::graph::graph::Graph;
use crate::state::snapshot::Snapshot;

fn calc_util(snapshot: &Snapshot, group: &Group, graph: &Graph, group_id: usize) -> f64 {
    let node_states = snapshot.node_states();
    let capacity_mod = snapshot.capacity_mod(group_id);
    let (agg_served, agg_capacity) = group
        .nodes()
        .iter()
        .filter(|n_id| node_states[n_id.index()].is_healthy())
        .map(|id| {
            let capacity = graph.node_by_id(*id).capacity() * capacity_mod.factor();
            let served = node_states[id.index()].served();
            (served, capacity)
        })
        .fold((0.0, 0.0), |(sum_util, sum_cap), (u, c)| {
            (sum_util + u, sum_cap + c)
        });
    if agg_capacity > 0.0 {
        agg_served / agg_capacity
    } else {
        0.0
    }
}

fn calc_health(snapshot: &Snapshot, group: &Group) -> f64 {
    let states = snapshot.node_states();
    let h = group
        .nodes()
        .iter()
        .map(|id| &states[id.index()])
        .map(|s| s.health())
        .collect::<Vec<f64>>();
    if h.len() == 0 {
        0.0
    } else {
        h.iter().sum::<f64>() / h.iter().count() as f64
    }
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
        .enumerate()
        .map(|(g_id, g)| {
            let prev_avg_util = calc_util(&previous_snapshot, &g, &graph, g_id);
            let curr_avg_util = calc_util(&current_snapshot, &g, &graph, g_id);
            let util_diff = curr_avg_util - prev_avg_util;

            let avg_util_trend = if util_diff > epsilon {
                GroupTrend::Up
            } else if util_diff < -epsilon {
                GroupTrend::Down
            } else {
                GroupTrend::Flat
            };

            let prev_health = calc_health(&previous_snapshot, &g);
            let curr_health = calc_health(&current_snapshot, &g);
            let health_diff = curr_health - prev_health;

            let health_trend = if health_diff > epsilon {
                GroupTrend::Up
            } else if health_diff < -epsilon {
                GroupTrend::Down
            } else {
                GroupTrend::Flat
            };

            let health = match curr_health {
                n if n > 0.8 => GroupHealth::Ok,
                n if n > 0.3 => GroupHealth::Degraded,
                n if n > 0.0 => GroupHealth::Critical,
                _ => GroupHealth::Failed,
            };

            let states = current_snapshot.node_states();
            let healthy_nodes = g
                .nodes()
                .iter()
                .filter(|n_id| states[n_id.index()].is_healthy())
                .count();

            let mut pressure = vec![0.0; group_set.groups().len()];
            g.nodes()
                .iter()
                .flat_map(|n_id| graph.incoming(*n_id))
                .for_each(|e_id| {
                    let load = current_snapshot.edge_load(*e_id, graph);
                    let edge = graph.edge_by_id(*e_id);
                    let source_group = group_set.group_by_node_id(edge.from().index());
                    pressure[source_group] += load;
                });

            GroupSummary::new(
                g.name().to_string(),
                curr_avg_util,
                avg_util_trend,
                g.nodes().len(),
                curr_health,
                health,
                health_trend,
                healthy_nodes,
                pressure,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::edge::{Edge, EdgeId};
    use crate::graph::node::{Node, NodeId};
    use crate::simulation::modifiers::CapacityModifier;
    use crate::state::edge_state::EdgeState;
    use crate::state::node_state::NodeState;
    use approx::assert_relative_eq;

    #[test]
    fn test_avg_utilisation_per_group() {
        let api1 = Node::new(NodeId(0), "api1".to_string(), 100.0, 1.0);
        let db1 = Node::new(NodeId(1), "db1".to_string(), 60.0, 1.0);
        let link1 = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);
        let api2 = Node::new(NodeId(2), "api2".to_string(), 80.0, 1.0);
        let db2 = Node::new(NodeId(3), "db2".to_string(), 10.0, 1.0);
        let link2 = Edge::new(EdgeId(1), NodeId(2), NodeId(3), 1.0);

        let graph = Graph::new(vec![api1, db1, api2, db2], vec![link1, link2]);

        let groupset = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0), NodeId(1)]),
            Group::new("group2".to_string(), vec![NodeId(2), NodeId(3)]),
        ]);

        let previous_snapshot = Snapshot::new(
            5,
            vec![
                NodeState::new(0.0, 20.0, 0.0, 0.9),
                NodeState::new(0.0, 10.0, 0.0, 0.06),
                NodeState::new(0.0, 60.0, 0.0, 0.2),
                NodeState::new(0.0, 40.0, 0.0, 0.6),
            ],
            vec![EdgeState::new(true), EdgeState::new(true)],
            vec![CapacityModifier::new(); 2],
        );

        let current_snapshot = Snapshot::new(
            6,
            vec![
                NodeState::new(0.0, 10.0, 0.0, 0.5),
                NodeState::new(0.0, 50.0, 0.0, 0.2),
                NodeState::new(0.0, 30.0, 0.0, 0.05),
                NodeState::new(0.0, 90.0, 0.0, 0.8),
            ],
            vec![EdgeState::new(true), EdgeState::new(true)],
            vec![CapacityModifier::new(); 2],
        );

        let summaries = aggregate_groups(&groupset, &current_snapshot, &previous_snapshot, &graph);

        assert_relative_eq!(
            (10.0 + 50.0) / (100.0 + 60.0),
            summaries[0].avg_utilization()
        );
        assert_relative_eq!(
            (30.0 + 90.0) / (80.0 + 10.0),
            summaries[1].avg_utilization()
        );
    }

    #[test]
    fn test_trend_detection() {
        let api1 = Node::new(NodeId(0), "api1".to_string(), 100.0, 1.0);
        let db1 = Node::new(NodeId(1), "db1".to_string(), 60.0, 1.0);
        let link1 = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);
        let api2 = Node::new(NodeId(2), "api2".to_string(), 80.0, 1.0);
        let db2 = Node::new(NodeId(3), "db2".to_string(), 10.0, 1.0);
        let link2 = Edge::new(EdgeId(1), NodeId(2), NodeId(3), 1.0);
        let api3 = Node::new(NodeId(4), "api3".to_string(), 30.0, 1.0);
        let db3 = Node::new(NodeId(5), "db3".to_string(), 70.0, 1.0);
        let link3 = Edge::new(EdgeId(2), NodeId(4), NodeId(5), 1.0);

        let graph = Graph::new(
            vec![api1, db1, api2, db2, api3, db3],
            vec![link1, link2, link3],
        );

        let groupset = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0), NodeId(1)]),
            Group::new("group2".to_string(), vec![NodeId(2), NodeId(3)]),
            Group::new("group3".to_string(), vec![NodeId(4), NodeId(5)]),
        ]);

        let previous_snapshot = Snapshot::new(
            5,
            vec![
                NodeState::new(0.0, 20.0, 0.0, 0.9),
                NodeState::new(0.0, 10.0, 0.0, 0.06),
                NodeState::new(0.0, 60.0, 0.0, 0.2),
                NodeState::new(0.0, 40.0, 0.0, 0.6),
                NodeState::new(0.0, 10.0, 0.0, 0.6),
                NodeState::new(0.0, 20.0, 0.0, 0.1),
            ],
            vec![
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
            ],
            vec![CapacityModifier::new(); 3],
        );

        let current_snapshot = Snapshot::new(
            6,
            vec![
                NodeState::new(0.0, 22.0, 0.0, 0.93),
                NodeState::new(0.0, 10.0, 0.0, 0.07),
                NodeState::new(0.0, 66.0, 0.0, 0.17),
                NodeState::new(0.0, 40.0, 0.0, 0.6),
                NodeState::new(0.0, 10.0, 0.0, 0.6),
                NodeState::new(0.0, 16.0, 0.0, 0.1),
            ],
            vec![
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
            ],
            vec![CapacityModifier::new(); 3],
        );

        let summaries = aggregate_groups(&groupset, &current_snapshot, &previous_snapshot, &graph);

        // delta ~ 0.02
        assert_eq!(GroupTrend::Flat, *summaries[0].utilization_trend());
        // delta > 0.02
        assert_eq!(GroupTrend::Up, *summaries[1].utilization_trend());
        // delta < -0.02
        assert_eq!(GroupTrend::Down, *summaries[2].utilization_trend());
    }

    #[test]
    fn test_health_classification_at_boundaries() {
        let api1 = Node::new(NodeId(0), "api1".to_string(), 100.0, 1.0);
        let db1 = Node::new(NodeId(1), "db1".to_string(), 60.0, 1.0);
        let link1 = Edge::new(EdgeId(0), NodeId(0), NodeId(1), 1.0);

        let api2 = Node::new(NodeId(2), "api2".to_string(), 200.0, 1.0);
        let db2 = Node::new(NodeId(3), "db2".to_string(), 60.0, 1.0);
        let link2 = Edge::new(EdgeId(1), NodeId(2), NodeId(3), 2.0);

        let api3 = Node::new(NodeId(4), "api3".to_string(), 200.0, 1.0);
        let db3 = Node::new(NodeId(5), "db3".to_string(), 60.0, 1.0);
        let link3 = Edge::new(EdgeId(2), NodeId(4), NodeId(5), 2.0);

        let api4 = Node::new(NodeId(6), "api4".to_string(), 200.0, 1.0);
        let db4 = Node::new(NodeId(7), "db4".to_string(), 60.0, 1.0);
        let link4 = Edge::new(EdgeId(3), NodeId(6), NodeId(7), 2.0);

        let graph = Graph::new(
            vec![api1, db1, api2, db2, api3, db3, api4, db4],
            vec![link1, link2, link3, link4],
        );

        let groupset = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0), NodeId(1)]),
            Group::new("group2".to_string(), vec![NodeId(2), NodeId(3)]),
            Group::new("group3".to_string(), vec![NodeId(4), NodeId(5)]),
            Group::new("group4".to_string(), vec![NodeId(6), NodeId(7)]),
        ]);

        let previous_snapshot = Snapshot::new(
            5,
            vec![
                NodeState::new(0.0, 20.0, 0.0, 0.9),
                NodeState::new(0.0, 10.0, 0.0, 0.8),
                NodeState::new(0.0, 20.0, 0.0, 0.1),
                NodeState::new(0.0, 10.0, 0.0, 0.1),
                NodeState::new(0.0, 20.0, 0.0, 0.9),
                NodeState::new(0.0, 10.0, 0.0, 0.8),
                NodeState::new(0.0, 20.0, 0.0, 0.9),
                NodeState::new(0.0, 10.0, 0.0, 0.8),
            ],
            vec![
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
            ],
            vec![CapacityModifier::new(); 4],
        );

        let current_snapshot = Snapshot::new(
            6,
            vec![
                NodeState::new(0.0, 20.0, 0.0, 0.9),
                NodeState::new(0.0, 10.0, 0.0, 0.8),
                NodeState::new(0.0, 20.0, 0.0, 0.4),
                NodeState::new(0.0, 10.0, 0.0, 0.2),
                NodeState::new(0.0, 20.0, 0.0, 0.1),
                NodeState::new(0.0, 10.0, 0.0, 0.05),
                NodeState::new(0.0, 20.0, 0.0, 0.0),
                NodeState::new(0.0, 10.0, 0.0, 0.0),
            ],
            vec![
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
            ],
            vec![CapacityModifier::new(); 4],
        );

        let summaries = aggregate_groups(&groupset, &current_snapshot, &previous_snapshot, &graph);

        // 0.85
        assert_eq!(GroupHealth::Ok, *summaries[0].health());
        assert_eq!(GroupTrend::Flat, *summaries[0].health_trend());
        // 0.3
        assert_eq!(GroupHealth::Degraded, *summaries[1].health());
        assert_eq!(GroupTrend::Up, *summaries[1].health_trend());
        // 0.2
        assert_eq!(GroupHealth::Critical, *summaries[2].health());
        assert_eq!(GroupTrend::Down, *summaries[2].health_trend());
        // 0.0
        assert_eq!(GroupHealth::Failed, *summaries[3].health());
        assert_eq!(GroupTrend::Down, *summaries[3].health_trend());
    }

    // api0  ← api1    api2
    //     ↘    ↓     ↙
    //          db0
    //       ↙   ↻
    //   api3
    #[test]
    fn test_pressure_grouping() {
        let api0 = Node::new(NodeId(0), "api0".to_string(), 100.0, 1.0);
        let api1 = Node::new(NodeId(1), "api1".to_string(), 100.0, 1.0);
        let api2 = Node::new(NodeId(2), "api2".to_string(), 100.0, 1.0);
        let api3 = Node::new(NodeId(3), "api3".to_string(), 100.0, 1.0);
        let db0 = Node::new(NodeId(4), "db0".to_string(), 100.0, 1.0);

        let link1 = Edge::new(EdgeId(0), NodeId(0), NodeId(4), 1.0);
        let link2 = Edge::new(EdgeId(1), NodeId(1), NodeId(4), 1.0);
        let link3 = Edge::new(EdgeId(2), NodeId(2), NodeId(4), 2.0);
        let link4 = Edge::new(EdgeId(3), NodeId(4), NodeId(3), 2.0);
        let link5 = Edge::new(EdgeId(4), NodeId(4), NodeId(4), 1.0);
        let link6 = Edge::new(EdgeId(5), NodeId(1), NodeId(0), 1.0);

        let graph = Graph::new(
            vec![api0, api1, api2, api3, db0],
            vec![link1, link2, link3, link4, link5, link6],
        );

        let groupset = GroupSet::new(vec![
            Group::new("group1".to_string(), vec![NodeId(0), NodeId(3)]),
            Group::new("group2".to_string(), vec![NodeId(1), NodeId(2)]),
            Group::new("group3".to_string(), vec![NodeId(4)]),
        ]);

        let previous_snapshot = Snapshot::new(
            5,
            vec![
                NodeState::new(20.0, 20.0, 0.0, 0.9),
                NodeState::new(10.0, 10.0, 0.0, 0.8),
                NodeState::new(20.0, 20.0, 0.0, 0.1),
                NodeState::new(10.0, 10.0, 0.0, 0.1),
                NodeState::new(20.0, 20.0, 0.0, 0.9),
            ],
            vec![
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
            ],
            vec![CapacityModifier::new(); 6],
        );

        let current_snapshot = Snapshot::new(
            6,
            vec![
                NodeState::new(30.0, 30.0, 0.0, 0.9),
                NodeState::new(20.0, 20.0, 0.0, 0.8),
                NodeState::new(30.0, 30.0, 0.0, 0.3),
                NodeState::new(20.0, 20.0, 0.0, 0.2),
                NodeState::new(30.0, 30.0, 0.0, 0.1),
            ],
            vec![
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
                EdgeState::new(true),
            ],
            vec![CapacityModifier::new(); 6],
        );

        let summaries = aggregate_groups(&groupset, &current_snapshot, &previous_snapshot, &graph);
        let pressure = summaries[2].pressure();

        assert_relative_eq!(30.0, pressure[0]);
        assert_relative_eq!(40.0, pressure[1]);
        assert_relative_eq!(10.0, pressure[2]);
    }
}
