//! Turns a plan's coordinates into the prepared road data used for scoring.
//!
//! This is the only place the app talks to the road network while solving.
//! It loads (or downloads once and caches) the OpenStreetMap roads around
//! every customer, depot, and charger, computes one travel-time and distance
//! matrix over all of them, and attaches the frozen result to the plan.

use std::sync::Arc;

use solverforge::cvrp::{DenseMatrix, ProblemData};
use solverforge_maps::{
    BoundingBox, Coord, NetworkConfig, RoadNetwork, RoutingError, SnapAccess, UNREACHABLE,
};

use super::routing_network::{CustomerTerms, NetworkHandle, RoutingNetwork};
use super::vehicle::VehicleRouting;
use super::{CoordValue, Plan};

/// Distinct depot locations in first-seen order, and each vehicle's index
/// into that list. Vans sharing a yard share one matrix node.
pub fn depot_sites(plan: &Plan) -> (Vec<(CoordValue, CoordValue)>, Vec<usize>) {
    let mut sites: Vec<(CoordValue, CoordValue)> = Vec::new();
    let mut vehicle_site = Vec::with_capacity(plan.vehicles.len());
    for vehicle in &plan.vehicles {
        let key = (vehicle.home_lat, vehicle.home_lng);
        let site = sites.iter().position(|known| *known == key).unwrap_or_else(|| {
            sites.push(key);
            sites.len() - 1
        });
        vehicle_site.push(site);
    }
    (sites, vehicle_site)
}

/// Every location in matrix node order: customers, depot sites, chargers.
pub fn plan_coords(plan: &Plan) -> Result<Vec<Coord>, RoutingError> {
    let (sites, _) = depot_sites(plan);
    let mut coords = Vec::with_capacity(plan.customers.len() + sites.len() + plan.chargers.len());
    for customer in &plan.customers {
        coords.push(customer.coord()?);
    }
    for (lat, lng) in sites {
        coords.push(Coord::try_new(lat.get(), lng.get())?);
    }
    for charger in &plan.chargers {
        coords.push(charger.coord()?);
    }
    Ok(coords)
}

/// Loads the road network around a plan and attaches its prepared matrices.
pub async fn prepare_plan(plan: &mut Plan) -> Result<(), RoutingError> {
    let coords = plan_coords(plan)?;
    if coords.is_empty() {
        attach_network(plan, Vec::new(), Vec::new());
        return Ok(());
    }
    let bbox = BoundingBox::from_coords(&coords).expand_for_routing(&coords);
    let network = RoadNetwork::load_or_fetch(&bbox, &NetworkConfig::default(), None).await?;
    let access = vec![SnapAccess::Both; coords.len()];
    let matrix = network.compute_matrix_with_access(&coords, &access, None).await;

    let n = coords.len();
    let mut seconds = vec![vec![0_i64; n]; n];
    let mut meters = vec![vec![0_i64; n]; n];
    for from in 0..n {
        for to in 0..n {
            seconds[from][to] = matrix.get(from, to).unwrap_or(UNREACHABLE);
            meters[from][to] = matrix.distance_meters(from, to).unwrap_or(UNREACHABLE);
        }
    }
    attach_network(plan, seconds, meters);
    Ok(())
}

/// Attaches already-computed matrices (node order as in [`plan_coords`]) and
/// refreshes every route shadow. Tests call this with synthetic matrices.
pub fn attach_network(plan: &mut Plan, seconds: Vec<Vec<i64>>, meters: Vec<Vec<i64>>) {
    let customers: Vec<CustomerTerms> = plan
        .customers
        .iter()
        .map(|customer| CustomerTerms {
            demand: customer.demand,
            min_start_time: customer.min_start_time,
            max_end_time: customer.max_end_time,
            service_duration: customer.service_duration,
        })
        .collect();
    let (sites, vehicle_site) = depot_sites(plan);
    let network = NetworkHandle(Arc::new(RoutingNetwork::new(
        customers,
        sites.len(),
        plan.chargers.len(),
        seconds,
        meters,
    )));

    plan.problem_data = plan
        .vehicles
        .iter()
        .zip(&vehicle_site)
        .map(|(vehicle, &site)| {
            Arc::new(problem_data(
                &network,
                network.depot_node(site),
                i64::from(vehicle.capacity),
                vehicle.departure_time,
            ))
        })
        .collect();
    for (vehicle, &site) in plan.vehicles.iter_mut().zip(&vehicle_site) {
        vehicle.routing = Some(VehicleRouting {
            network: network.clone(),
            depot: network.depot_node(site),
        });
        vehicle.refresh_route_shadows();
    }
}

/// Stock CVRP view of one vehicle, used by Clarke-Wright savings and the
/// nearby distance meters. Its "distance" is road seconds, so construction
/// merges routes by travel time, the same quantity the soft score minimises.
fn problem_data(
    network: &RoutingNetwork,
    depot: usize,
    capacity: i64,
    departure_time: i64,
) -> ProblemData {
    let nodes = network.seconds.len();
    let mut demands = vec![0_i32; nodes];
    let mut time_windows = vec![(0_i64, i64::MAX); nodes];
    let mut service_durations = vec![0_i64; nodes];
    for (idx, customer) in network.customers.iter().enumerate() {
        demands[idx] = customer.demand;
        time_windows[idx] = (customer.min_start_time, customer.max_end_time);
        service_durations[idx] = customer.service_duration;
    }
    ProblemData {
        capacity,
        depot,
        demands,
        metric: DenseMatrix::new(network.seconds.clone(), network.seconds.clone()),
        time_windows,
        service_durations,
        vehicle_departure_time: departure_time,
    }
}
