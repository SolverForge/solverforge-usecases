//! Route walks on a hand-built network where every place sits on one straight
//! road, so each expected number can be checked with a ruler.
//!
//! Positions in kilometres: depot 0, customer A 40, customer B 80, charger 45.
//! Driving is 10 m/s.

use std::sync::Arc;

use super::*;
use crate::domain::{CustomerTerms, NetworkHandle};

const KM: i64 = 1_000;

fn network() -> NetworkHandle {
    network_with_b_closing(86_400)
}

fn network_with_b_closing(b_max_end: i64) -> NetworkHandle {
    // Node order: customer A, customer B, depot, charger.
    let positions = [40 * KM, 80 * KM, 0, 45 * KM];
    let meters: Vec<Vec<i64>> = positions
        .iter()
        .map(|a| positions.iter().map(|b| (a - b).abs()).collect())
        .collect();
    let seconds = meters
        .iter()
        .map(|row| row.iter().map(|m| m / 10).collect())
        .collect();
    let customer = CustomerTerms {
        demand: 3,
        min_start_time: 0,
        max_end_time: 86_400,
        service_duration: 600,
    };
    let b = CustomerTerms {
        max_end_time: b_max_end,
        ..customer
    };
    NetworkHandle(Arc::new(RoutingNetwork::new(
        vec![customer, b],
        1,
        1,
        seconds,
        meters,
    )))
}

fn terms(initial_km: i64, range_km: i64) -> VehicleTerms {
    VehicleTerms {
        capacity: 10,
        departure_time: 8 * 3_600,
        battery_range_m: range_km * KM,
        initial_charge_m: initial_km * KM,
        recharge_seconds: 1_800,
        reserve_charge_m: 0,
    }
}

#[test]
fn empty_route_drives_nowhere() {
    let summary = walk_route(&network(), 2, terms(50, 100), &[], None);
    assert_eq!(summary.distance_m, 0);
    assert_eq!(summary.return_time, 8 * 3_600);
}

#[test]
fn full_battery_needs_no_stop() {
    let summary = walk_route(&network(), 2, terms(200, 200), &[0, 1], None);
    assert!(summary.charges.is_empty());
    assert_eq!(summary.shortfall_m, 0);
    assert_eq!(summary.distance_m, 160 * KM);
    assert_eq!(summary.driving_s, 16_000);
    assert_eq!(summary.demand, 6);
}

#[test]
fn low_battery_charges_twice_on_the_way() {
    // 50 km at departure, 100 km per charge, 160 km route: one stop cannot
    // cover it, two stops on the charger at 45 km (outbound A->B and on the
    // way back B->depot) can, with no detour because it lies on the road.
    let mut events = Vec::new();
    let summary = walk_route(&network(), 2, terms(50, 100), &[0, 1], Some(&mut events));
    assert_eq!(summary.shortfall_m, 0);
    let legs: Vec<usize> = summary.charges.iter().map(|stop| stop.leg).collect();
    assert_eq!(legs, vec![1, 2]);
    assert_eq!(summary.distance_m, 160 * KM);
    assert_eq!(summary.charging_s, 3_600);

    let kinds: Vec<RouteEventKind> = events.iter().map(|event| event.kind).collect();
    assert_eq!(
        kinds,
        vec![
            RouteEventKind::Depart,
            RouteEventKind::Visit,
            RouteEventKind::Charge,
            RouteEventKind::Visit,
            RouteEventKind::Charge,
            RouteEventKind::Return,
        ]
    );
    // Arrive at A after 40 km, unload 10 min, 5 km to the charger.
    assert_eq!(events[1].arrival, 8 * 3_600 + 4_000);
    assert_eq!(events[2].arrival, events[1].end + 500);
    assert_eq!(events[2].charge_on_arrival_m, 5 * KM);
    assert_eq!(events[2].end, events[2].arrival + 1_800);
    // The van reaches B with 100 - 35 = 65 km left.
    assert_eq!(events[3].charge_on_arrival_m, 65 * KM);
    assert_eq!(summary.return_time, events[5].arrival);
}

#[test]
fn hopeless_battery_reports_shortfall_in_metres() {
    // 20 km to start and only 30 km per charge: the 40 km leg to A cannot be
    // driven, and B is 35 km from the only charger.
    let summary = walk_route(&network(), 2, terms(20, 30), &[0, 1], None);
    assert!(summary.shortfall_m > 0);
}

#[test]
fn lateness_counts_service_end_after_window() {
    let net = network_with_b_closing(8 * 3_600 + 9_000);
    let summary = walk_route(&net, 2, terms(200, 200), &[0, 1], None);
    // A: arrive 4 000 s, leave 4 600 s; B: arrive 8 600 s, leave 9 200 s.
    assert_eq!(summary.late_s, 200);
}

#[test]
fn reserve_is_never_spent() {
    // 50 km at departure with a 10 km reserve leaves 40 km usable: the van
    // reaches A (40 km) exactly at the reserve, so it may not detour 5 km
    // further to the charger without dipping below it.
    let reserve = VehicleTerms {
        reserve_charge_m: 10 * KM,
        ..terms(50, 100)
    };
    let summary = walk_route(&network(), 2, reserve, &[0, 1], None);
    assert_eq!(summary.shortfall_m, 5 * KM);

    let mut events = Vec::new();
    let roomy = VehicleTerms {
        reserve_charge_m: 5 * KM,
        ..terms(50, 100)
    };
    let summary = walk_route(&network(), 2, roomy, &[0, 1], Some(&mut events));
    assert_eq!(summary.shortfall_m, 0);
    assert!(events.iter().all(|event| event.charge_on_arrival_m >= 5 * KM));
}
