//! Charging-stop placement for one fixed customer sequence.
//!
//! The solver decides which customers a van visits and in what order. Where the
//! van stops to charge is not a planning variable: for a given order the best
//! charging stops can be computed exactly, so they are derived here every time
//! a route changes instead of being searched for.
//!
//! Battery model:
//! - charge is measured in metres of driving range, and one metre driven uses
//!   one metre of charge;
//! - every charging stop refills the battery completely and takes the
//!   vehicle's fixed `recharge_seconds`;
//! - a charging stop sits on a leg between two consecutive route nodes, as a
//!   detour `from -> charger -> to`.
//!
//! The planner is a dynamic program over "the leg where the van last charged".
//! For every leg it considers a small set of candidate chargers (see
//! [`LegChargerOption`]). Among all ways of placing stops on those candidates
//! it returns the plan that first minimises the range shortfall (metres the
//! battery cannot cover, which the hard score penalises) and then minimises
//! the time the stops add to the route (detour driving plus charging).

use serde::{Deserialize, Serialize};

/// One way to charge while driving a single leg `from -> to`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LegChargerOption {
    /// Charger index into `Plan.chargers`.
    pub charger: usize,
    /// Road metres from the leg start to the charger.
    pub to_charger_m: i64,
    /// Road metres from the charger to the leg end.
    pub from_charger_m: i64,
    /// Road seconds from the leg start to the charger.
    pub to_charger_s: i64,
    /// Road seconds from the charger to the leg end.
    pub from_charger_s: i64,
}

/// The facts the planner needs about one leg of the route.
#[derive(Clone, Copy, Debug, Default)]
pub struct LegInput<'a> {
    /// Direct road metres of the leg; `0` for an unreachable leg, which the
    /// `unreachable_legs` rule scores instead.
    pub direct_m: i64,
    /// Direct road seconds of the leg.
    pub direct_s: i64,
    /// Candidate chargers for a detour on this leg.
    pub options: &'a [LegChargerOption],
}

/// One planned charging stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedCharge {
    /// Leg index: leg `k` runs from route node `k` to route node `k + 1`,
    /// where node `0` is the depot departure.
    pub leg: usize,
    /// Charger index into `Plan.chargers`.
    pub charger: usize,
}

/// The cheapest charging plan for one route.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChargingPlan {
    pub stops: Vec<PlannedCharge>,
    /// Metres of driving the battery cannot supply under this plan.
    pub shortfall_m: i64,
}

/// Lexicographic cost of a partial plan: shortfall first, added seconds next.
type Cost = (i64, i64);

/// A dynamic-programming state: the van has just charged at one candidate.
#[derive(Clone, Copy)]
struct State {
    leg: usize,
    option: usize,
    cost: Cost,
    /// Index of the previous state, or `None` when charged first time here.
    previous: Option<usize>,
}

/// Places charging stops on one route.
///
/// `initial_charge_m` is the charge at departure and `range_m` the charge after
/// any full recharge. Runs in `O(S^2)` for `S` candidate stops (legs times
/// options per leg), which stays small because each leg keeps at most three
/// candidates.
pub fn plan_charging(
    legs: &[LegInput<'_>],
    initial_charge_m: i64,
    range_m: i64,
    recharge_seconds: i64,
) -> ChargingPlan {
    let initial = initial_charge_m.clamp(0, range_m.max(0));
    let range = range_m.max(0);

    // prefix[k] = metres driven on legs 0..k with no detours.
    let mut prefix = Vec::with_capacity(legs.len() + 1);
    prefix.push(0_i64);
    for leg in legs {
        prefix.push(prefix.last().copied().unwrap_or(0) + leg.direct_m.max(0));
    }
    let total = prefix[legs.len()];

    let mut states: Vec<State> = Vec::new();
    for (leg_idx, leg) in legs.iter().enumerate() {
        for (option_idx, option) in leg.options.iter().enumerate() {
            let added_s = stop_seconds(leg, option, recharge_seconds);

            // Charge here first: drive from the depot on the initial charge.
            let drain = prefix[leg_idx] + option.to_charger_m;
            let mut best = State {
                leg: leg_idx,
                option: option_idx,
                cost: ((drain - initial).max(0), added_s),
                previous: None,
            };

            // Or charge here after an earlier stop on an earlier leg.
            for (state_idx, earlier) in states.iter().enumerate() {
                if earlier.leg >= leg_idx {
                    continue;
                }
                let from_earlier = legs[earlier.leg].options[earlier.option].from_charger_m;
                let drain =
                    from_earlier + (prefix[leg_idx] - prefix[earlier.leg + 1]) + option.to_charger_m;
                let cost = (
                    earlier.cost.0 + (drain - range).max(0),
                    earlier.cost.1 + added_s,
                );
                if cost < best.cost {
                    best.cost = cost;
                    best.previous = Some(state_idx);
                }
            }
            states.push(best);
        }
    }

    // No stop at all: the whole route runs on the initial charge.
    let mut best_cost: Cost = ((total - initial).max(0), 0);
    let mut best_last: Option<usize> = None;
    for (state_idx, state) in states.iter().enumerate() {
        let from_charger = legs[state.leg].options[state.option].from_charger_m;
        let drain = from_charger + (total - prefix[state.leg + 1]);
        let cost = (state.cost.0 + (drain - range).max(0), state.cost.1);
        if cost < best_cost {
            best_cost = cost;
            best_last = Some(state_idx);
        }
    }

    let mut stops = Vec::new();
    let mut cursor = best_last;
    while let Some(state_idx) = cursor {
        let state = states[state_idx];
        stops.push(PlannedCharge {
            leg: state.leg,
            charger: legs[state.leg].options[state.option].charger,
        });
        cursor = state.previous;
    }
    stops.reverse();

    ChargingPlan {
        stops,
        shortfall_m: best_cost.0,
    }
}

/// Seconds a stop adds compared with driving the leg directly.
fn stop_seconds(leg: &LegInput<'_>, option: &LegChargerOption, recharge_seconds: i64) -> i64 {
    option.to_charger_s + option.from_charger_s - leg.direct_s + recharge_seconds.max(0)
}

#[cfg(test)]
mod tests;
