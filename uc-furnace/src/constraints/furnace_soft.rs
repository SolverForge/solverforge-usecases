use super::shared::*;

/// Penalizes late jobs for one priority band using that band's lateness weight.
fn priority_lateness(
    name: &'static str,
    priority: PriorityBand,
    multiplier: i64,
) -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(move |assignment: &FurnaceAssignment| assignment.priority == priority)
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .end_minutes()
                .is_some_and(|end| end > assignment.due_datetime_minutes)
        })
        .penalize(move |assignment: &FurnaceAssignment| {
            priority_lateness_score(
                assignment.end_minutes().unwrap(),
                assignment.due_datetime_minutes,
                multiplier,
            )
            .unwrap()
        })
        .named(name)
}

/// Penalizes express work that finishes after its due time.
pub(super) fn express_lateness() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    priority_lateness("expressLateness", PriorityBand::Express, 60)
}

/// Penalizes urgent work that finishes after its due time.
pub(super) fn urgent_lateness() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    priority_lateness("urgentLateness", PriorityBand::Urgent, 20)
}

/// Penalizes standard-priority work that finishes after its due time.
pub(super) fn standard_lateness() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    priority_lateness("standardLateness", PriorityBand::Standard, 5)
}

/// Penalizes adjacent jobs on the same furnace for thermal changeover cost.
pub(super) fn thermal_changeover() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join(equal(|assignment: &FurnaceAssignment| {
            assignment.furnace_idx()
        }))
        .filter(|left: &FurnaceAssignment, right: &FurnaceAssignment| {
            left.work_order_idx != right.work_order_idx
                && right.start_minutes().unwrap() == left.end_minutes().unwrap()
        })
        .penalize(|left: &FurnaceAssignment, right: &FurnaceAssignment| {
            HardSoftScore::of_soft(calculate_changeover_cost_for_sequence(left, right) as i64)
        })
        .named("thermalChangeover")
}

/// Penalizes completing work earlier than necessary to reduce early-start pressure.
pub(super) fn early_start_pressure() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .end_minutes()
                .is_some_and(|end| end < assignment.due_datetime_minutes)
        })
        .penalize(|assignment: &FurnaceAssignment| {
            early_start_pressure_score(
                assignment.end_minutes().unwrap(),
                assignment.due_datetime_minutes,
            )
            .unwrap()
        })
        .named("earlyStartPressure")
}

/// Penalizes non-long-cycle processes that start on night shifts.
pub(super) fn night_start_preference() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .start_shift_identity()
                .is_some_and(|shift| shift.shift_type == ShiftType::Night)
        })
        .filter(|assignment: &FurnaceAssignment| {
            !matches!(
                assignment.process,
                crate::domain::HeatTreatmentProcess::Carburizing
                    | crate::domain::HeatTreatmentProcess::Nitriding
            )
        })
        .penalize(|assignment: &FurnaceAssignment| {
            HardSoftScore::of_soft(match assignment.priority {
                PriorityBand::Express => 40,
                PriorityBand::Urgent => 70,
                PriorityBand::Standard => 120,
            })
        })
        .named("nightStartPreference")
}
