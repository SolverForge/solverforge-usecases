use solverforge::prelude::*;

use super::constants::RosterDay;
use super::enums::{
    FurnaceType, HeatTreatmentProcess, OperatorRole, OperatorSkill, PriorityBand, ShiftType,
};
use super::support::{ShiftCoverageEntry, ShiftIdentity, SkillMask};
use super::time::{canonical_shift_label, canonical_shift_time_label};

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

#[problem_fact]
pub struct Operator {
    #[planning_id]
    pub id: usize,
    pub name: &'static str,
    pub role: OperatorRole,
    pub day_only: bool,
    pub skills: &'static [OperatorSkill],
    pub skill_mask: SkillMask,
}

impl Operator {
    pub fn finalize(&mut self) {}
}

#[problem_fact]
pub struct Shift {
    #[planning_id]
    pub id: usize,
    pub roster_day: RosterDay,
    pub shift_type: ShiftType,
    pub start_minute: usize,
    pub end_minute: usize,
}

impl Shift {
    pub fn identity(&self) -> ShiftIdentity {
        ShiftIdentity {
            shift_id: self.id,
            roster_day: self.roster_day,
            shift_type: self.shift_type,
        }
    }

    pub fn label(&self) -> String {
        canonical_shift_label(self.roster_day, self.shift_type)
    }

    pub fn time_label(&self) -> String {
        canonical_shift_time_label(self.roster_day, self.shift_type, self.end_minute)
    }

    pub fn finalize(&mut self) {}
}

#[problem_fact]
pub struct ShiftCoverageDemand {
    #[planning_id]
    pub id: usize,
    pub shift_id: usize,
    pub roster_day: RosterDay,
    pub shift_type: ShiftType,
    pub role: OperatorRole,
    pub required_count: i64,
}

impl ShiftCoverageDemand {
    pub fn finalize(&mut self) {}
}

pub struct ShiftCoverageEntries;

impl Projection<ShiftCoverageDemand> for ShiftCoverageEntries {
    type Out = ShiftCoverageEntry;
    const MAX_EMITS: usize = 1;

    fn project<Sink>(&self, demand: &ShiftCoverageDemand, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        out.emit(ShiftCoverageEntry {
            shift_id: demand.shift_id,
            role: demand.role,
            delta: demand.required_count,
        });
    }
}
