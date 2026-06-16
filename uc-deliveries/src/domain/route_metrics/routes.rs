use solverforge_maps::{encode_polyline, BoundingBox, Coord, NetworkConfig, RoadNetwork, RoutingError};

use crate::domain::Plan;

use super::helpers::{delivery_coords, route_bounds};
use super::metrics::metrics_for_vehicle;
use super::types::{RouteLegGeometry, RouteLegSummary, RoutesSnapshot};

/// Builds browser route geometry for one already-selected solution snapshot.
///
/// This is separate from `prepare_plan`: preparation builds matrices for
/// scoring, while `/jobs/{id}/routes` turns the retained snapshot into encoded
/// map geometry the UI can draw.
pub async fn build_routes_snapshot(plan: &Plan) -> Result<RoutesSnapshot, RoutingError> {
    let bounds = route_bounds(plan)?;
    let vehicles = build_road_routes(plan).await?;

    Ok(RoutesSnapshot {
        routing_mode: "road_network".to_string(),
        bounds,
        vehicles,
    })
}

async fn build_road_routes(plan: &Plan) -> Result<Vec<RouteLegSummary>, RoutingError> {
    let mut coords = delivery_coords(plan)?;
    for vehicle in &plan.vehicles {
        coords.push(vehicle.depot_coord()?);
    }

    if coords.is_empty() {
        return Ok(Vec::new());
    }

    let bbox = BoundingBox::from_coords(&coords).expand_for_routing(&coords);
    let network = RoadNetwork::load_or_fetch(&bbox, &NetworkConfig::default(), None).await?;

    let mut routes = Vec::with_capacity(plan.vehicles.len());
    for vehicle in &plan.vehicles {
        routes.push(build_vehicle_road_route(plan, &network, vehicle).await?);
    }
    Ok(routes)
}

async fn build_vehicle_road_route(
    plan: &Plan,
    network: &RoadNetwork,
    vehicle: &crate::domain::Vehicle,
) -> Result<RouteLegSummary, RoutingError> {
    let metrics = metrics_for_vehicle(plan, vehicle);
    let mut segments = Vec::new();
    let mut total_distance_meters = 0_i64;
    let mut total_travel_seconds = 0_i64;
    let mut previous_coord = vehicle.depot_coord()?;
    let mut previous_id = None;

    for &delivery_id in &vehicle.delivery_order {
        let delivery = &plan.deliveries[delivery_id];
        let coord = delivery.coord()?;
        let route = network.route(previous_coord, coord)?;
        let distance_meters = route.distance_meters.round() as i64;
        total_distance_meters += distance_meters;
        total_travel_seconds += route.duration_seconds;
        segments.push(RouteLegGeometry {
            vehicle_id: vehicle.id,
            from_kind: if previous_id.is_some() {
                "delivery"
            } else {
                "depot"
            },
            from_id: previous_id,
            to_kind: "delivery",
            to_id: Some(delivery_id),
            duration_seconds: route.duration_seconds,
            distance_meters,
            encoded_polyline: encode_polyline(&route_geometry_with_road_endpoints(
                network,
                previous_coord,
                coord,
                &route.geometry,
            )?),
        });
        previous_coord = coord;
        previous_id = Some(delivery_id);
    }

    if let Some(last_delivery_id) = previous_id {
        let depot = vehicle.depot_coord()?;
        let route = network.route(previous_coord, depot)?;
        let distance_meters = route.distance_meters.round() as i64;
        total_distance_meters += distance_meters;
        total_travel_seconds += route.duration_seconds;
        segments.push(RouteLegGeometry {
            vehicle_id: vehicle.id,
            from_kind: "delivery",
            from_id: Some(last_delivery_id),
            to_kind: "depot",
            to_id: None,
            duration_seconds: route.duration_seconds,
            distance_meters,
            encoded_polyline: encode_polyline(&route_geometry_with_road_endpoints(
                network,
                previous_coord,
                depot,
                &route.geometry,
            )?),
        });
    }

    Ok(RouteLegSummary {
        vehicle_id: vehicle.id,
        vehicle_name: vehicle.name.clone(),
        total_travel_seconds,
        total_distance_meters,
        total_demand: metrics.total_demand,
        total_late_seconds: metrics.total_late_seconds,
        stop_count: vehicle.delivery_order.len(),
        segments,
    })
}

fn route_geometry_with_road_endpoints(
    network: &RoadNetwork,
    from: Coord,
    to: Coord,
    network_geometry: &[Coord],
) -> Result<Vec<Coord>, RoutingError> {
    let from_snap = network.snap_to_edge(from)?;
    let to_snap = network.snap_to_edge(to)?;
    Ok(route_geometry_with_endpoints(
        from,
        Some(from_snap.snapped),
        to,
        Some(to_snap.snapped),
        network_geometry,
    ))
}

fn route_geometry_with_endpoints(
    from: Coord,
    from_road: Option<Coord>,
    to: Coord,
    to_road: Option<Coord>,
    network_geometry: &[Coord],
) -> Vec<Coord> {
    let mut geometry = Vec::with_capacity(network_geometry.len() + 4);
    push_unique_coord(&mut geometry, from);
    if let Some(coord) = from_road {
        push_unique_coord(&mut geometry, coord);
    }
    for &coord in network_geometry {
        push_unique_coord(&mut geometry, coord);
    }
    if let Some(coord) = to_road {
        push_unique_coord(&mut geometry, coord);
    }
    push_unique_coord(&mut geometry, to);
    geometry
}

fn push_unique_coord(geometry: &mut Vec<Coord>, coord: Coord) {
    if geometry.last().copied() != Some(coord) {
        geometry.push(coord);
    }
}

#[cfg(test)]
mod tests;
