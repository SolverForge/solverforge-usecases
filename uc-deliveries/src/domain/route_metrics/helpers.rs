use solverforge_maps::{BoundingBox, Coord, RoutingError, UNREACHABLE};

use crate::domain::{Delivery, Plan};

use super::types::RouteBounds;

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
