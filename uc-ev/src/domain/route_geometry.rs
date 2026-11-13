//! Road geometry for drawing one solution on the map.
//!
//! Scoring never needs geometry, only the frozen matrices. The browser does:
//! this module turns each route's timed events (depot, customers, chargers)
//! into road polylines for `GET /jobs/{id}/routes`.

use serde::Serialize;
use solverforge_maps::{encode_polyline, BoundingBox, Coord, NetworkConfig, RoadNetwork, RoutingError};

use super::prepare::plan_coords;
use super::route_walk::RouteEventKind;
use super::{Plan, Vehicle};

/// Geometry for every vehicle route in one snapshot.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesSnapshot {
    pub vehicles: Vec<VehicleGeometry>,
}

/// One vehicle's road legs, in driving order.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleGeometry {
    pub vehicle_id: String,
    pub legs: Vec<LegGeometry>,
}

/// One driven leg between two route events.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegGeometry {
    /// `true` when the leg drives to or away from a charger.
    pub charging_detour: bool,
    pub distance_meters: i64,
    pub duration_seconds: i64,
    pub encoded_polyline: String,
}

/// Routes every vehicle's legs over the road network.
pub async fn build_routes_snapshot(plan: &Plan) -> Result<RoutesSnapshot, RoutingError> {
    let coords = plan_coords(plan)?;
    if coords.is_empty() {
        return Ok(RoutesSnapshot::default());
    }
    let bbox = BoundingBox::from_coords(&coords).expand_for_routing(&coords);
    let network = RoadNetwork::load_or_fetch(&bbox, &NetworkConfig::default(), None).await?;

    let mut vehicles = Vec::with_capacity(plan.vehicles.len());
    for vehicle in &plan.vehicles {
        vehicles.push(vehicle_geometry(plan, &network, vehicle)?);
    }
    Ok(RoutesSnapshot { vehicles })
}

fn vehicle_geometry(
    plan: &Plan,
    network: &RoadNetwork,
    vehicle: &Vehicle,
) -> Result<VehicleGeometry, RoutingError> {
    let depot = vehicle.depot_coord()?;
    let mut stops: Vec<(Coord, bool)> = Vec::with_capacity(vehicle.schedule.len());
    for event in &vehicle.schedule {
        let coord = match (event.kind, event.index) {
            (RouteEventKind::Visit, Some(idx)) => plan.customers[idx].coord()?,
            (RouteEventKind::Charge, Some(idx)) => plan.chargers[idx].coord()?,
            _ => depot,
        };
        stops.push((coord, event.kind == RouteEventKind::Charge));
    }

    let mut legs = Vec::with_capacity(stops.len().saturating_sub(1));
    for pair in stops.windows(2) {
        let ((from, from_charger), (to, to_charger)) = (pair[0], pair[1]);
        let route = network.route(from, to)?;
        let mut points = Vec::with_capacity(route.geometry.len() + 2);
        points.push(from);
        points.extend(route.geometry.iter().copied());
        points.push(to);
        points.dedup();
        legs.push(LegGeometry {
            charging_detour: from_charger || to_charger,
            distance_meters: route.distance_meters.round() as i64,
            duration_seconds: route.duration_seconds,
            encoded_polyline: encode_polyline(&points),
        });
    }

    Ok(VehicleGeometry {
        vehicle_id: vehicle.id.clone(),
        legs,
    })
}
