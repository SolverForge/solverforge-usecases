use super::*;
use crate::data::{generate, DemoData};
use solverforge::cvrp::ProblemData;
use solverforge::SolverConfig;
use std::collections::BTreeSet;
use std::sync::Arc;

fn attach_synthetic_routing(plan: &mut Plan) {
    let delivery_count = plan.deliveries.len();
    let demands = plan
        .deliveries
        .iter()
        .map(|delivery| delivery.demand)
        .collect::<Vec<_>>();
    let time_windows = plan
        .deliveries
        .iter()
        .map(|delivery| (delivery.min_start_time, delivery.max_end_time))
        .collect::<Vec<_>>();
    let service_durations = plan
        .deliveries
        .iter()
        .map(|delivery| delivery.service_duration)
        .collect::<Vec<_>>();
    let travel_times = matrix(delivery_count, 300, 45);
    let distances = matrix(delivery_count, 1_000, 150);

    plan.prepared_problem_data.clear();
    for (vehicle_idx, vehicle) in plan.vehicles.iter_mut().enumerate() {
        let depot_to_delivery_seconds = depot_legs(delivery_count, vehicle_idx, 600, 10);
        let delivery_to_depot_seconds = depot_legs(delivery_count, vehicle_idx, 630, 10);
        let depot_to_delivery_meters = depot_legs(delivery_count, vehicle_idx, 2_000, 20);
        let delivery_to_depot_meters = depot_legs(delivery_count, vehicle_idx, 2_100, 20);
        let problem_matrix = problem_matrix(
            delivery_count,
            &travel_times,
            &depot_to_delivery_seconds,
            &delivery_to_depot_seconds,
        );

        plan.prepared_problem_data.push(Arc::new(ProblemData {
            capacity: vehicle.capacity as i64,
            depot: delivery_count,
            demands: demands.clone(),
            distance_matrix: problem_matrix.clone(),
            time_windows: time_windows.clone(),
            service_durations: service_durations.clone(),
            travel_times: problem_matrix,
            vehicle_departure_time: vehicle.departure_time,
        }));
        vehicle.prepared_routing = Some(PreparedVehicleRouting {
            problem_data_index: vehicle_idx,
            capacity: vehicle.capacity as i64,
            demands: demands.clone(),
            distance_matrix: distances.clone(),
            time_windows: time_windows.clone(),
            service_durations: service_durations.clone(),
            travel_times: travel_times.clone(),
            vehicle_departure_time: vehicle.departure_time,
            depot_to_delivery_seconds,
            delivery_to_depot_seconds,
            depot_to_delivery_meters,
            delivery_to_depot_meters,
        });
    }
}

fn matrix(size: usize, base: i64, step: i64) -> Vec<Vec<i64>> {
    (0..size)
        .map(|from| {
            (0..size)
                .map(|to| {
                    if from == to {
                        0
                    } else {
                        base + from.abs_diff(to) as i64 * step
                    }
                })
                .collect()
        })
        .collect()
}

fn depot_legs(size: usize, vehicle_idx: usize, base: i64, step: i64) -> Vec<i64> {
    (0..size)
        .map(|delivery_idx| base + delivery_idx as i64 * step + vehicle_idx as i64)
        .collect()
}

fn problem_matrix(
    delivery_count: usize,
    travel_times: &[Vec<i64>],
    depot_to_delivery_seconds: &[i64],
    delivery_to_depot_seconds: &[i64],
) -> Vec<Vec<i64>> {
    let mut matrix = vec![vec![0_i64; delivery_count + 1]; delivery_count + 1];
    for (from, row) in travel_times.iter().enumerate() {
        for (to, seconds) in row.iter().copied().enumerate() {
            matrix[from][to] = seconds;
        }
    }
    for (delivery_idx, seconds) in depot_to_delivery_seconds.iter().copied().enumerate() {
        matrix[delivery_count][delivery_idx] = seconds;
    }
    for (delivery_idx, seconds) in delivery_to_depot_seconds.iter().copied().enumerate() {
        matrix[delivery_idx][delivery_count] = seconds;
    }
    matrix
}

#[test]
fn clarke_wright_construction_assigns_full_philadelphia_fixture() {
    let mut plan = generate(DemoData::Philadelphia);
    attach_synthetic_routing(&mut plan);
    assert_eq!(plan.deliveries.len(), 82);

    let config = clarke_wright_only_config();
    let solved = Plan::test_solve_with_config(plan, &config);
    assert_all_deliveries_assigned(&solved, 82);
}

#[test]
fn construction_policy_assigns_full_philadelphia_fixture() {
    let mut plan = generate(DemoData::Philadelphia);
    attach_synthetic_routing(&mut plan);
    assert_eq!(plan.deliveries.len(), 82);

    let config = clarke_wright_then_k_opt_config();
    let solved = Plan::test_solve_with_config(plan, &config);
    assert_all_deliveries_assigned(&solved, 82);
}

#[test]
fn clarke_wright_assigns_over_capacity_delivery_for_scoring() {
    let mut plan = single_delivery_plan(20, 5, (0, 86_400), 60);
    attach_synthetic_routing(&mut plan);
    let unassigned_hard_score = evaluate_plan(&plan).hard_score();

    let config = clarke_wright_only_config();
    let solved = Plan::test_solve_with_config(plan, &config);
    let components = evaluate_plan(&solved);

    assert_all_deliveries_assigned(&solved, 1);
    assert!(components.capacity_overage > 0);
    assert!(
        components.hard_score() > unassigned_hard_score,
        "capacity overage must be scored as a better assignment than leaving the delivery unassigned"
    );
}

#[test]
fn clarke_wright_assigns_late_delivery_for_scoring() {
    let mut plan = single_delivery_plan(1, 10, (0, 100), 1_000);
    attach_synthetic_routing(&mut plan);
    let unassigned_hard_score = evaluate_plan(&plan).hard_score();

    let config = clarke_wright_only_config();
    let solved = Plan::test_solve_with_config(plan, &config);
    let components = evaluate_plan(&solved);

    assert_all_deliveries_assigned(&solved, 1);
    assert!(components.late_seconds > 0);
    assert!(
        components.hard_score() > unassigned_hard_score,
        "lateness must be scored as a better assignment than leaving the delivery unassigned"
    );
}

#[tokio::test]
async fn live_clarke_wright_construction_assigns_full_philadelphia_fixture_when_enabled() {
    if std::env::var("SOLVERFORGE_RUN_LIVE_TESTS").ok().as_deref() != Some("1") {
        return;
    }

    let mut plan = generate(DemoData::Philadelphia);
    prepare_plan(&mut plan)
        .await
        .expect("live road-network preparation should succeed");
    assert_eq!(plan.deliveries.len(), 82);

    let config = clarke_wright_only_config();
    let solved = Plan::test_solve_with_config(plan, &config);
    assert_all_deliveries_assigned(&solved, 82);
}

fn single_delivery_plan(
    demand: i32,
    capacity: i32,
    time_window: (i64, i64),
    service_duration: i64,
) -> Plan {
    Plan::new(
        "Single delivery",
        vec![Delivery::new(
            0,
            "Only stop",
            DeliveryKind::Business,
            (39.9526, -75.1652),
            demand,
            time_window,
            service_duration,
        )],
        vec![Vehicle::new(0, "Truck", capacity, 39.9520, -75.1640, 0)],
    )
}

fn clarke_wright_only_config() -> SolverConfig {
    SolverConfig::from_toml_str(
        r#"
environment_mode = "reproducible"
random_seed = 42

[[phases]]
type = "construction_heuristic"
construction_heuristic_type = "list_clarke_wright"
entity_class = "Vehicle"
variable_name = "delivery_order"
"#,
    )
    .expect("valid Clarke-Wright-only test config")
}

fn clarke_wright_then_k_opt_config() -> SolverConfig {
    SolverConfig::from_toml_str(
        r#"
environment_mode = "reproducible"
random_seed = 42

[[phases]]
type = "construction_heuristic"
construction_heuristic_type = "list_clarke_wright"
entity_class = "Vehicle"
variable_name = "delivery_order"

[[phases]]
type = "construction_heuristic"
construction_heuristic_type = "list_k_opt"
k = 2
entity_class = "Vehicle"
variable_name = "delivery_order"
"#,
    )
    .expect("valid Clarke-Wright plus k-opt test config")
}

fn assert_all_deliveries_assigned(plan: &Plan, expected_count: usize) {
    let assigned = plan
        .vehicles
        .iter()
        .flat_map(|vehicle| vehicle.delivery_order.iter().copied())
        .collect::<Vec<_>>();
    let unique = assigned.iter().copied().collect::<BTreeSet<_>>();

    assert_eq!(assigned.len(), expected_count);
    assert_eq!(unique.len(), expected_count);
}

#[test]
fn production_local_search_scans_until_score_improves() {
    let solver_toml = include_str!("../../solver.toml");

    assert!(
        solver_toml.contains("type = \"first_last_step_score_improving\""),
        "local search must keep scanning past equal accepted moves"
    );
    assert!(
        !solver_toml.contains("type = \"accepted_count\""),
        "accepted_count can stop after equal-score accepted moves before reaching an improvement"
    );
}
