use super::shared::*;

/// Requires manual task assignments to reference a scheduled furnace job.
pub(super) fn assigned_tasks_require_scheduled_furnace(
) -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .project(AssignedTaskWithoutScheduleViolations)
        .penalize(hard_weight(|violation: &HardViolation| {
            HardSoftScore::of_hard(violation.weight)
        }))
        .named("assignedTasksRequireScheduledFurnace")
}

/// Requires every mandatory manual task on a scheduled job to have an operator assigned.
pub(super) fn required_task_assigned() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .project(MissingTaskOperatorViolations)
        .penalize(hard_weight(|violation: &HardViolation| {
            HardSoftScore::of_hard(violation.weight)
        }))
        .named("requiredTaskAssigned")
}

/// Requires assigned task operators to hold the skills needed for their manual tasks.
pub(super) fn task_operator_skilled() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("taskOperatorSkilled")
}

/// Requires assigned task operators to be rostered on the shift that owns the task.
pub(super) fn task_operator_on_owning_shift() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("taskOperatorOnOwningShift")
}

/// Prevents an operator from being assigned to overlapping manual tasks.
pub(super) fn operator_double_booked() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .project(ManualTaskBookingEntries)
        .join(equal(|booking: &ManualTaskBooking| booking.operator_id))
        .filter(|left: &ManualTaskBooking, right: &ManualTaskBooking| {
            left.identity_key() < right.identity_key() && left.overlaps(right)
        })
        .penalize(hard_weight(
            |left: &ManualTaskBooking, right: &ManualTaskBooking| {
                HardSoftScore::of_hard(booking_overlap_minutes(left, right) as i64 * 900)
            },
        ))
        .named("operatorDoubleBooked")
}

/// Requires each shift to meet its configured role coverage demand.
pub(super) fn shift_role_coverage() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("shiftRoleCoverage")
}

fn assignment_uses_operator(assignment: &FurnaceAssignment, operator: &Operator) -> bool {
    [
        assignment.load_build_operator_id,
        assignment.program_operator_id,
        assignment.quench_operator_id,
        assignment.unload_operator_id,
    ]
    .into_iter()
    .any(|operator_id| operator_id == Some(operator.id))
}

fn invalid_skill_count(assignment: &FurnaceAssignment, operator: &Operator) -> usize {
    [
        (ManualTaskKind::LoadBuild, assignment.load_build_operator_id),
        (ManualTaskKind::Program, assignment.program_operator_id),
        (ManualTaskKind::Quench, assignment.quench_operator_id),
        (assignment.unload_task_kind(), assignment.unload_operator_id),
    ]
    .into_iter()
    .filter(|(kind, operator_id)| {
        *operator_id == Some(operator.id)
            && assignment.task_interval(*kind).is_some()
            && !operator
                .skill_mask
                .contains_all(assignment.task_required_skills(*kind))
    })
    .count()
}

fn booking_overlap_minutes(left: &ManualTaskBooking, right: &ManualTaskBooking) -> usize {
    left.end_minute
        .min(right.end_minute)
        .saturating_sub(left.start_minute.max(right.start_minute))
}

#[cfg(test)]
mod tests {
    use crate::constraints::create_constraints;
    use crate::data::build_demo_plan;
    use crate::domain::{ManualTaskKind, OperatorRole, Plan, RosterDay, ShiftType};
    use solverforge::ConstraintSet;
    use std::collections::HashMap;

    fn hard_score(plan: &Plan, constraint_name: &str) -> i64 {
        create_constraints()
            .evaluate_each(plan)
            .into_iter()
            .find(|result| result.name == constraint_name)
            .expect("constraint result")
            .score
            .hard()
    }

    fn assign_role_count(plan: &mut Plan, shift_id: usize, role: OperatorRole, count: usize) {
        let shift = plan.shifts[shift_id].identity();
        let mut assigned = 0usize;
        for assignment in &mut plan.operator_shift_assignments {
            if assigned == count {
                break;
            }
            if assignment.role == role
                && assignment.roster_day == shift.roster_day
                && assignment.shift_type.is_none()
                && assignment
                    .allowed_shift_types
                    .contains(&shift.shift_type.index())
            {
                assignment.shift_type = Some(shift.shift_type.index());
                assigned += 1;
            }
        }
        assert_eq!(assigned, count);
    }

    fn schedule_operator_on_shift(plan: &mut Plan, operator_id: usize, shift_id: usize) {
        let shift = plan.shifts[shift_id].identity();
        let assignment = plan
            .operator_shift_assignments
            .iter_mut()
            .find(|assignment| {
                assignment.operator_idx == operator_id
                    && assignment.roster_day == shift.roster_day
                    && assignment
                        .allowed_shift_types
                        .contains(&shift.shift_type.index())
            })
            .expect("operator shift assignment");
        assert!(
            assignment.shift_type.is_none()
                || assignment.shift_type == Some(shift.shift_type.index()),
            "operator cannot be assigned to two shift types on the same roster day"
        );
        assignment.shift_type = Some(shift.shift_type.index());
    }

    fn pick_operator_for_task_shift(
        plan: &Plan,
        kind: ManualTaskKind,
        rostered: &HashMap<(usize, RosterDay), ShiftType>,
    ) -> usize {
        let shift = plan.assignments[0]
            .task_shift_identity(kind)
            .expect("task shift");
        plan.assignments[0]
            .task_eligible_operator_ids(kind)
            .iter()
            .copied()
            .find(|operator_id| {
                rostered
                    .get(&(*operator_id, shift.roster_day))
                    .is_none_or(|assigned_shift| *assigned_shift == shift.shift_type)
            })
            .expect("eligible operator without conflicting roster-day shift")
    }

    fn value_with_start(
        assignment: &crate::domain::FurnaceAssignment,
        start: usize,
    ) -> Option<usize> {
        assignment
            .compatible_assignments
            .iter()
            .copied()
            .find(|value| {
                (value % crate::domain::NUM_TIME_SLOTS) * crate::domain::TIME_STEP == start
            })
    }

    fn load_build_pair_with_start_delta(plan: &Plan, delta: usize) -> (usize, usize, usize, usize) {
        for left_index in 0..plan.assignments.len() {
            for right_index in (left_index + 1)..plan.assignments.len() {
                for left_value in &plan.assignments[left_index].compatible_assignments {
                    let left_start =
                        (left_value % crate::domain::NUM_TIME_SLOTS) * crate::domain::TIME_STEP;
                    if left_start < 60 {
                        continue;
                    }
                    let right_start = left_start + delta;
                    if let Some(right_value) =
                        value_with_start(&plan.assignments[right_index], right_start)
                    {
                        return (left_index, *left_value, right_index, right_value);
                    }
                }
            }
        }
        panic!("load-build pair with start delta {delta}");
    }

    #[test]
    fn shift_role_coverage_penalizes_missing_required_crew() {
        let plan = build_demo_plan();

        assert!(hard_score(&plan, "shiftRoleCoverage") < 0);
    }

    #[test]
    fn shift_role_coverage_allows_exact_required_crew() {
        let mut plan = build_demo_plan();
        let demands = plan.shift_coverage_demands.clone();
        for demand in demands {
            assign_role_count(
                &mut plan,
                demand.shift_id,
                demand.role,
                demand.required_count as usize,
            );
        }

        assert_eq!(hard_score(&plan, "shiftRoleCoverage"), 0);
    }

    #[test]
    fn task_operator_on_owning_shift_allows_task_operator_working_that_shift() {
        let mut plan = build_demo_plan();
        let value = plan.assignments[0].compatible_assignments[0];
        plan.assignments[0].assignment = Some(value);

        let mut rostered = HashMap::new();
        let task_kinds = plan.assignments[0].required_task_kinds();
        for kind in task_kinds {
            let shift = plan.assignments[0]
                .task_shift_identity(kind)
                .expect("task shift");
            let operator_id = pick_operator_for_task_shift(&plan, kind, &rostered);
            match kind {
                ManualTaskKind::LoadBuild => {
                    plan.assignments[0].load_build_operator_id = Some(operator_id)
                }
                ManualTaskKind::Program => {
                    plan.assignments[0].program_operator_id = Some(operator_id)
                }
                ManualTaskKind::Quench => {
                    plan.assignments[0].quench_operator_id = Some(operator_id)
                }
                ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => {
                    plan.assignments[0].unload_operator_id = Some(operator_id)
                }
            }
            let shift_id = plan.assignments[0]
                .task_shift_identity(kind)
                .expect("task shift")
                .shift_id;
            schedule_operator_on_shift(&mut plan, operator_id, shift_id);
            rostered.insert((operator_id, shift.roster_day), shift.shift_type);
        }

        assert_eq!(hard_score(&plan, "taskOperatorOnOwningShift"), 0);
    }

    #[test]
    fn operator_double_booked_penalizes_actual_overlap_minutes_once() {
        let mut plan = build_demo_plan();
        let (left_index, left_value, right_index, right_value) =
            load_build_pair_with_start_delta(&plan, 15);
        plan.assignments[left_index].assignment = Some(left_value);
        plan.assignments[right_index].assignment = Some(right_value);
        plan.assignments[left_index].load_build_operator_id = Some(0);
        plan.assignments[right_index].load_build_operator_id = Some(0);

        assert_eq!(hard_score(&plan, "operatorDoubleBooked"), -13_500);
    }

    #[test]
    fn operator_double_booked_allows_touching_manual_task_intervals() {
        let mut plan = build_demo_plan();
        let (left_index, left_value, right_index, right_value) =
            load_build_pair_with_start_delta(&plan, 30);
        plan.assignments[left_index].assignment = Some(left_value);
        plan.assignments[right_index].assignment = Some(right_value);
        plan.assignments[left_index].load_build_operator_id = Some(0);
        plan.assignments[right_index].load_build_operator_id = Some(0);

        assert_eq!(hard_score(&plan, "operatorDoubleBooked"), 0);
    }
}
