use solverforge_maps::UNREACHABLE;

use crate::domain::Plan;

use super::helpers::{
    fallback_delivery_to_delivery, fallback_delivery_to_vehicle, fallback_vehicle_to_delivery,
    normalized_travel_time,
};
use super::types::DeliveryRoutingSolution;

/// Depot token used by Clarke-Wright before it commits visits to vehicle routes.
pub fn delivery_clarke_wright_depot<S: DeliveryRoutingSolution>(solution: &S) -> usize {
    virtual_depot_value(solution.delivery_plan(), 0)
}

/// Per-vehicle depot token used by k-opt route reconnection.
pub fn delivery_k_opt_depot<S: DeliveryRoutingSolution>(solution: &S, entity_idx: usize) -> usize {
    virtual_depot_value(solution.delivery_plan(), entity_idx)
}

/// Reads travel time between delivery ids and virtual depot ids.
///
/// Virtual depot ids are `deliveries.len() + vehicle_idx`. Prepared route
/// matrices are preferred; straight-line fallback keeps previews and tests
/// usable before full map-backed preparation has run.
pub fn delivery_route_distance<S: DeliveryRoutingSolution>(
    solution: &S,
    from: usize,
    to: usize,
) -> i64 {
    if from == to {
        return 0;
    }

    let plan = solution.delivery_plan();
    match (
        virtual_depot_entity(plan, from),
        virtual_depot_entity(plan, to),
        plan.deliveries.get(from),
        plan.deliveries.get(to),
    ) {
        (Some(_), Some(_), _, _) => 0,
        (Some(vehicle_idx), None, _, Some(_)) => depot_to_delivery_seconds(plan, vehicle_idx, to),
        (None, Some(vehicle_idx), Some(_), _) => delivery_to_depot_seconds(plan, vehicle_idx, from),
        (None, None, Some(_), Some(_)) => delivery_to_delivery_seconds(plan, from, to),
        _ => 0,
    }
}

pub fn delivery_element_load<S: DeliveryRoutingSolution>(solution: &S, delivery_id: usize) -> i64 {
    solution
        .delivery_plan()
        .deliveries
        .get(delivery_id)
        .map_or(0, |delivery| i64::from(delivery.demand))
}

/// Conservative route capacity used by list construction when assigning visits.
pub fn delivery_route_capacity<S: DeliveryRoutingSolution>(solution: &S) -> i64 {
    let plan = solution.delivery_plan();
    plan.deliveries
        .iter()
        .map(|delivery| i64::from(delivery.demand))
        .sum::<i64>()
        .max(1)
}

/// Gives list-aware move selectors an owned copy of one vehicle route.
pub fn get_delivery_route<S: DeliveryRoutingSolution>(solution: &S, entity_idx: usize) -> Vec<usize> {
    solution.vehicle_visits(entity_idx).to_vec()
}

/// Replaces one route after construction or a k-opt/list move accepts it.
pub fn replace_delivery_route<S: DeliveryRoutingSolution>(
    solution: &mut S,
    entity_idx: usize,
    route: Vec<usize>,
) {
    *solution.vehicle_visits_mut(entity_idx) = route;
}

/// Guards k-opt proposals with the same route feasibility signals the app scores.
///
/// The hook rejects unknown delivery ids, unreachable legs, and routes that
/// would miss a delivery time window after travel, waiting, and service time.
pub fn delivery_k_opt_feasible<S: DeliveryRoutingSolution>(
    solution: &S,
    entity_idx: usize,
    route: &[usize],
) -> bool {
    let plan = solution.delivery_plan();
    let Some(prepared) = plan
        .vehicles
        .get(entity_idx)
        .and_then(|vehicle| vehicle.prepared_routing.as_ref())
    else {
        return true;
    };

    let mut current = prepared.vehicle_departure_time;
    let mut previous: Option<usize> = None;

    for &delivery_id in route {
        if delivery_id >= prepared.demands.len() {
            return false;
        }
        let travel = match previous {
            Some(previous_id) => prepared.travel_times[previous_id][delivery_id],
            None => prepared.depot_to_delivery_seconds[delivery_id],
        };
        if travel == UNREACHABLE {
            return false;
        }

        current += normalized_travel_time(travel);
        let (min_start, max_end) = prepared.time_windows[delivery_id];
        if current < min_start {
            current = min_start;
        }

        current += prepared.service_durations[delivery_id];
        if current > max_end {
            return false;
        }
        previous = Some(delivery_id);
    }

    true
}

fn virtual_depot_value(plan: &Plan, entity_idx: usize) -> usize {
    plan.deliveries.len().saturating_add(entity_idx)
}

fn virtual_depot_entity(plan: &Plan, value: usize) -> Option<usize> {
    value
        .checked_sub(plan.deliveries.len())
        .filter(|&entity_idx| entity_idx < plan.vehicles.len())
}

fn depot_to_delivery_seconds(plan: &Plan, vehicle_idx: usize, delivery_id: usize) -> i64 {
    plan.vehicles
        .get(vehicle_idx)
        .and_then(|vehicle| vehicle.prepared_routing.as_ref())
        .and_then(|prepared| {
            prepared
                .depot_to_delivery_seconds
                .get(delivery_id)
                .copied()
        })
        .map(normalized_travel_time)
        .or_else(|| fallback_vehicle_to_delivery(plan, vehicle_idx, delivery_id))
        .unwrap_or(0)
}

fn delivery_to_depot_seconds(plan: &Plan, vehicle_idx: usize, delivery_id: usize) -> i64 {
    plan.vehicles
        .get(vehicle_idx)
        .and_then(|vehicle| vehicle.prepared_routing.as_ref())
        .and_then(|prepared| {
            prepared
                .delivery_to_depot_seconds
                .get(delivery_id)
                .copied()
        })
        .map(normalized_travel_time)
        .or_else(|| fallback_delivery_to_vehicle(plan, vehicle_idx, delivery_id))
        .unwrap_or(0)
}

fn delivery_to_delivery_seconds(plan: &Plan, from: usize, to: usize) -> i64 {
    plan.vehicles
        .iter()
        .find_map(|vehicle| vehicle.prepared_routing.as_ref())
        .and_then(|prepared| {
            prepared
                .travel_times
                .get(from)
                .and_then(|row| row.get(to))
                .copied()
        })
        .map(normalized_travel_time)
        .or_else(|| fallback_delivery_to_delivery(plan, from, to))
        .unwrap_or(0)
}
