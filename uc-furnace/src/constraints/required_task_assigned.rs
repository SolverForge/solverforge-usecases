use super::shared::*;

/// Requires every mandatory manual task on a scheduled job to have an operator assigned.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .project(MissingTaskOperatorViolations)
        .penalize(hard_weight(|violation: &HardViolation| {
            HardSoftScore::of_hard(violation.weight)
        }))
        .named("required_task_assigned")
}
