use solverforge_maps::UNREACHABLE;

use crate::domain::{Plan, Vehicle};

use super::helpers::{normalized_distance, normalized_travel_time, straight_line_leg};
use super::types::{RouteStopMetrics, VehicleRouteMetrics};

pub(super) fn metrics_for_vehicle(plan: &Plan, vehicle: &Vehicle) -> VehicleRouteMetrics {
    let mut metrics = VehicleRouteMetrics {
        vehicle_id: vehicle.id,
        start_time: vehicle.departure_time,
        end_time: vehicle.departure_time,
        ..VehicleRouteMetrics::default()
    };

    let mut current_time = vehicle.departure_time;
    let mut previous_delivery_id = None;

    for (sequence, &delivery_id) in vehicle.delivery_order.iter().enumerate() {
        let Some(delivery) = plan.deliveries.get(delivery_id) else {
            continue;
        };

        metrics.total_demand += delivery.demand;
        let (travel_seconds, travel_meters, unreachable) =
            leg_from_previous(plan, vehicle, previous_delivery_id, delivery_id);
        metrics.total_travel_seconds += travel_seconds;
        metrics.total_distance_meters += travel_meters;
        metrics.unreachable_legs += usize::from(unreachable);

        let arrival_time = current_time.saturating_add(travel_seconds);
        let service_start_time = arrival_time.max(delivery.min_start_time);
        let wait_seconds = service_start_time.saturating_sub(arrival_time);
        let departure_time = service_start_time.saturating_add(delivery.service_duration);
        let late_seconds = departure_time.saturating_sub(delivery.max_end_time).max(0);

        metrics.total_wait_seconds += wait_seconds;
        metrics.total_service_seconds += delivery.service_duration;
        metrics.total_late_seconds += late_seconds;
        metrics.end_time = departure_time;
        metrics.stops.push(RouteStopMetrics {
            delivery_id,
            sequence,
            arrival_time,
            service_start_time,
            departure_time,
            travel_seconds_from_previous: travel_seconds,
            wait_seconds,
            late_seconds,
        });

        current_time = departure_time;
        previous_delivery_id = Some(delivery_id);
    }

    if let Some(last_delivery_id) = previous_delivery_id {
        let (return_seconds, return_meters, unreachable) =
            leg_to_depot(plan, vehicle, last_delivery_id);
        metrics.total_travel_seconds += return_seconds;
        metrics.total_distance_meters += return_meters;
        metrics.unreachable_legs += usize::from(unreachable);
        metrics.end_time = metrics.end_time.saturating_add(return_seconds);
    }

    metrics.capacity_overage = (metrics.total_demand - vehicle.capacity).max(0);
    metrics
}

fn leg_from_previous(
    plan: &Plan,
    vehicle: &Vehicle,
    previous_delivery_id: Option<usize>,
    current_delivery_id: usize,
) -> (i64, i64, bool) {
    match previous_delivery_id {
        Some(previous_delivery_id) => prepared_or_straight_line_delivery_leg(
            plan,
            vehicle,
            previous_delivery_id,
            current_delivery_id,
        ),
        None => prepared_or_straight_line_depot_leg(plan, vehicle, current_delivery_id),
    }
}

fn prepared_or_straight_line_delivery_leg(
    plan: &Plan,
    vehicle: &Vehicle,
    from_delivery_id: usize,
    to_delivery_id: usize,
) -> (i64, i64, bool) {
    if let Some(prepared) = &vehicle.prepared_routing {
        let seconds = prepared.travel_times[from_delivery_id][to_delivery_id];
        let meters = prepared.distance_matrix[from_delivery_id][to_delivery_id];
        return (
            normalized_travel_time(seconds),
            normalized_distance(meters),
            seconds == UNREACHABLE,
        );
    }

    let from = plan.deliveries[from_delivery_id].coord().expect("valid coord");
    let to = plan.deliveries[to_delivery_id].coord().expect("valid coord");
    let (seconds, meters) = straight_line_leg(from, to);
    (seconds, meters, false)
}

fn prepared_or_straight_line_depot_leg(
    plan: &Plan,
    vehicle: &Vehicle,
    delivery_id: usize,
) -> (i64, i64, bool) {
    if let Some(prepared) = &vehicle.prepared_routing {
        let seconds = prepared.depot_to_delivery_seconds[delivery_id];
        let meters = prepared.depot_to_delivery_meters[delivery_id];
        return (
            normalized_travel_time(seconds),
            normalized_distance(meters),
            seconds == UNREACHABLE,
        );
    }

    let from = vehicle.depot_coord().expect("valid coord");
    let to = plan.deliveries[delivery_id].coord().expect("valid coord");
    let (seconds, meters) = straight_line_leg(from, to);
    (seconds, meters, false)
}

fn leg_to_depot(plan: &Plan, vehicle: &Vehicle, delivery_id: usize) -> (i64, i64, bool) {
    if let Some(prepared) = &vehicle.prepared_routing {
        let seconds = prepared.delivery_to_depot_seconds[delivery_id];
        let meters = prepared.delivery_to_depot_meters[delivery_id];
        return (
            normalized_travel_time(seconds),
            normalized_distance(meters),
            seconds == UNREACHABLE,
        );
    }

    let from = plan.deliveries[delivery_id].coord().expect("valid coord");
    let to = vehicle.depot_coord().expect("valid coord");
    let (seconds, meters) = straight_line_leg(from, to);
    (seconds, meters, false)
}
