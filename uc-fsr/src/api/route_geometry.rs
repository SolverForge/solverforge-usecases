//! Browser route-geometry builder for FSR snapshots.
//!
//! Scoring uses cached `TravelLeg` facts so local search remains cheap. The
//! browser asks for drawable road geometry only for retained snapshots, and this
//! module converts those route legs into per-segment DTOs.

use axum::http::StatusCode;

use super::route_dto::{RouteGeometryStatus, RouteSegmentDto, TechnicianRouteGeometryDto};
use crate::data::{load_network, DemoDataError};
use crate::domain::{FieldServicePlan, TravelLeg};

pub(super) fn status_from_routing_error(error: solverforge_maps::RoutingError) -> StatusCode {
    eprintln!("Bergamo route geometry failed: {error}");
    match error {
        solverforge_maps::RoutingError::InvalidCoordinate { .. } => StatusCode::BAD_REQUEST,
        solverforge_maps::RoutingError::Cancelled => StatusCode::REQUEST_TIMEOUT,
        solverforge_maps::RoutingError::Network(_)
        | solverforge_maps::RoutingError::Parse(_)
        | solverforge_maps::RoutingError::Io(_)
        | solverforge_maps::RoutingError::SnapFailed { .. }
        | solverforge_maps::RoutingError::NoPath { .. } => StatusCode::BAD_GATEWAY,
    }
}

pub(super) async fn build_route_geometry(
    plan: &FieldServicePlan,
) -> Result<Vec<TechnicianRouteGeometryDto>, solverforge_maps::RoutingError> {
    // Geometry is built on demand for retained snapshots. It is separate from
    // scoring so local search can use cached matrix facts without asking the map
    // service to draw every candidate move.
    let network = load_network().await.map_err(|error| match error {
        DemoDataError::Routing(error) => error,
    })?;
    let mut routes = Vec::with_capacity(plan.technician_routes.len());

    for route in &plan.technician_routes {
        let mut segments = Vec::new();
        let mut previous_location_idx = route.start_location_idx;
        for &visit_idx in &route.visits {
            let Some(visit) = plan.service_visits.get(visit_idx) else {
                continue;
            };
            segments.push(build_route_segment(
                plan,
                &network,
                &route.id,
                previous_location_idx,
                visit.location_idx,
            )?);
            previous_location_idx = visit.location_idx;
        }
        if !route.visits.is_empty() {
            segments.push(build_route_segment(
                plan,
                &network,
                &route.id,
                previous_location_idx,
                route.end_location_idx,
            )?);
        }

        routes.push(TechnicianRouteGeometryDto {
            route_id: route.id.clone(),
            technician_id: route.technician_id.clone(),
            technician_name: route.technician_name.clone(),
            color: route.color.clone(),
            segments,
        });
    }

    Ok(routes)
}

fn build_route_segment(
    plan: &FieldServicePlan,
    network: &solverforge_maps::RoadNetwork,
    route_id: &str,
    from_location_idx: usize,
    to_location_idx: usize,
) -> Result<RouteSegmentDto, solverforge_maps::RoutingError> {
    let travel_leg = find_travel_leg(plan, from_location_idx, to_location_idx);
    if !travel_leg.is_some_and(|leg| leg.reachable) {
        // Preserve the segment in the response even when it cannot be drawn.
        // The UI can then show a partial route instead of hiding useful legs.
        return Ok(non_routed_segment(
            route_id,
            from_location_idx,
            to_location_idx,
            travel_leg,
            RouteGeometryStatus::UnreachableLeg,
        ));
    }

    let from = plan.locations.get(from_location_idx).ok_or_else(|| {
        solverforge_maps::RoutingError::Network("route source location missing".into())
    })?;
    let to = plan.locations.get(to_location_idx).ok_or_else(|| {
        solverforge_maps::RoutingError::Network("route target location missing".into())
    })?;

    let route_result = network.route(
        solverforge_maps::Coord::new(from.lat(), from.lng()),
        solverforge_maps::Coord::new(to.lat(), to.lng()),
    );
    let route = match route_result {
        Ok(route) => route.simplify(12.0),
        Err(error) => {
            // Snap and no-path failures are segment-level map problems. Treat
            // them as display status rather than failing the whole snapshot.
            if let Some(status) = recoverable_geometry_status(&error) {
                return Ok(non_routed_segment(
                    route_id,
                    from_location_idx,
                    to_location_idx,
                    travel_leg,
                    status,
                ));
            }
            return Err(error);
        }
    };

    Ok(RouteSegmentDto {
        route_id: route_id.to_string(),
        from_location_idx,
        to_location_idx,
        duration_seconds: route.duration_seconds,
        distance_meters: route.distance_meters.round() as i64,
        reachable: true,
        geometry_status: RouteGeometryStatus::Routed,
        encoded_polyline: solverforge_maps::encode_polyline(&route.geometry),
    })
}

fn find_travel_leg(
    plan: &FieldServicePlan,
    from_location_idx: usize,
    to_location_idx: usize,
) -> Option<&TravelLeg> {
    let width = plan.locations.len();
    plan.travel_legs
        .get(from_location_idx.checked_mul(width)? + to_location_idx)
        .filter(|leg| {
            leg.from_location_idx == from_location_idx && leg.to_location_idx == to_location_idx
        })
        .or_else(|| {
            plan.travel_legs.iter().find(|leg| {
                leg.from_location_idx == from_location_idx && leg.to_location_idx == to_location_idx
            })
        })
}

fn non_routed_segment(
    route_id: &str,
    from_location_idx: usize,
    to_location_idx: usize,
    travel_leg: Option<&TravelLeg>,
    geometry_status: RouteGeometryStatus,
) -> RouteSegmentDto {
    RouteSegmentDto {
        route_id: route_id.to_string(),
        from_location_idx,
        to_location_idx,
        duration_seconds: travel_leg.map_or(0, |leg| leg.duration_seconds),
        distance_meters: travel_leg.map_or(0, |leg| leg.distance_meters),
        reachable: false,
        geometry_status,
        encoded_polyline: String::new(),
    }
}

fn recoverable_geometry_status(
    error: &solverforge_maps::RoutingError,
) -> Option<RouteGeometryStatus> {
    match error {
        solverforge_maps::RoutingError::SnapFailed { .. } => Some(RouteGeometryStatus::SnapFailed),
        solverforge_maps::RoutingError::NoPath { .. } => Some(RouteGeometryStatus::NoPath),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FieldServicePlan, Location, TravelLegInit};

    #[test]
    fn finds_dense_or_sparse_travel_leg() {
        let plan = test_plan(vec![TravelLeg::new(TravelLegInit {
            id: "leg-01-02".to_string(),
            name: "leg-01-02".to_string(),
            from_location_idx: 1,
            to_location_idx: 2,
            duration_seconds: 42,
            distance_meters: 1000,
            reachable: true,
        })]);

        let leg = find_travel_leg(&plan, 1, 2).expect("travel leg");

        assert_eq!(leg.duration_seconds, 42);
    }

    #[test]
    fn non_routed_segment_preserves_known_metrics() {
        let plan = test_plan(vec![TravelLeg::new(TravelLegInit {
            id: "leg-00-01".to_string(),
            name: "leg-00-01".to_string(),
            from_location_idx: 0,
            to_location_idx: 1,
            duration_seconds: 90,
            distance_meters: 1200,
            reachable: false,
        })]);

        let segment = non_routed_segment(
            "route-00",
            0,
            1,
            find_travel_leg(&plan, 0, 1),
            RouteGeometryStatus::UnreachableLeg,
        );

        assert!(!segment.reachable);
        assert_eq!(segment.geometry_status, RouteGeometryStatus::UnreachableLeg);
        assert_eq!(segment.duration_seconds, 90);
        assert!(segment.encoded_polyline.is_empty());
    }

    #[test]
    fn only_snap_and_no_path_are_recoverable_segment_failures() {
        let from = solverforge_maps::Coord::new(45.0, 9.0);
        let to = solverforge_maps::Coord::new(46.0, 10.0);

        assert_eq!(
            recoverable_geometry_status(&solverforge_maps::RoutingError::NoPath { from, to }),
            Some(RouteGeometryStatus::NoPath)
        );
        assert_eq!(
            recoverable_geometry_status(&solverforge_maps::RoutingError::SnapFailed {
                coord: from,
                nearest_distance_m: None,
            }),
            Some(RouteGeometryStatus::SnapFailed)
        );
        assert_eq!(
            recoverable_geometry_status(&solverforge_maps::RoutingError::Network("down".into())),
            None
        );
    }

    fn test_plan(travel_legs: Vec<TravelLeg>) -> FieldServicePlan {
        FieldServicePlan::new(
            vec![
                Location::new(
                    "loc-0",
                    "loc-0",
                    "A".into(),
                    45_000_000,
                    9_000_000,
                    "x".into(),
                ),
                Location::new(
                    "loc-1",
                    "loc-1",
                    "B".into(),
                    45_001_000,
                    9_001_000,
                    "x".into(),
                ),
                Location::new(
                    "loc-2",
                    "loc-2",
                    "C".into(),
                    45_002_000,
                    9_002_000,
                    "x".into(),
                ),
            ],
            Vec::new(),
            travel_legs,
            Vec::new(),
        )
    }
}
