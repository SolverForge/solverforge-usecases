use solverforge_maps::{Coord, RoadNetwork};

use super::{route_geometry_with_endpoints, route_geometry_with_road_endpoints};

#[test]
fn road_route_geometry_adds_edge_snapped_visual_endpoints() {
    let network = RoadNetwork::from_test_data(
        &[(0.0, 0.0), (0.0, 0.01), (0.01, 0.01)],
        &[
            (0, 1, 60.0, 1_000.0),
            (1, 2, 60.0, 1_000.0),
            (1, 0, 60.0, 1_000.0),
            (2, 1, 60.0, 1_000.0),
        ],
    );
    let from = Coord::new(0.001, 0.004);
    let to = Coord::new(0.006, 0.011);

    let geometry = route_geometry_with_road_endpoints(
        &network,
        from,
        to,
        &[
            Coord::new(0.0, 0.0),
            Coord::new(0.0, 0.01),
            Coord::new(0.01, 0.01),
        ],
    )
    .expect("edge snaps should exist");

    assert_eq!(geometry.first().copied(), Some(from));
    assert_eq!(geometry.get(1).copied(), Some(Coord::new(0.0, 0.004)));
    assert_eq!(
        geometry.get(geometry.len() - 2).copied(),
        Some(Coord::new(0.006, 0.01))
    );
    assert_eq!(geometry.last().copied(), Some(to));
}

#[test]
fn road_route_geometry_is_stitched_to_exact_domain_endpoints() {
    let from = Coord::new(39.0, -75.0);
    let snapped_start = Coord::new(39.0002, -75.0002);
    let snapped_end = Coord::new(39.0018, -75.0018);
    let to = Coord::new(39.002, -75.002);

    let geometry =
        route_geometry_with_endpoints(from, None, to, None, &[snapped_start, snapped_end]);

    assert_eq!(geometry.first().copied(), Some(from));
    assert_eq!(geometry.last().copied(), Some(to));
    assert_eq!(geometry.len(), 4);
}

#[test]
fn road_route_geometry_avoids_duplicate_endpoint_points() {
    let from = Coord::new(39.0, -75.0);
    let mid = Coord::new(39.001, -75.001);
    let to = Coord::new(39.002, -75.002);

    let geometry = route_geometry_with_endpoints(from, None, to, None, &[from, mid, to]);

    assert_eq!(geometry, vec![from, mid, to]);
}
