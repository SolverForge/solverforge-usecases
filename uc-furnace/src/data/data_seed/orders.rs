use rand::rngs::SmallRng;
use rand::RngExt;

use crate::domain::requires_quench;
use crate::domain::HeatTreatmentProcess::*;
use crate::domain::{PriorityBand, WorkOrder};

use super::super::constants::{pick, rand_range, CUSTOMERS};
use super::super::specs::PROCESS_SPECS;
use super::catalog::DemoDataProfile;

pub(super) fn build_work_orders(rng: &mut SmallRng, profile: DemoDataProfile) -> Vec<WorkOrder> {
    let total_orders = profile.order_counts.iter().map(|(_, count)| *count).sum();
    let mut work_orders = Vec::with_capacity(total_orders);
    let mut id = 0usize;
    let mut order_seq = 1400usize;

    for &(spec_idx, count) in profile.order_counts {
        let spec = &PROCESS_SPECS[spec_idx];

        for sequence in 0..count {
            let customer = *pick(rng, CUSTOMERS);
            let part_description = *pick(rng, spec.parts);
            let material = *pick(rng, spec.materials);

            let mut soak_time_minutes = rand_range(rng, spec.soak_min, spec.soak_max);
            let mut load_weight_kg = rand_range(rng, spec.load_min, spec.load_max);

            if spec.process == Tempering && sequence % 4 == 0 {
                soak_time_minutes = rand_range(rng, 40, 90);
            }

            if spec.process == StressRelieving && sequence % 6 == 0 {
                load_weight_kg = if sequence % 12 == 0 { 850 } else { 950 };
            }

            let due_datetime_minutes = {
                let base = profile.due_windows[rng.random_range(0..profile.due_windows.len())];
                let slack =
                    profile.due_slack_minutes[rng.random_range(0..profile.due_slack_minutes.len())];
                (base + slack).min(10020)
            };

            let priority = match id % profile.express_period {
                0 => PriorityBand::Express,
                value if value <= profile.urgent_width => PriorityBand::Urgent,
                _ => PriorityBand::Standard,
            };

            let order_code = format!("WO-2026-{}{:04}", spec.order_prefix, order_seq);
            let order_code: &'static str = Box::leak(order_code.into_boxed_str());

            work_orders.push(WorkOrder {
                id,
                order_code,
                customer,
                part_description,
                material,
                process: spec.process,
                temperature_celsius: rand_range(rng, spec.temp_min, spec.temp_max),
                soak_time_minutes,
                load_weight_kg,
                due_datetime_minutes,
                priority,
                requires_quench: requires_quench(spec.process),
            });

            id += 1;
            order_seq += 1;
        }
    }

    work_orders
}
