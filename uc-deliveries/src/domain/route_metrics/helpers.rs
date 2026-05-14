use solverforge_maps::{haversine_distance, BoundingBox, Coord, RoutingError, UNREACHABLE};

use crate::domain::{Delivery, Plan};

use super::types::RouteBounds;

const DEFAULT_SPEED_KMPH: f64 = 50.0;

pub(super) fn meters_to_seconds(meters: i64) -> i64 {
    let meters_per_second = DEFAULT_SPEED_KMPH * 1000.0 / 3600.0;
    (meters as f64 / meters_per_second).round() as i64
}

pub(super) fn normalized_travel_time(seconds: i64) -> i64 {
    if seconds == UNREACHABLE {
        super::types::UNREACHABLE_HARD_PENALTY
    } else {
        seconds.max(0)
    }
}

pub(super) fn normalized_distance(meters: i64) -> i64 {
    if meters == UNREACHABLE {
        0
    } else {
        meters.max(0)
    }
}

pub(super) fn build_delivery_distance_matrix(delivery_coords: &[Coord]) -> Vec<Vec<i64>> {
    delivery_coords
        .iter()
        .map(|from| {
            delivery_coords
                .iter()
                .map(|to| haversine_distance(*from, *to).round() as i64)
                .collect()
        })
        .collect()
}

pub(super) fn build_travel_time_matrix(delivery_coords: &[Coord]) -> Vec<Vec<i64>> {
    delivery_coords
        .iter()
        .map(|from| {
            delivery_coords
                .iter()
                .map(|to| meters_to_seconds(haversine_distance(*from, *to).round() as i64))
                .collect()
        })
        .collect()
}

pub(super) fn delivery_coords(plan: &Plan) -> Result<Vec<Coord>, RoutingError> {
    plan.deliveries
        .iter()
        .map(Delivery::coord)
        .collect::<Result<Vec<_>, _>>()
}

pub(super) fn route_bounds(plan: &Plan) -> Result<Option<RouteBounds>, RoutingError> {
    if plan.deliveries.is_empty() && plan.vehicles.is_empty() {
        return Ok(None);
    }

    let mut coords = delivery_coords(plan)?;
    for vehicle in &plan.vehicles {
        coords.push(vehicle.depot_coord()?);
    }
    let bbox = BoundingBox::from_coords(&coords);
    Ok(Some(RouteBounds {
        south_west: [bbox.min_lat, bbox.min_lng],
        north_east: [bbox.max_lat, bbox.max_lng],
    }))
}

pub(super) fn straight_line_leg(from: Coord, to: Coord) -> (i64, i64) {
    let meters = haversine_distance(from, to).round() as i64;
    (meters_to_seconds(meters), meters)
}

pub(super) fn fallback_vehicle_to_delivery(
    plan: &Plan,
    vehicle_idx: usize,
    delivery_id: usize,
) -> Option<i64> {
    let depot = plan.vehicles.get(vehicle_idx)?.depot_coord().ok()?;
    let delivery = plan.deliveries.get(delivery_id)?.coord().ok()?;
    Some(straight_line_leg(depot, delivery).0)
}

pub(super) fn fallback_delivery_to_vehicle(
    plan: &Plan,
    vehicle_idx: usize,
    delivery_id: usize,
) -> Option<i64> {
    let delivery = plan.deliveries.get(delivery_id)?.coord().ok()?;
    let depot = plan.vehicles.get(vehicle_idx)?.depot_coord().ok()?;
    Some(straight_line_leg(delivery, depot).0)
}

pub(super) fn fallback_delivery_to_delivery(plan: &Plan, from: usize, to: usize) -> Option<i64> {
    let from = plan.deliveries.get(from)?.coord().ok()?;
    let to = plan.deliveries.get(to)?.coord().ok()?;
    Some(straight_line_leg(from, to).0)
}
