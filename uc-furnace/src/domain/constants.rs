pub const TIME_STEP: usize = 15;
pub const NUM_TIME_SLOTS: usize = 672;
pub const HORIZON_MINUTES: usize = 10080;
pub const MINUTES_PER_DAY: usize = 1440;
pub const NUM_DAYS: usize = 7;
pub const STAFFING_ROSTER_DAY_START: i32 = -1;
pub const STAFFING_ROSTER_DAY_END: i32 = (NUM_DAYS as i32) - 1;
pub const CARRY_OUT_END_MINUTE: usize = HORIZON_MINUTES + 360;
pub const OFF_SHIFT_VALUE: usize = 3;

pub type RosterDay = i32;
pub type ShiftId = usize;
