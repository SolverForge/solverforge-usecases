use super::shared::*;

/// Requires enough monitor-capable operator capacity for all active furnace monitoring demand.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    let factory = ConstraintFactory::<Plan, HardSoftScore>::new();
    factory
        .assignments()
        .project(MonitoringDemandEntries)
        .merge(
            ConstraintFactory::<Plan, HardSoftScore>::new()
                .operator_shift_assignments()
                .project(OperatorMonitoringCapacityEntries),
        )
        .merge(
            ConstraintFactory::<Plan, HardSoftScore>::new()
                .assignments()
                .project(AssignedTaskCapacityEntries),
        )
        .group_by(
            |entry: &MonitoringEntry| entry.bucket,
            sum(|entry: &MonitoringEntry| entry.load),
        )
        .penalize(hard_weight(|_bucket: &usize, load: &MonitoringLoad| {
            let demand = load.required_operators();
            HardSoftScore::of_hard((demand - load.capacity).max(0) * 900)
        }))
        .named("monitoring_capacity")
}
