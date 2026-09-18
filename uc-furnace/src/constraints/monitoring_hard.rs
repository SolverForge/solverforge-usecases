use super::shared::*;

/// Requires enough monitor-capable operator capacity for all active furnace monitoring demand.
pub(super) fn monitoring_capacity() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("monitoringCapacity")
}

#[cfg(test)]
mod tests {
    use crate::constraints::create_constraints;
    use crate::data::build_demo_plan;
    use crate::domain::{OperatorSkill, Plan};
    use solverforge::ConstraintSet;

    fn monitoring_hard(plan: &Plan) -> i64 {
        create_constraints()
            .evaluate_each(plan)
            .into_iter()
            .find(|result| result.name == "monitoringCapacity")
            .expect("monitoring constraint result")
            .score
            .hard()
    }

    fn monitoring_score(plan: &Plan) -> i64 {
        monitoring_hard(plan)
    }

    fn schedule_first_order(plan: &mut Plan) {
        let value = plan.assignments[0].compatible_assignments[0];
        plan.assignments[0].assignment = Some(value);
    }

    fn assign_monitoring_shift(plan: &mut Plan, assignment_idx: usize, count: usize) {
        let shifts = plan.assignments[assignment_idx].occupied_shift_identities();
        assert!(
            !shifts.is_empty(),
            "scheduled assignment should occupy at least one shift"
        );

        for shift in shifts {
            let mut assigned = 0usize;
            for assignment in &mut plan.operator_shift_assignments {
                if assigned == count {
                    break;
                }
                if assignment.roster_day == shift.roster_day
                    && (assignment.shift_type.is_none()
                        || assignment.shift_type == Some(shift.shift_type.index()))
                    && assignment.skill_mask.contains(OperatorSkill::Monitor)
                    && assignment
                        .allowed_shift_types
                        .contains(&shift.shift_type.index())
                {
                    assignment.shift_type = Some(shift.shift_type.index());
                    assigned += 1;
                }
            }
            assert_eq!(
                assigned, count,
                "demo should have enough monitoring operators"
            );
        }
    }

    #[test]
    fn monitoring_capacity_penalizes_shortage() {
        let mut plan = build_demo_plan();
        schedule_first_order(&mut plan);

        assert_ne!(monitoring_score(&plan), 0);
    }

    #[test]
    fn monitoring_capacity_allows_exact_capacity() {
        let mut plan = build_demo_plan();
        schedule_first_order(&mut plan);
        assign_monitoring_shift(&mut plan, 0, 2);

        assert_eq!(monitoring_score(&plan), 0);
    }

    #[test]
    fn monitoring_capacity_allows_surplus_capacity() {
        let mut plan = build_demo_plan();
        schedule_first_order(&mut plan);
        assign_monitoring_shift(&mut plan, 0, 3);

        assert_eq!(monitoring_score(&plan), 0);
    }

    #[test]
    fn monitoring_capacity_counts_manual_task_consumption() {
        let base = build_demo_plan();

        for assignment_idx in 0..base.assignments.len() {
            for capacity in 1..=3 {
                let mut plan = base.clone();
                plan.assignments[assignment_idx].assignment = plan.assignments[assignment_idx]
                    .compatible_assignments
                    .first()
                    .copied();
                assign_monitoring_shift(&mut plan, assignment_idx, capacity);
                if monitoring_score(&plan) != 0 {
                    continue;
                }

                plan.assignments[assignment_idx].load_build_operator_id = plan.assignments
                    [assignment_idx]
                    .eligible_load_build_operator_ids
                    .first()
                    .copied();
                if monitoring_score(&plan) != 0 {
                    return;
                }
            }
        }

        panic!("demo should contain a schedule where manual task consumption creates shortage");
    }

    #[test]
    fn monitoring_capacity_retracts_updated_assignment() {
        let mut plan = build_demo_plan();
        schedule_first_order(&mut plan);
        assert_ne!(monitoring_hard(&plan), 0);

        plan.assignments[0].assignment = None;

        assert_eq!(monitoring_hard(&plan), 0);
    }
}
