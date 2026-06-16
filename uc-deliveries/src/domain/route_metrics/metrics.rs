//! Route metric calculation shared by delivery constraints and previews.
//!
//! SolverForge mutates delivery order lists. This module walks those lists in
//! route order and turns them into arrival times, capacity usage, lateness, and
//! travel totals that multiple constraints can score consistently.

use solverforge_maps::UNREACHABLE;

use crate::domain::{Plan, Vehicle};

use super::helpers::{normalized_distance, normalized_travel_time};
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
            leg_from_previous(vehicle, previous_delivery_id, delivery_id);
        metrics.total_travel_seconds += travel_seconds;
        metrics.total_distance_meters += travel_meters;
        metrics.unreachable_legs += usize::from(unreachable);

        // Time windows are modeled around the moment service finishes. Waiting
        // is allowed, but leaving after `max_end_time` contributes lateness.
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
        let (return_seconds, return_meters, unreachable) = leg_to_depot(vehicle, last_delivery_id);
        metrics.total_travel_seconds += return_seconds;
        metrics.total_distance_meters += return_meters;
        metrics.unreachable_legs += usize::from(unreachable);
        metrics.end_time = metrics.end_time.saturating_add(return_seconds);
    }

    metrics.capacity_overage = (metrics.total_demand - vehicle.capacity).max(0);
    metrics
}

fn leg_from_previous(
    vehicle: &Vehicle,
    previous_delivery_id: Option<usize>,
    current_delivery_id: usize,
) -> (i64, i64, bool) {
    match previous_delivery_id {
        Some(previous_delivery_id) => prepared_delivery_leg(vehicle, previous_delivery_id, current_delivery_id),
        None => prepared_depot_leg(vehicle, current_delivery_id),
    }
}

fn prepared_delivery_leg(vehicle: &Vehicle, from_delivery_id: usize, to_delivery_id: usize) -> (i64, i64, bool) {
    if let Some(prepared) = &vehicle.prepared_routing {
        let seconds = prepared.travel_times[from_delivery_id][to_delivery_id];
        let meters = prepared.distance_matrix[from_delivery_id][to_delivery_id];
        return (
            normalized_travel_time(seconds),
            normalized_distance(meters),
            seconds == UNREACHABLE,
        );
    }

    (0, 0, false)
}

fn prepared_depot_leg(vehicle: &Vehicle, delivery_id: usize) -> (i64, i64, bool) {
    if let Some(prepared) = &vehicle.prepared_routing {
        let seconds = prepared.depot_to_delivery_seconds[delivery_id];
        let meters = prepared.depot_to_delivery_meters[delivery_id];
        return (
            normalized_travel_time(seconds),
            normalized_distance(meters),
            seconds == UNREACHABLE,
        );
    }

    (0, 0, false)
}

fn leg_to_depot(vehicle: &Vehicle, delivery_id: usize) -> (i64, i64, bool) {
    if let Some(prepared) = &vehicle.prepared_routing {
        let seconds = prepared.delivery_to_depot_seconds[delivery_id];
        let meters = prepared.delivery_to_depot_meters[delivery_id];
        return (
            normalized_travel_time(seconds),
            normalized_distance(meters),
            seconds == UNREACHABLE,
        );
    }

    (0, 0, false)
}
