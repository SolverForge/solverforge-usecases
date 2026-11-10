use solverforge::prelude::*;

use super::constants::RosterDay;
use super::enums::{OperatorRole, ShiftType};
use super::support::ShiftCoverageEntry;

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
