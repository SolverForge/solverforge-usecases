use super::*;
use crate::domain::{Delivery, DeliveryKind, Plan, Vehicle};
use solverforge::cvrp::ProblemData;
use std::sync::Arc;

fn sample_plan() -> Plan {
    let deliveries = vec![
        Delivery::new(
            0,
            "A",
            DeliveryKind::Business,
            (43.7696, 11.2558),
            3,
            (8 * 3600, 12 * 3600),
            10 * 60,
        ),
        Delivery::new(
            1,
            "B",
            DeliveryKind::Residential,
            (43.7710, 11.2620),
            2,
            (9 * 3600, 18 * 3600),
            5 * 60,
        ),
        Delivery::new(
            2,
            "C",
            DeliveryKind::Restaurant,
            (43.7755, 11.2540),
            4,
            (6 * 3600, 10 * 3600),
            15 * 60,
        ),
    ];
    let mut vehicles = vec![
        Vehicle::new(0, "Alpha", 10, 43.7696, 11.2558, 6 * 3600),
        Vehicle::new(1, "Bravo", 8, 43.7745, 11.2487, 6 * 3600),
    ];
    vehicles[0].delivery_order = vec![0, 1];
    vehicles[1].delivery_order = vec![2];
    Plan::new("Sample", deliveries, vehicles)
}

#[tokio::test]
async fn prepare_plan_handles_empty_road_network_data() {
    let mut plan = Plan::new("Empty", Vec::new(), Vec::new());
    prepare_plan(&mut plan)
        .await
        .expect("empty road-network prep should succeed");
    assert!(plan.prepared_problem_data.is_empty());
    assert!(plan.vehicles.is_empty());
}

#[test]
fn prepared_routing_wires_public_cvrp_depot_hooks() {
    let mut plan = sample_plan();
    attach_sample_routing(&mut plan);

    let vehicle_idx = 1;
    let depot = solverforge::cvrp::depot_for_entity(&plan, vehicle_idx);
    let prepared = plan.vehicles[vehicle_idx]
        .prepared_routing
        .as_ref()
        .expect("vehicle should have prepared routing");

    assert_eq!(depot, plan.deliveries.len());
    assert_eq!(
        solverforge::cvrp::route_distance(&plan, vehicle_idx, depot, 0),
        prepared.depot_to_delivery_seconds[0]
    );
    assert_eq!(
        solverforge::cvrp::route_distance(&plan, vehicle_idx, 0, depot),
        prepared.delivery_to_depot_seconds[0]
    );
    assert_eq!(
        solverforge::cvrp::route_distance(&plan, vehicle_idx, 0, 1),
        prepared.travel_times[0][1]
    );
}

#[test]
fn stock_clarke_wright_hooks_use_owner_specific_cvrp_data() {
    let mut plan = sample_plan();
    attach_sample_routing(&mut plan);
    let depot = plan.deliveries.len();

    assert_ne!(
        solverforge::cvrp::savings_metric_class(&plan, 0),
        solverforge::cvrp::savings_metric_class(&plan, 1),
        "separate per-vehicle ProblemData keeps stock Clarke-Wright metric classes owner-specific"
    );
    assert_eq!(solverforge::cvrp::savings_hooks::depot(&plan, 1), depot);
    assert_ne!(
        solverforge::cvrp::route_distance(&plan, 0, depot, 0),
        solverforge::cvrp::route_distance(&plan, 1, depot, 0),
        "route-local CVRP helpers stay owner-specific"
    );
    assert_eq!(
        solverforge::cvrp::savings_hooks::distance(&plan, 0, depot, 0),
        300
    );
    assert_eq!(
        solverforge::cvrp::savings_hooks::distance(&plan, 1, depot, 0),
        301,
        "stock Clarke-Wright hooks use the target owner's depot legs"
    );
    assert_eq!(
        solverforge::cvrp::savings_hooks::distance(&plan, 1, 0, 1),
        100
    );
}

#[test]
fn stock_clarke_wright_hooks_use_available_owner_depot_legs_for_partial_resolve() {
    let mut plan = sample_plan();
    plan.vehicles[0].delivery_order = vec![0, 1];
    plan.vehicles[1].delivery_order.clear();
    attach_sample_routing(&mut plan);
    let depot = plan.deliveries.len();

    assert_eq!(
        solverforge::cvrp::savings_hooks::distance(&plan, 1, depot, 2),
        501,
        "remaining construction for an empty owner must use that owner's depot legs"
    );
    assert!(
        solverforge::cvrp::savings_hooks::distance(&plan, 1, depot, 2)
            != solverforge::cvrp::savings_hooks::distance(&plan, 0, depot, 2),
        "partial re-solve must not silently fall back to the first prepared vehicle"
    );
}

#[test]
fn stock_savings_feasibility_is_relaxed_while_route_feasibility_is_strict() {
    let mut plan = sample_plan();
    plan.vehicles[0].capacity = 4;
    attach_sample_routing(&mut plan);

    assert!(
        !solverforge::cvrp::route_hooks::feasible(&plan, 0, &[0, 1]),
        "route-local CVRP feasibility remains strict for k-opt/local route moves"
    );
    assert!(
        solverforge::cvrp::savings_hooks::feasible(&plan, 0, &[0, 1]),
        "Clarke-Wright construction admits scoreable capacity violations"
    );
    assert!(
        !solverforge::cvrp::savings_hooks::feasible(&plan, 0, &[plan.deliveries.len()]),
        "construction feasibility still rejects malformed delivery ids"
    );
}

#[test]
fn stock_cvrp_feasibility_does_not_prune_scoreable_capacity_violations() {
    let mut plan = sample_plan();
    plan.vehicles[0].capacity = 4;
    plan.vehicles[1].capacity = 9;
    attach_sample_routing(&mut plan);

    assert!(
        solverforge::cvrp::savings_hooks::feasible(&plan, 0, &[0, 1]),
        "construction should let scoring penalize routes that exceed the target vehicle"
    );
    assert!(
        solverforge::cvrp::savings_hooks::feasible(&plan, 1, &[0, 1, 2]),
        "construction should also allow routes that fit the target vehicle"
    );
}

fn attach_sample_routing(plan: &mut Plan) {
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
    let travel_times = vec![vec![0, 100, 200], vec![120, 0, 180], vec![220, 160, 0]];
    let distances = vec![
        vec![0, 1_000, 2_000],
        vec![1_200, 0, 1_800],
        vec![2_200, 1_600, 0],
    ];

    plan.prepared_problem_data.clear();
    for (vehicle_idx, vehicle) in plan.vehicles.iter_mut().enumerate() {
        let depot_to_delivery_seconds = vec![
            300 + vehicle_idx as i64,
            400 + vehicle_idx as i64,
            500 + vehicle_idx as i64,
        ];
        let delivery_to_depot_seconds = vec![
            350 + vehicle_idx as i64,
            450 + vehicle_idx as i64,
            550 + vehicle_idx as i64,
        ];
        let depot_to_delivery_meters = vec![
            3_000 + vehicle_idx as i64,
            4_000 + vehicle_idx as i64,
            5_000 + vehicle_idx as i64,
        ];
        let delivery_to_depot_meters = vec![
            3_500 + vehicle_idx as i64,
            4_500 + vehicle_idx as i64,
            5_500 + vehicle_idx as i64,
        ];
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
            depot_to_delivery_seconds,
            delivery_to_depot_seconds,
            depot_to_delivery_meters,
            delivery_to_depot_meters,
        });
    }
}

#[test]
fn preview_reports_assignments() {
    let plan = sample_plan();
    let preview = preview_for_plan(&plan);
    assert_eq!(preview.unassigned_delivery_ids.len(), 0);
    assert_eq!(preview.vehicles.len(), 2);
    assert_eq!(preview.deliveries[0].assigned_vehicle_id, Some(0));
}

#[test]
fn unassigned_deliveries_are_dominant_hard_penalties() {
    let mut plan = sample_plan();
    plan.vehicles[0].delivery_order.clear();
    plan.vehicles[1].delivery_order.clear();

    let components = evaluate_plan(&plan);

    assert_eq!(components.unassigned_count, 3);
    assert!(
        components.hard_score() <= -(3 * UNASSIGNED_DELIVERY_HARD_PENALTY),
        "unassigned deliveries must not be cheaper than seconds-based route violations"
    );
}

#[tokio::test]
async fn empty_road_network_routes_have_no_bounds_or_vehicles() {
    let plan = Plan::new("Empty", Vec::new(), Vec::new());

    let snapshot = build_routes_snapshot(&plan)
        .await
        .expect("empty road-network routes should not require a bounding box");

    assert_eq!(snapshot.routing_mode, "road_network");
    assert!(snapshot.bounds.is_none());
    assert!(snapshot.vehicles.is_empty());
}
