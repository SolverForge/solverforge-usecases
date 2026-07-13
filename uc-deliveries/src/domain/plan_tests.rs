use super::*;
use crate::domain::{DeliveryKind, UNASSIGNED_DELIVERY_HARD_PENALTY};
use solverforge::cvrp::ProblemData;
use solverforge::{ScoreDirector, SolverConfig, SolverEvent, SolverManager};
use std::sync::Arc;

fn tiny_plan() -> Plan {
    Plan::new(
        "tiny",
        vec![
            Delivery::new(
                0,
                "A",
                DeliveryKind::Residential,
                (39.9526, -75.1652),
                1,
                (8 * 3600, 18 * 3600),
                10 * 60,
            ),
            Delivery::new(
                1,
                "B",
                DeliveryKind::Business,
                (39.9626, -75.1752),
                1,
                (8 * 3600, 18 * 3600),
                10 * 60,
            ),
        ],
        vec![Vehicle::new(0, "Van 1", 4, 39.9526, -75.1652, 8 * 3600)],
    )
}

fn prepared_tiny_plan_with_route() -> Plan {
    let mut plan = tiny_plan();
    plan.vehicles[0].delivery_order = vec![0, 1];
    attach_test_routing(&mut plan);
    plan
}

fn prepared_tiny_plan() -> Plan {
    let mut plan = tiny_plan();
    attach_test_routing(&mut plan);
    plan
}

fn attach_test_routing(plan: &mut Plan) {
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
    let travel_times = vec![vec![0, 600], vec![660, 0]];
    let distances = vec![vec![0, 3_000], vec![3_300, 0]];
    let depot_to_delivery_seconds = vec![300, 900];
    let delivery_to_depot_seconds = vec![360, 840];
    let depot_to_delivery_meters = vec![1_500, 4_500];
    let delivery_to_depot_meters = vec![1_800, 4_200];

    plan.prepared_problem_data.clear();
    for (vehicle_idx, vehicle) in plan.vehicles.iter_mut().enumerate() {
        let mut problem_matrix = vec![vec![0_i64; delivery_count + 1]; delivery_count + 1];
        for (from, row) in travel_times.iter().enumerate() {
            for (to, seconds) in row.iter().copied().enumerate() {
                problem_matrix[from][to] = seconds;
            }
        }
        for (delivery_idx, seconds) in depot_to_delivery_seconds.iter().copied().enumerate() {
            problem_matrix[delivery_count][delivery_idx] = seconds;
        }
        for (delivery_idx, seconds) in delivery_to_depot_seconds.iter().copied().enumerate() {
            problem_matrix[delivery_idx][delivery_count] = seconds;
        }

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
            depot_to_delivery_seconds: depot_to_delivery_seconds.clone(),
            delivery_to_depot_seconds: delivery_to_depot_seconds.clone(),
            depot_to_delivery_meters: depot_to_delivery_meters.clone(),
            delivery_to_depot_meters: delivery_to_depot_meters.clone(),
        });
    }
}

#[test]
fn route_shadow_listener_populates_vehicle_route_shadows() {
    let mut plan = prepared_tiny_plan_with_route();
    assert_eq!(
        plan.vehicles[0].route_total_demand, 0,
        "prepared transport data should not eagerly populate solver shadows"
    );

    plan.refresh_vehicle_route_shadows(0);
    let vehicle = &plan.vehicles[0];

    assert_eq!(vehicle.total_assigned_demand(), 2);
    assert_eq!(vehicle.capacity_overage(), 0);
    assert!(
        vehicle.total_travel_seconds() > 0,
        "route travel should be maintained as a shadow value"
    );
}

#[test]
fn vehicle_route_shadows_refresh_after_list_variable_changes() {
    let plan = prepared_tiny_plan_with_route();
    let mut director = ScoreDirector::with_descriptor(
        plan,
        crate::constraints::create_constraints(),
        Plan::descriptor(),
        Plan::entity_count,
    );
    director.calculate_score();
    assert_eq!(
        director.working_solution().vehicles[0].total_assigned_demand(),
        2
    );

    director.before_variable_changed(0, 0);
    director.working_solution_mut().vehicles[0]
        .delivery_order
        .clear();
    director.after_variable_changed(0, 0);
    let score = director.calculate_score();

    let vehicle = &director.working_solution().vehicles[0];
    assert_eq!(vehicle.total_assigned_demand(), 0);
    assert_eq!(vehicle.total_travel_seconds(), 0);
    assert_eq!(vehicle.time_window_violation_seconds(), 0);
    assert_eq!(score.hard(), -(2 * UNASSIGNED_DELIVERY_HARD_PENALTY));
}

#[test]
fn generated_list_runtime_builds_routes() {
    static MANAGER: SolverManager<Plan> = SolverManager::new();

    let plan = prepared_tiny_plan();

    assert!(
        Plan::test_has_list_variable(),
        "delivery plan should expose a list variable"
    );
    assert_eq!(Plan::test_total_list_entities(&plan), 1);
    assert_eq!(Plan::test_total_list_elements(&plan), 2);
    let config =
        SolverConfig::from_toml_str(include_str!("../../solver.toml")).expect("valid config");
    assert_eq!(
        config.phases.len(),
        3,
        "expected Clarke-Wright construction + list k-opt + local search"
    );

    let (job_id, mut receiver) = MANAGER.solve(plan).expect("solve should start");
    let mut saw_non_empty_best = false;
    loop {
        match receiver
            .blocking_recv()
            .expect("event stream should reach a terminal event")
        {
            SolverEvent::BestSolution { solution, .. } => {
                if solution
                    .vehicles
                    .iter()
                    .any(|vehicle| !vehicle.delivery_order.is_empty())
                {
                    saw_non_empty_best = true;
                    MANAGER.cancel(job_id).expect("job cancel should succeed");
                }
            }
            SolverEvent::Completed { .. } | SolverEvent::Cancelled { .. } => break,
            SolverEvent::Failed { error, .. } => {
                panic!("solve unexpectedly failed: {error}");
            }
            SolverEvent::Progress { .. }
            | SolverEvent::PauseRequested { .. }
            | SolverEvent::Paused { .. }
            | SolverEvent::Resumed { .. } => {}
        }
    }
    MANAGER
        .delete(job_id)
        .expect("completed test job should delete");

    assert!(
        saw_non_empty_best,
        "expected a non-empty best solution before cancellation"
    );
}
