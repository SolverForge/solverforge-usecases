//! Exact score tests for every constraint, plus an incremental-versus-full
//! walk that pins the path the solver actually uses.
//!
//! The fixture puts every place on one straight road (positions in km) so each
//! expected penalty can be worked out by hand:
//! hub 0, customer A 40, customer B 80, customer C 20, charger 45; 10 m/s.

use solverforge::prelude::*;
use solverforge::ScoreDirector;
use solverforge_maps::UNREACHABLE;

use super::create_constraints;
use crate::domain::{attach_network, Charger, Customer, Plan, Vehicle, VehicleInit};

const KM: i64 = 1_000;
const DAY: (i64, i64) = (0, 86_400);

fn van(id: &str, capacity: i32, initial_km: i64, range_km: i64) -> Vehicle {
    Vehicle::new(VehicleInit {
        id: id.into(),
        name: id.into(),
        capacity,
        home: (45.0, 9.0),
        departure_time: 8 * 3_600,
        battery_range_m: range_km * KM,
        initial_charge_m: initial_km * KM,
        recharge_seconds: 1_800,
        reserve_charge_m: 0,
    })
}

/// Builds a plan whose routes are `routes[v]`, then attaches the line network.
fn plan(vans: Vec<Vehicle>, routes: &[&[usize]], b_closes: i64, cut_a_c: bool) -> Plan {
    let customers = vec![
        Customer::new("a", "A", (45.1, 9.0), 3, DAY, 0),
        Customer::new("b", "B", (45.2, 9.0), 3, (0, b_closes), 0),
        Customer::new("c", "C", (45.3, 9.0), 3, DAY, 0),
    ];
    let chargers = vec![Charger::new("k", "K", 45.4, 9.0)];
    let mut plan = Plan::new(customers, chargers, vans);
    for (vehicle, route) in plan.vehicles.iter_mut().zip(routes) {
        vehicle.visit_order = route.to_vec();
    }
    // Node order: A, B, C, hub, charger.
    let positions = [40 * KM, 80 * KM, 20 * KM, 0, 45 * KM];
    let mut meters: Vec<Vec<i64>> = positions
        .iter()
        .map(|a| positions.iter().map(|b| (a - b).abs()).collect())
        .collect();
    if cut_a_c {
        meters[0][2] = UNREACHABLE;
        meters[2][0] = UNREACHABLE;
    }
    let seconds = meters
        .iter()
        .map(|row| {
            row.iter()
                .map(|&m| if m == UNREACHABLE { m } else { m / 10 })
                .collect()
        })
        .collect();
    attach_network(&mut plan, seconds, meters);
    plan
}

fn full_battery(capacity: i32) -> Vehicle {
    van("v0", capacity, 500, 500)
}

#[test]
fn vehicle_capacity_penalises_each_crate_over() {
    let plan = plan(vec![full_battery(5)], &[&[2, 0, 1]], 86_400, false);
    let score = (super::vehicle_capacity::constraint(),).evaluate_all(&plan);
    assert_eq!(score, HardSoftScore::of(-4, 0));
}

#[test]
fn customer_time_windows_penalise_late_seconds() {
    // Hub -> A (4 000 s) -> B (8 000 s after departure); B closes 100 s earlier.
    let closes = 8 * 3_600 + 7_900;
    let plan = plan(vec![full_battery(50)], &[&[0, 1]], closes, false);
    let score = (super::customer_time_windows::constraint(),).evaluate_all(&plan);
    assert_eq!(score, HardSoftScore::of(-100, 0));
}

#[test]
fn battery_shortfall_penalises_missing_metres_after_best_charging() {
    // Hub -> C -> hub is 40 km on 20 km of charge with a 30 km range. The
    // charger at 45 km is a 25 km detour from C, so no charging plan helps:
    // the best plan drives without stopping and misses 20 km.
    let plan = plan(vec![van("v0", 50, 20, 30)], &[&[2]], 86_400, false);
    let score = (super::battery_shortfall::constraint(),).evaluate_all(&plan);
    assert_eq!(score, HardSoftScore::of(-20 * KM, 0));
}

#[test]
fn battery_shortfall_is_zero_once_a_charge_covers_the_route() {
    // Hub -> A -> B -> hub (160 km) on 50 km with a 100 km range: two stops on
    // the roadside charger make it feasible.
    let plan = plan(vec![van("v0", 50, 50, 100)], &[&[0, 1]], 86_400, false);
    let score = (super::battery_shortfall::constraint(),).evaluate_all(&plan);
    assert_eq!(score, HardSoftScore::of(0, 0));
    assert_eq!(plan.vehicles[0].route_charging_seconds, 3_600);
}

#[test]
fn unreachable_legs_count_each_missing_road() {
    let plan = plan(vec![full_battery(50)], &[&[0, 2]], 86_400, true);
    let score = (super::unreachable_legs::constraint(),).evaluate_all(&plan);
    assert_eq!(score, HardSoftScore::of(-1, 0));
}

#[test]
fn total_travel_time_counts_driving_and_charging() {
    let full = plan(vec![full_battery(50)], &[&[0]], 86_400, false);
    let score = (super::total_travel_time::constraint(),).evaluate_all(&full);
    assert_eq!(score, HardSoftScore::of(0, -8_000));

    let charging = plan(vec![van("v0", 50, 50, 100)], &[&[0, 1]], 86_400, false);
    let score = (super::total_travel_time::constraint(),).evaluate_all(&charging);
    assert_eq!(score, HardSoftScore::of(0, -(16_000 + 3_600)));
}

#[test]
fn incremental_scoring_matches_full_evaluation_on_every_move() {
    let vans = vec![van("v0", 6, 50, 100), van("v1", 4, 30, 60)];
    let start = plan(vans, &[&[0, 1], &[2]], 8 * 3_600 + 9_000, false);
    let mut director = ScoreDirector::new(start, create_constraints());
    director.calculate_score();

    let mut seed = 0x9e37_79b9_u64;
    let mut next = |bound: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize % bound.max(1)
    };
    for _ in 0..300 {
        let from = next(2);
        let to = next(2);
        let len = director.working_solution().vehicles[from].visit_order.len();
        if len == 0 {
            continue;
        }
        let take = next(len);
        let insert_bound = director.working_solution().vehicles[to].visit_order.len() + 1;
        let at = next(insert_bound);
        director.before_variable_changed(0, from);
        if to != from {
            director.before_variable_changed(0, to);
        }
        let solution = director.working_solution_mut();
        let customer = solution.vehicles[from].visit_order.remove(take);
        let at = at.min(solution.vehicles[to].visit_order.len());
        solution.vehicles[to].visit_order.insert(at, customer);
        director.after_variable_changed(0, from);
        if to != from {
            director.after_variable_changed(0, to);
        }
        let incremental = director.calculate_score();
        let full = create_constraints().evaluate_all(director.working_solution());
        assert_eq!(incremental, full, "after moving customer {customer}");
    }
}

#[test]
fn construction_feasibility_hook_requires_a_drivable_battery_plan() {
    use crate::domain::EvRouteSolution;

    let chargeable = plan(vec![van("v0", 50, 50, 100)], &[&[]], 86_400, false);
    assert!(chargeable.ev_route_feasible(0, &[0, 1]));
    let stranded = plan(vec![van("v0", 50, 20, 30)], &[&[]], 86_400, false);
    assert!(!stranded.ev_route_feasible(0, &[2]));
    assert!(
        !stranded.ev_route_feasible(0, &[7]),
        "unknown customer index"
    );
    assert_eq!(
        stranded.depot_class(0),
        3,
        "hub is node 3 after three customers"
    );
}
