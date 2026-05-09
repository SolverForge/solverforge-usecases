use crate::domain::{Employee, Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::stream::{source, ChangeSource};
use solverforge::IncrementalConstraint;

const SCORE_SCALE: i64 = 100_000;
const STRUCTURAL_MINUTE_HARD_UNITS: i64 = 20;

/// Hard-penalizes unavailable-date overlap, scaled by overlapping minutes.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .filter(|shift: &Shift| shift.employee_idx.is_some())
        .join((
            source(Plan::employees_slice, ChangeSource::Static),
            joiner::equal_bi(
                |shift: &Shift| shift.employee_idx,
                |employee: &Employee| Some(employee.index),
            ),
        ))
        .filter(|shift: &Shift, employee: &Employee| {
            employee.unavailable_days.iter().any(|date| {
                let day_start = date.and_hms_opt(0, 0, 0).unwrap();
                let day_end = date
                    .succ_opt()
                    .unwrap_or(*date)
                    .and_hms_opt(0, 0, 0)
                    .unwrap();
                let overlap_start = shift.start.max(day_start);
                let overlap_end = shift.end.min(day_end);
                overlap_start < overlap_end
            })
        })
        .penalize_hard_with(|shift: &Shift, employee: &Employee| {
            let overlap_minutes: i64 = employee
                .unavailable_days
                .iter()
                .map(|date| {
                    let day_start = date.and_hms_opt(0, 0, 0).unwrap();
                    let day_end = date
                        .succ_opt()
                        .unwrap_or(*date)
                        .and_hms_opt(0, 0, 0)
                        .unwrap();
                    let overlap_start = shift.start.max(day_start);
                    let overlap_end = shift.end.min(day_end);
                    if overlap_start < overlap_end {
                        (overlap_end - overlap_start).num_minutes()
                    } else {
                        0
                    }
                })
                .sum();
            HardSoftDecimalScore::of_hard_scaled(
                overlap_minutes * STRUCTURAL_MINUTE_HARD_UNITS * SCORE_SCALE,
            )
        })
        .named("Unavailable employee")
}
