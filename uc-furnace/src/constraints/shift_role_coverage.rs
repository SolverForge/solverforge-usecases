use super::shared::*;

/// Requires each shift to meet its configured role coverage demand.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .shift_coverage_demands()
        .project(ShiftCoverageEntries)
        .merge(
            ConstraintFactory::<Plan, HardSoftScore>::new()
                .operator_shift_assignments()
                .project(OperatorShiftCoverageEntries),
        )
        .group_by(
            |entry: &ShiftCoverageEntry| (entry.shift_id, entry.role),
            sum(|entry: &ShiftCoverageEntry| entry.delta),
        )
        .penalize(hard_weight(
            |_key: &(usize, OperatorRole), shortage: &i64| {
                HardSoftScore::of_hard((*shortage).max(0) * 900)
            },
        ))
        .named("shift_role_coverage")
}
