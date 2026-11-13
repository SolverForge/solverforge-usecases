//! Deterministic demo plans: electric vans delivering to pharmacies around
//! Bergamo from one distribution hub.
//!
//! Every value is fixed so a demo id always yields the same plan. Routes start
//! empty; building them is the solver's job.

use std::str::FromStr;

use crate::domain::{Charger, Customer, Plan, Vehicle, VehicleInit};

mod bergamo_places;

use bergamo_places::{CHARGERS, PHARMACIES};

/// The distribution hub every van leaves from and returns to (Stezzano,
/// south of Bergamo).
const HUB: (f64, f64) = (45.6700, 9.6560);

/// Seconds after midnight.
const fn clock(hours: i64, minutes: i64) -> i64 {
    hours * 3_600 + minutes * 60
}

/// Public demo ids served by `/demo-data`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoData {
    /// All 48 pharmacies, 6 vans.
    Standard,
}

const AVAILABLE_DEMO_DATA: &[DemoData] = &[DemoData::Standard];
const DEFAULT_DEMO_DATA: DemoData = DemoData::Standard;

pub fn default_demo_data() -> DemoData {
    DEFAULT_DEMO_DATA
}

pub fn available_demo_data() -> &'static [DemoData] {
    AVAILABLE_DEMO_DATA
}

impl DemoData {
    pub fn id(self) -> &'static str {
        match self {
            DemoData::Standard => "STANDARD",
        }
    }

    pub fn default_demo_data() -> Self {
        default_demo_data()
    }

    pub fn available_demo_data() -> &'static [Self] {
        available_demo_data()
    }
}

impl FromStr for DemoData {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "STANDARD" => Ok(DemoData::Standard),
            _ => Err(()),
        }
    }
}

/// Builds one demo plan.
pub fn generate(demo: DemoData) -> Plan {
    match demo {
        DemoData::Standard => build_plan(6),
    }
}

fn build_plan(vehicle_count: usize) -> Plan {
    let customers = PHARMACIES
        .iter()
        .enumerate()
        .map(|(idx, &(name, lat, lng))| {
            // Two of every three pharmacies take a morning delivery before
            // lunch closing; the rest take theirs after the afternoon opening.
            let window = if idx % 3 == 2 {
                (clock(14, 30), clock(18, 30))
            } else {
                (clock(8, 0), clock(12, 30))
            };
            let demand = 2 + (idx * 7 % 5) as i32;
            let service = 360 + (idx as i64 * 97 % 5) * 60;
            Customer::new(
                format!("pharmacy-{idx}"),
                name,
                (lat, lng),
                demand,
                window,
                service,
            )
        })
        .collect();

    let chargers = CHARGERS
        .iter()
        .enumerate()
        .map(|(idx, &(name, lat, lng))| Charger::new(format!("charger-{idx}"), name, lat, lng))
        .collect();

    let vehicles = (0..vehicle_count).map(van).collect();
    Plan::new(customers, chargers, vehicles)
}

/// Two van models alternate. Overnight charging at the hub is shared, so the
/// vans leave with different, mostly partial, charge levels. Every van keeps
/// a 10 km reserve it never plans to use.
fn van(idx: usize) -> Vehicle {
    const INITIAL_CHARGE_KM: [i64; 6] = [70, 45, 90, 55, 40, 80];
    let large = idx.is_multiple_of(2);
    Vehicle::new(VehicleInit {
        id: format!("van-{idx}"),
        name: format!("Van {}", idx + 1),
        capacity: if large { 40 } else { 28 },
        home: HUB,
        departure_time: clock(7, 30),
        battery_range_m: if large { 160_000 } else { 110_000 },
        initial_charge_m: INITIAL_CHARGE_KM[idx % INITIAL_CHARGE_KM.len()] * 1_000,
        recharge_seconds: if large { 45 * 60 } else { 35 * 60 },
        reserve_charge_m: 10_000,
    })
}

#[cfg(test)]
mod tests;
