use super::shared::*;

/// Requires manual task assignments to reference a scheduled furnace job.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .project(AssignedTaskWithoutScheduleViolations)
        .penalize(hard_weight(|violation: &HardViolation| {
            HardSoftScore::of_hard(violation.weight)
        }))
        .named("assigned_tasks_require_scheduled_furnace")
}
