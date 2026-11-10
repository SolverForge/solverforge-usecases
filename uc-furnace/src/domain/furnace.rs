use solverforge::prelude::*;

use super::enums::{FurnaceType, HeatTreatmentProcess};

#[problem_fact]
pub struct Furnace {
    #[planning_id]
    pub id: usize,
    pub name: &'static str,
    pub max_temp_celsius: u32,
    pub max_load_kg: u32,
    pub furnace_type: FurnaceType,
    pub heating_rate_c_per_minute: u32,
    pub cooling_rate_c_per_minute: u32,
    pub supported_processes: &'static [HeatTreatmentProcess],
}

impl Furnace {
    pub fn finalize(&mut self) {}
}

impl Furnace {
    pub fn supports_process(&self, process: HeatTreatmentProcess) -> bool {
        self.supported_processes.contains(&process)
    }

    pub fn supports_temperature(&self, temperature_celsius: u32) -> bool {
        temperature_celsius <= self.max_temp_celsius
    }

    pub fn supports_load(&self, load_weight_kg: u32) -> bool {
        load_weight_kg <= self.max_load_kg
    }
}
