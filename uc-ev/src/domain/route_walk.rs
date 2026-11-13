//! One walk over a vehicle route: load, time, battery, and charging stops.
//!
//! Every score-relevant route fact is produced here, once, so the shadow
//! refresh, the construction feasibility hook, and the browser schedule all
//! read the same numbers.

use serde::{Deserialize, Serialize};

use super::charging::{plan_charging, LegInput, PlannedCharge};
use super::routing_network::RoutingNetwork;

/// Vehicle parameters a route walk needs.
#[derive(Clone, Copy, Debug, Default)]
pub struct VehicleTerms {
    pub capacity: i64,
    pub departure_time: i64,
    pub battery_range_m: i64,
    pub initial_charge_m: i64,
    pub recharge_seconds: i64,
    /// Range the battery must keep at all times; only charge above it is usable.
    pub reserve_charge_m: i64,
}

/// Totals of one route, read by constraints through vehicle shadows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RouteSummary {
    pub demand: i64,
    pub distance_m: i64,
    pub driving_s: i64,
    pub charging_s: i64,
    pub late_s: i64,
    pub unreachable_legs: i64,
    /// Metres the route dips below the reserve, after optimal charging.
    pub shortfall_m: i64,
    pub charges: Vec<PlannedCharge>,
    pub return_time: i64,
}

/// What happens at one point of the route.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteEventKind {
    #[default]
    Depart,
    Charge,
    Visit,
    Return,
}

/// A timed route event for the browser schedule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteEvent {
    pub kind: RouteEventKind,
    /// Customer index for `visit`, charger index for `charge`, else `None`.
    pub index: Option<usize>,
    /// Seconds after midnight the vehicle arrives.
    pub arrival: i64,
    /// Seconds after midnight work (service or charging) starts.
    pub start: i64,
    /// Seconds after midnight the vehicle leaves.
    pub end: i64,
    /// Battery metres left on arrival; negative means the range ran out.
    pub charge_on_arrival_m: i64,
    /// Seconds the service ended after the customer's window closed.
    pub late_s: i64,
}

/// Walks `depot -> visits... -> depot` and, when `schedule` is given, records
/// every timed event.
pub fn walk_route(
    network: &RoutingNetwork,
    depot: usize,
    terms: VehicleTerms,
    visits: &[usize],
    mut schedule: Option<&mut Vec<RouteEvent>>,
) -> RouteSummary {
    let mut summary = RouteSummary {
        return_time: terms.departure_time,
        ..RouteSummary::default()
    };
    if visits.is_empty() {
        return summary;
    }

    let nodes: Vec<usize> = std::iter::once(depot)
        .chain(visits.iter().copied())
        .chain(std::iter::once(depot))
        .collect();
    let legs: Vec<LegInput<'_>> = nodes
        .windows(2)
        .map(|pair| network.leg_input(pair[0], pair[1]))
        .collect();
    // Plan on usable charge only: everything above the reserve.
    let range = terms.battery_range_m.max(0);
    let reserve = terms.reserve_charge_m.clamp(0, range);
    let plan = plan_charging(
        &legs,
        terms.initial_charge_m.min(range) - reserve,
        range - reserve,
        terms.recharge_seconds,
    );

    let mut charge = terms.initial_charge_m.clamp(0, range);
    let mut time = terms.departure_time;
    record(&mut schedule, RouteEvent::at(RouteEventKind::Depart, None, time, charge));

    let mut next_stop = plan.stops.iter().peekable();
    for (leg_idx, pair) in nodes.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        if !network.reachable(from, to) {
            summary.unreachable_legs += 1;
        }

        if let Some(stop) = next_stop.next_if(|stop| stop.leg == leg_idx) {
            let option = legs[leg_idx]
                .options
                .iter()
                .find(|option| option.charger == stop.charger)
                .copied()
                .unwrap_or_default();
            time += option.to_charger_s;
            charge -= option.to_charger_m;
            record(
                &mut schedule,
                RouteEvent {
                    end: time + terms.recharge_seconds,
                    ..RouteEvent::at(RouteEventKind::Charge, Some(stop.charger), time, charge)
                },
            );
            time += terms.recharge_seconds + option.from_charger_s;
            charge = range - option.from_charger_m;
            summary.driving_s += option.to_charger_s + option.from_charger_s;
            summary.charging_s += terms.recharge_seconds;
            summary.distance_m += option.to_charger_m + option.from_charger_m;
        } else {
            time += legs[leg_idx].direct_s;
            charge -= legs[leg_idx].direct_m;
            summary.driving_s += legs[leg_idx].direct_s;
            summary.distance_m += legs[leg_idx].direct_m;
        }

        if to == depot {
            record(&mut schedule, RouteEvent::at(RouteEventKind::Return, None, time, charge));
            continue;
        }
        let customer = network.customers[to];
        summary.demand += i64::from(customer.demand);
        let arrival = time;
        let start = arrival.max(customer.min_start_time);
        let end = start + customer.service_duration.max(0);
        let late = (end - customer.max_end_time).max(0);
        summary.late_s += late;
        record(
            &mut schedule,
            RouteEvent {
                kind: RouteEventKind::Visit,
                index: Some(to),
                arrival,
                start,
                end,
                charge_on_arrival_m: charge,
                late_s: late,
            },
        );
        time = end;
    }

    summary.return_time = time;
    summary.shortfall_m = plan.shortfall_m;
    summary.charges = plan.stops;
    summary
}

impl RouteEvent {
    /// An event that starts and ends on arrival (depart, return, or the
    /// arrival half of a charge before its end is set).
    fn at(kind: RouteEventKind, index: Option<usize>, time: i64, charge: i64) -> Self {
        Self {
            kind,
            index,
            arrival: time,
            start: time,
            end: time,
            charge_on_arrival_m: charge,
            late_s: 0,
        }
    }
}

fn record(schedule: &mut Option<&mut Vec<RouteEvent>>, event: RouteEvent) {
    if let Some(events) = schedule.as_deref_mut() {
        events.push(event);
    }
}

#[cfg(test)]
mod tests;
