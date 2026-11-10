use solverforge::prelude::*;

use super::constants::RosterDay;
use super::enums::ShiftType;
use super::support::ShiftIdentity;
use super::time::{canonical_shift_label, canonical_shift_time_label};

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
