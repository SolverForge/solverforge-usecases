use super::shared::*;

/// Requires assigned task operators to be rostered on the shift that owns the task.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .project(TaskShiftWorkEntries)
        .merge(
            ConstraintFactory::<Plan, HardSoftScore>::new()
                .operator_shift_assignments()
                .project(OperatorTaskShiftWorkEntries),
        )
        .group_by(
            |entry: &ShiftWorkEntry| (entry.operator_id, entry.shift_id),
            sum(|entry: &ShiftWorkEntry| entry.delta),
        )
        .penalize(hard_weight(|_key: &(usize, usize), shortage: &i64| {
            HardSoftScore::of_hard((*shortage).max(0) * 900)
        }))
        .named("task_operator_on_owning_shift")
}
