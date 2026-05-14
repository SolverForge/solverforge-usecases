use std::sync::Arc;

use solverforge::cvrp::ProblemData;
use solverforge_maps::{
    haversine_distance, BoundingBox, Coord, NetworkConfig, RoadNetwork, RoutingError, UNREACHABLE,
};

use crate::domain::{Plan, RoutingMode};

use super::helpers::{
    build_delivery_distance_matrix, build_travel_time_matrix, delivery_coords, meters_to_seconds,
};
use super::types::PreparedVehicleRouting;

/// Builds the routing data SolverForge should read during a solve.
///
/// This is the boundary between transport/domain data and the hot scoring path:
/// route ids are normalized, delivery matrices are computed once, per-vehicle
/// depot legs are attached, and the list-variable hooks can then score from
/// cached `ProblemData` instead of resolving maps during every move.
pub async fn prepare_plan(plan: &mut Plan) -> Result<(), RoutingError> {
    plan.normalize();
    let delivery_coords = delivery_coords(plan)?;
    let delivery_distance_matrix = build_delivery_distance_matrix(&delivery_coords);
    let delivery_demands: Vec<i32> = plan
        .deliveries
        .iter()
        .map(|delivery| delivery.demand)
        .collect();
    let delivery_time_windows: Vec<(i64, i64)> = plan
        .deliveries
        .iter()
        .map(|delivery| (delivery.min_start_time, delivery.max_end_time))
        .collect();
    let delivery_service_durations: Vec<i64> = plan
        .deliveries
        .iter()
        .map(|delivery| delivery.service_duration)
        .collect();

    let depot_routing = match plan.routing_mode {
        RoutingMode::StraightLine => DepotRoutingData::straight_line(plan, &delivery_coords)?,
        RoutingMode::RoadNetwork => DepotRoutingData::road_network(plan, &delivery_coords).await?,
    };

    plan.prepared_problem_data.clear();
    for (vehicle_idx, vehicle) in plan.vehicles.iter_mut().enumerate() {
        plan.prepared_problem_data.push(Arc::new(ProblemData {
            capacity: vehicle.capacity as i64,
            depot: 0,
            demands: delivery_demands.clone(),
            distance_matrix: delivery_distance_matrix.clone(),
            time_windows: delivery_time_windows.clone(),
            service_durations: delivery_service_durations.clone(),
            travel_times: depot_routing.delivery_travel_times.clone(),
            vehicle_departure_time: vehicle.departure_time,
        }));
        vehicle.prepared_routing = Some(PreparedVehicleRouting {
            problem_data_index: vehicle_idx,
            capacity: vehicle.capacity as i64,
            demands: delivery_demands.clone(),
            distance_matrix: delivery_distance_matrix.clone(),
            time_windows: delivery_time_windows.clone(),
            service_durations: delivery_service_durations.clone(),
            travel_times: depot_routing.delivery_travel_times.clone(),
            vehicle_departure_time: vehicle.departure_time,
            depot_to_delivery_seconds: depot_routing.depot_to_delivery_seconds[vehicle_idx].clone(),
            delivery_to_depot_seconds: depot_routing.delivery_to_depot_seconds[vehicle_idx].clone(),
            depot_to_delivery_meters: depot_routing.depot_to_delivery_meters[vehicle_idx].clone(),
            delivery_to_depot_meters: depot_routing.delivery_to_depot_meters[vehicle_idx].clone(),
        });
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct DepotRoutingData {
    delivery_travel_times: Vec<Vec<i64>>,
    depot_to_delivery_seconds: Vec<Vec<i64>>,
    delivery_to_depot_seconds: Vec<Vec<i64>>,
    depot_to_delivery_meters: Vec<Vec<i64>>,
    delivery_to_depot_meters: Vec<Vec<i64>>,
}

impl DepotRoutingData {
    fn straight_line(plan: &Plan, delivery_coords: &[Coord]) -> Result<Self, RoutingError> {
        let delivery_travel_times = build_travel_time_matrix(delivery_coords);
        let mut depot_to_delivery_seconds = Vec::with_capacity(plan.vehicles.len());
        let mut delivery_to_depot_seconds = Vec::with_capacity(plan.vehicles.len());
        let mut depot_to_delivery_meters = Vec::with_capacity(plan.vehicles.len());
        let mut delivery_to_depot_meters = Vec::with_capacity(plan.vehicles.len());

        for vehicle in &plan.vehicles {
            let depot = vehicle.depot_coord()?;
            let mut outbound_seconds = Vec::with_capacity(delivery_coords.len());
            let mut inbound_seconds = Vec::with_capacity(delivery_coords.len());
            let mut outbound_meters = Vec::with_capacity(delivery_coords.len());
            let mut inbound_meters = Vec::with_capacity(delivery_coords.len());
            for coord in delivery_coords {
                let meters = haversine_distance(depot, *coord).round() as i64;
                let seconds = meters_to_seconds(meters);
                outbound_seconds.push(seconds);
                inbound_seconds.push(seconds);
                outbound_meters.push(meters);
                inbound_meters.push(meters);
            }
            depot_to_delivery_seconds.push(outbound_seconds);
            delivery_to_depot_seconds.push(inbound_seconds);
            depot_to_delivery_meters.push(outbound_meters);
            delivery_to_depot_meters.push(inbound_meters);
        }

        Ok(Self {
            delivery_travel_times,
            depot_to_delivery_seconds,
            delivery_to_depot_seconds,
            depot_to_delivery_meters,
            delivery_to_depot_meters,
        })
    }

    async fn road_network(plan: &Plan, delivery_coords: &[Coord]) -> Result<Self, RoutingError> {
        let mut all_coords = delivery_coords.to_vec();
        for vehicle in &plan.vehicles {
            all_coords.push(vehicle.depot_coord()?);
        }

        if all_coords.is_empty() {
            return Ok(Self {
                delivery_travel_times: Vec::new(),
                depot_to_delivery_seconds: Vec::new(),
                delivery_to_depot_seconds: Vec::new(),
                depot_to_delivery_meters: Vec::new(),
                delivery_to_depot_meters: Vec::new(),
            });
        }

        let bbox = BoundingBox::from_coords(&all_coords).expand_for_routing(&all_coords);
        let network = RoadNetwork::load_or_fetch(&bbox, &NetworkConfig::default(), None).await?;
        let matrix = network.compute_matrix(&all_coords, None).await;

        let delivery_count = delivery_coords.len();
        let mut delivery_travel_times = vec![vec![0_i64; delivery_count]; delivery_count];
        for (i, row) in delivery_travel_times.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = matrix.get(i, j).unwrap_or(UNREACHABLE);
            }
        }

        let mut depot_to_delivery_seconds = Vec::with_capacity(plan.vehicles.len());
        let mut delivery_to_depot_seconds = Vec::with_capacity(plan.vehicles.len());
        let mut depot_to_delivery_meters = Vec::with_capacity(plan.vehicles.len());
        let mut delivery_to_depot_meters = Vec::with_capacity(plan.vehicles.len());

        for vehicle_idx in 0..plan.vehicles.len() {
            let depot_idx = delivery_count + vehicle_idx;
            let depot = plan.vehicles[vehicle_idx].depot_coord()?;
            let mut outbound_seconds = Vec::with_capacity(delivery_count);
            let mut inbound_seconds = Vec::with_capacity(delivery_count);
            let mut outbound_meters = Vec::with_capacity(delivery_count);
            let mut inbound_meters = Vec::with_capacity(delivery_count);
            for (delivery_idx, delivery_coord) in delivery_coords.iter().enumerate() {
                outbound_seconds.push(matrix.get(depot_idx, delivery_idx).unwrap_or(UNREACHABLE));
                inbound_seconds.push(matrix.get(delivery_idx, depot_idx).unwrap_or(UNREACHABLE));
                let meters = haversine_distance(depot, *delivery_coord).round() as i64;
                outbound_meters.push(meters);
                inbound_meters.push(meters);
            }
            depot_to_delivery_seconds.push(outbound_seconds);
            delivery_to_depot_seconds.push(inbound_seconds);
            depot_to_delivery_meters.push(outbound_meters);
            delivery_to_depot_meters.push(inbound_meters);
        }

        Ok(Self {
            delivery_travel_times,
            depot_to_delivery_seconds,
            delivery_to_depot_seconds,
            depot_to_delivery_meters,
            delivery_to_depot_meters,
        })
    }
}
