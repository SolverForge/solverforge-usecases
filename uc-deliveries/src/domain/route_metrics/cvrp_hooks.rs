use solverforge_maps::UNREACHABLE;

use super::helpers::normalized_travel_time;
use super::types::DeliveryRoutingSolution;

/// Guards route proposals with the same feasibility signals the app scores.
///
/// Empty owner routes are Clarke-Wright construction candidates, so they keep
/// the old construction contract and only reject unknown delivery ids. Once a
/// route exists, the same hook keeps the old k-opt pruning behavior and rejects
/// unreachable legs or missed delivery time windows.
pub fn delivery_route_feasible<S: DeliveryRoutingSolution>(
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

    if solution.vehicle_visits(entity_idx).is_empty() {
        return route
            .iter()
            .all(|&delivery_id| delivery_id < prepared.demands.len());
    }

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
