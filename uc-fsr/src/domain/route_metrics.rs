//! Shared route measurements stored as technician-route shadow values.
//!
//! SolverForge calls each stock constraint separately, but the business
//! concepts overlap: travel, time windows, skills, parts, overtime, and priority
//! slack all require walking the same ordered visit list. This module
//! centralizes that walk so route entities can expose simple shadow fields to
//! the constraint builders.

use crate::domain::{FieldServicePlan, ServiceVisit, TechnicianRoute, TravelLeg};

/// Aggregated measurements for one technician route.
///
/// Individual constraints reuse this struct so each business rule can stay
/// small. For example, the time-window constraint reads `late_minutes`, while
/// the travel minimization rule reads `travel_seconds` and `distance_meters`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteStats {
    pub invalid_visits: i64,
    pub valid_visits: i64,
    pub scored_travel_legs: i64,
    pub unreachable_legs: i64,
    pub missing_skill_visits: i64,
    pub missing_part_visits: i64,
    pub late_visits: i64,
    pub late_minutes: i64,
    pub overtime_minutes: i64,
    pub travel_seconds: i64,
    pub distance_meters: i64,
    pub service_minutes: i64,
    pub waiting_minutes: i64,
    pub route_minutes: i64,
    pub finish_minute: i32,
    pub territory_matches: i64,
    pub priority_slack: i64,
}

#[derive(Debug, Clone, Copy)]
struct VisitTiming {
    visit_idx: usize,
    service_start: i32,
}

pub fn route_stats(plan: &FieldServicePlan, route: &TechnicianRoute) -> RouteStats {
    let mut stats = RouteStats {
        finish_minute: route.shift_start_minute,
        ..RouteStats::default()
    };
    let mut clock = route.shift_start_minute;
    let mut previous_location = route.start_location_idx;
    let mut timings = Vec::with_capacity(route.visits.len());

    // Walk the route in visit order. This mirrors how a technician would drive:
    // depot to first visit, visit to visit, then back to the end depot.
    for &visit_idx in &route.visits {
        let Some(visit) = plan.service_visits.get(visit_idx) else {
            stats.invalid_visits += 1;
            continue;
        };
        stats.valid_visits += 1;

        apply_leg(
            plan,
            previous_location,
            visit.location_idx,
            &mut clock,
            &mut stats,
        );

        // Waiting is allowed and soft-neutral; lateness is a hard feasibility
        // problem scored by the time-window constraint.
        if clock < visit.earliest_minute {
            stats.waiting_minutes += i64::from(visit.earliest_minute - clock);
            clock = visit.earliest_minute;
        }
        if clock > visit.latest_minute {
            stats.late_visits += 1;
            stats.late_minutes += i64::from(clock - visit.latest_minute);
        }

        if !mask_contains(route.skill_mask, visit.required_skill_mask) {
            stats.missing_skill_visits += 1;
        }
        if !mask_contains(route.inventory_mask, visit.required_parts_mask) {
            stats.missing_part_visits += 1;
        }
        if route.territory == visit.territory {
            stats.territory_matches += 1;
        }

        timings.push(VisitTiming {
            visit_idx,
            service_start: clock,
        });

        let service_minutes = visit.duration_minutes.max(0);
        stats.service_minutes += i64::from(service_minutes);
        clock = clock.saturating_add(service_minutes);
        previous_location = visit.location_idx;
    }

    apply_leg(
        plan,
        previous_location,
        route.end_location_idx,
        &mut clock,
        &mut stats,
    );

    stats.finish_minute = clock;
    stats.route_minutes = i64::from(clock.saturating_sub(route.shift_start_minute));
    stats.overtime_minutes = i64::from((clock - route.shift_end_minute).max(0))
        + (stats.route_minutes - i64::from(route.max_route_minutes)).max(0);
    stats.priority_slack = priority_slack(plan, &timings);
    stats
}

pub fn leg_for(
    plan: &FieldServicePlan,
    from_location_idx: usize,
    to_location_idx: usize,
) -> Option<&TravelLeg> {
    let width = plan.locations.len();
    // Travel legs are normally stored as a dense row-major matrix. The secondary
    // scan keeps tests and sparse diagnostics readable without changing the
    // public fact shape.
    let direct_idx = from_location_idx
        .checked_mul(width)?
        .checked_add(to_location_idx)?;

    if let Some(leg) = plan.travel_legs.get(direct_idx) {
        if leg.from_location_idx == from_location_idx && leg.to_location_idx == to_location_idx {
            return Some(leg);
        }
    }

    plan.travel_legs.iter().find(|leg| {
        leg.from_location_idx == from_location_idx && leg.to_location_idx == to_location_idx
    })
}

fn apply_leg(
    plan: &FieldServicePlan,
    from_location_idx: usize,
    to_location_idx: usize,
    clock: &mut i32,
    stats: &mut RouteStats,
) {
    let Some(leg) = leg_for(plan, from_location_idx, to_location_idx) else {
        stats.unreachable_legs += 1;
        return;
    };

    if !leg.reachable {
        stats.unreachable_legs += 1;
        return;
    }

    // Scoring uses seconds for precision but the route clock advances in whole
    // minutes because visits and shifts are modeled on a minute calendar.
    stats.travel_seconds += leg.duration_seconds.max(0);
    stats.distance_meters += leg.distance_meters.max(0);
    if leg.duration_seconds > 0 || leg.distance_meters > 0 {
        stats.scored_travel_legs += 1;
    }
    *clock = clock.saturating_add(div_ceil(leg.duration_seconds.max(0), 60) as i32);
}

fn priority_slack(plan: &FieldServicePlan, timings: &[VisitTiming]) -> i64 {
    timings
        .iter()
        .filter_map(|timing| {
            plan.service_visits
                .get(timing.visit_idx)
                .map(|visit| visit_priority_slack(visit, timing.service_start))
        })
        .sum()
}

fn visit_priority_slack(visit: &ServiceVisit, service_start: i32) -> i64 {
    let slack_quarters = i64::from((visit.latest_minute - service_start).max(0) / 15);
    i64::from(visit.priority.max(1)) * (slack_quarters + 1)
}

fn mask_contains(available: i64, required: i64) -> bool {
    (available & required) == required
}

fn div_ceil(value: i64, divisor: i64) -> i64 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}
