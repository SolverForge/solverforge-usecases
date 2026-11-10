use solverforge::prelude::*;

use super::enums::{HeatTreatmentProcess, PriorityBand};

#[problem_fact]
pub struct WorkOrder {
    #[planning_id]
    pub id: usize,
    pub order_code: &'static str,
    pub customer: &'static str,
    pub part_description: &'static str,
    pub material: &'static str,
    pub process: HeatTreatmentProcess,
    pub temperature_celsius: u32,
    pub soak_time_minutes: u32,
    pub load_weight_kg: u32,
    pub due_datetime_minutes: usize,
    pub priority: PriorityBand,
    pub requires_quench: bool,
}

impl WorkOrder {
    pub fn finalize(&mut self) {}
}
