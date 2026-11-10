use super::shared::*;

/// Penalizes non-long-cycle processes that start on night shifts.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("night_start_preference")
}
