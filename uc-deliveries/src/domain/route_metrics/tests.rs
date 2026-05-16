use super::*;
use crate::domain::{Delivery, DeliveryKind, Plan, RoutingMode, Vehicle};

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
async fn prepare_plan_populates_vehicle_routing_data() {
    let mut plan = sample_plan();
    plan.routing_mode = RoutingMode::StraightLine;
    prepare_plan(&mut plan)
        .await
        .expect("straight-line prep should succeed");
    assert!(plan
        .vehicles
        .iter()
        .all(|vehicle| vehicle.prepared_routing.is_some()));
}

#[tokio::test]
async fn prepare_plan_wires_public_cvrp_depot_hooks() {
    let mut plan = sample_plan();
    plan.routing_mode = RoutingMode::StraightLine;
    prepare_plan(&mut plan)
        .await
        .expect("straight-line prep should succeed");

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
    let mut plan = Plan::new("Empty", Vec::new(), Vec::new());
    plan.routing_mode = RoutingMode::RoadNetwork;

    let snapshot = build_routes_snapshot(&plan)
        .await
        .expect("empty road-network routes should not require a bounding box");

    assert_eq!(snapshot.routing_mode, "road_network");
    assert!(snapshot.bounds.is_none());
    assert!(snapshot.vehicles.is_empty());
}
