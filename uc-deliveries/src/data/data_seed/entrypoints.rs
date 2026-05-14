//! Public dataset ids and generator dispatch for delivery demos.
//!
//! City-specific modules only contain depots and visit groups. This file turns
//! those static fixtures into normalized `Plan` values with vehicles, delivery
//! ids, routing mode, and deterministic service windows.

use std::str::FromStr;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::types::{LocationData, VEHICLE_NAMES};
use super::{firenze, hartford, philadelphia};
use crate::domain::{Delivery, Plan, RoutingMode, Vehicle};

#[derive(Debug, Clone, Copy)]
pub enum DemoData {
    Philadelphia,
    Hartford,
    Firenze,
}

impl DemoData {
    pub fn id(self) -> &'static str {
        match self {
            DemoData::Philadelphia => "PHILADELPHIA",
            DemoData::Hartford => "HARTFORD",
            DemoData::Firenze => "FIRENZE",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DemoData::Philadelphia => "Philadelphia",
            DemoData::Hartford => "Hartford",
            DemoData::Firenze => "Firenze",
        }
    }
}

impl FromStr for DemoData {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "PHILADELPHIA" => Ok(DemoData::Philadelphia),
            "HARTFORD" => Ok(DemoData::Hartford),
            "FIRENZE" => Ok(DemoData::Firenze),
            _ => Err(()),
        }
    }
}

pub fn generate(demo: DemoData) -> Plan {
    match demo {
        DemoData::Philadelphia => generate_demo_data(
            demo,
            0,
            philadelphia::DEPOTS,
            philadelphia::VISIT_GROUPS,
            6 * 3600,
            36,
            48,
        ),
        DemoData::Hartford => generate_demo_data(
            demo,
            1,
            hartford::DEPOTS,
            hartford::VISIT_GROUPS,
            6 * 3600,
            24,
            34,
        ),
        DemoData::Firenze => generate_demo_data(
            demo,
            2,
            firenze::DEPOTS,
            firenze::VISIT_GROUPS,
            6 * 3600,
            38,
            52,
        ),
    }
}

/// Builds one city plan from static stops and deterministic vehicle settings.
fn generate_demo_data(
    demo: DemoData,
    seed: u64,
    depots: &[LocationData],
    stop_groups: &[&[LocationData]],
    departure_time: i64,
    min_capacity: i32,
    max_capacity: i32,
) -> Plan {
    let mut rng = StdRng::seed_from_u64(seed);
    let vehicles = depots
        .iter()
        .enumerate()
        .map(|(idx, depot)| {
            Vehicle::new(
                idx,
                VEHICLE_NAMES[idx % VEHICLE_NAMES.len()],
                rng.random_range(min_capacity..=max_capacity),
                depot.lat,
                depot.lng,
                departure_time,
            )
        })
        .collect::<Vec<_>>();

    let deliveries = stop_groups
        .iter()
        .flat_map(|stops| stops.iter())
        .enumerate()
        .map(|(idx, location)| {
            let (kind, min_start_time, max_end_time, demand_range, service_range) =
                location.customer_type.profile();
            Delivery::new(
                idx,
                location.name,
                kind,
                (location.lat, location.lng),
                rng.random_range(demand_range.0..=demand_range.1),
                (min_start_time, max_end_time),
                rng.random_range(service_range.0..=service_range.1),
            )
        })
        .collect::<Vec<_>>();

    let mut plan = Plan::new(demo.label(), deliveries, vehicles);
    plan.routing_mode = RoutingMode::RoadNetwork;
    plan
}
