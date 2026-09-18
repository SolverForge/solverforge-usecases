pub(super) use solverforge::prelude::*;
pub(super) use solverforge::stream::collector::{count, sum};
pub(super) use solverforge::stream::joiner::{equal, equal_bi};
pub(super) use solverforge::IncrementalConstraint;

pub(super) use crate::domain::{
    calculate_changeover_cost_for_sequence, has_insufficient_rest, shift_rest_minutes,
    AssignedTaskCapacityEntries, AssignedTaskWithoutScheduleViolations, Furnace, FurnaceAssignment,
    HardViolation, ManualTaskBooking, ManualTaskBookingEntries, ManualTaskKind,
    MissingTaskOperatorViolations, MonitoringDemandEntries, MonitoringEntry, MonitoringLoad,
    NightWindowEntry, Operator, OperatorMonitoringCapacityEntries, OperatorNightWindowEntries,
    OperatorRole, OperatorShiftAssignment, OperatorShiftCoverageEntries,
    OperatorTaskShiftWorkEntries, Plan, PlanConstraintStreams, PriorityBand, ShiftCoverageEntries,
    ShiftCoverageEntry, ShiftType, ShiftWorkEntry, TaskShiftWorkEntries, HORIZON_MINUTES,
    MAX_CONSECUTIVE_NIGHTS, MAX_VISIBLE_SHIFTS_PER_WEEK, MINIMUM_REST_MINUTES,
    TARGET_VISIBLE_SHIFTS_PER_WEEK,
};

/// Returns the missing changeover minutes when two jobs are too close together.
pub(super) fn changeover_shortage(
    left_end: usize,
    right_start: usize,
    required_gap: usize,
) -> Option<usize> {
    (right_start >= left_end && right_start - left_end < required_gap)
        .then(|| required_gap - (right_start - left_end))
}

/// Scores lateness for a job that finishes after its due time.
pub(super) fn priority_lateness_score(
    end_minutes: usize,
    due_datetime_minutes: usize,
    multiplier: i64,
) -> Option<HardSoftScore> {
    (end_minutes > due_datetime_minutes)
        .then(|| HardSoftScore::of_soft(((end_minutes - due_datetime_minutes) as i64) * multiplier))
}

/// Scores the pressure from finishing a job before its due time.
pub(super) fn early_start_pressure_score(
    end_minutes: usize,
    due_datetime_minutes: usize,
) -> Option<HardSoftScore> {
    (end_minutes < due_datetime_minutes)
        .then(|| HardSoftScore::of_soft(((due_datetime_minutes - end_minutes) as i64) / 40))
}
