use super::shared::*;

/// Requires assigned task operators to hold the skills needed for their manual tasks.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .join((
            ConstraintFactory::<Plan, HardSoftScore>::new().operators(),
            assignment_uses_operator,
        ))
        .filter(|assignment: &FurnaceAssignment, operator: &Operator| {
            invalid_skill_count(assignment, operator) > 0
        })
        .penalize(hard_weight(
            |assignment: &FurnaceAssignment, operator: &Operator| {
                HardSoftScore::of_hard(invalid_skill_count(assignment, operator) as i64 * 900)
            },
        ))
        .named("task_operator_skilled")
}
