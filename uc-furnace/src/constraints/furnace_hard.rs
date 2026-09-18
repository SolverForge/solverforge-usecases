use super::shared::*;

/// Penalizes work orders that have not been assigned to a furnace and start time.
pub(super) fn unassigned_orders() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_none())
        .penalize(hard_weight(|_: &FurnaceAssignment| {
            HardSoftScore::of_hard(30_000)
        }))
        .named("unassignedOrders")
}

/// Penalizes assignments scheduled on furnaces that cannot run the required process.
pub(super) fn incompatible_process() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join((
            ConstraintFactory::<Plan, HardSoftScore>::new().furnaces(),
            equal_bi(
                |assignment: &FurnaceAssignment| assignment.furnace_idx(),
                |furnace: &Furnace| Some(furnace.id),
            ),
        ))
        .filter(|assignment: &FurnaceAssignment, furnace: &Furnace| {
            !furnace.supports_process(assignment.process)
        })
        .penalize(hard_weight(|_: &FurnaceAssignment, _: &Furnace| {
            HardSoftScore::of_hard(4_000)
        }))
        .named("incompatibleProcess")
}

/// Penalizes assignments whose required temperature exceeds the selected furnace limit.
pub(super) fn excessive_temperature() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join((
            ConstraintFactory::<Plan, HardSoftScore>::new().furnaces(),
            equal_bi(
                |assignment: &FurnaceAssignment| assignment.furnace_idx(),
                |furnace: &Furnace| Some(furnace.id),
            ),
        ))
        .filter(|assignment: &FurnaceAssignment, furnace: &Furnace| {
            !furnace.supports_temperature(assignment.temperature_celsius)
        })
        .penalize(hard_weight(
            |assignment: &FurnaceAssignment, furnace: &Furnace| {
                HardSoftScore::of_hard(
                    (assignment.temperature_celsius - furnace.max_temp_celsius) as i64 * 20,
                )
            },
        ))
        .named("excessiveTemperature")
}

/// Penalizes assignments whose load weight exceeds the selected furnace capacity.
pub(super) fn excessive_load() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join((
            ConstraintFactory::<Plan, HardSoftScore>::new().furnaces(),
            equal_bi(
                |assignment: &FurnaceAssignment| assignment.furnace_idx(),
                |furnace: &Furnace| Some(furnace.id),
            ),
        ))
        .filter(|assignment: &FurnaceAssignment, furnace: &Furnace| {
            !furnace.supports_load(assignment.load_weight_kg)
        })
        .penalize(hard_weight(
            |assignment: &FurnaceAssignment, furnace: &Furnace| {
                HardSoftScore::of_hard(
                    (assignment.load_weight_kg - furnace.max_load_kg) as i64 * 10,
                )
            },
        ))
        .named("excessiveLoad")
}

/// Penalizes scheduled work that finishes beyond the weekly planning horizon.
pub(super) fn outside_week() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .end_minutes()
                .is_some_and(|end| end > HORIZON_MINUTES)
        })
        .penalize(hard_weight(|assignment: &FurnaceAssignment| {
            HardSoftScore::of_hard((assignment.end_minutes().unwrap() - HORIZON_MINUTES) as i64 * 5)
        }))
        .named("outsideWeek")
}

/// Penalizes overlapping jobs assigned to the same furnace.
pub(super) fn furnace_overlap() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join(equal(|assignment: &FurnaceAssignment| {
            assignment.furnace_idx()
        }))
        .filter(|left: &FurnaceAssignment, right: &FurnaceAssignment| {
            if left.work_order_idx >= right.work_order_idx {
                return false;
            }
            let left_start = left.start_minutes().unwrap();
            let left_end = left.end_minutes().unwrap();
            let right_start = right.start_minutes().unwrap();
            let right_end = right.end_minutes().unwrap();
            left_start < right_end && right_start < left_end
        })
        .penalize(hard_weight(
            |left: &FurnaceAssignment, right: &FurnaceAssignment| {
                let overlap_start = left
                    .start_minutes()
                    .unwrap()
                    .max(right.start_minutes().unwrap());
                let overlap_end = left
                    .end_minutes()
                    .unwrap()
                    .min(right.end_minutes().unwrap());
                HardSoftScore::of_hard((overlap_end - overlap_start) as i64 * 20)
            },
        ))
        .named("furnaceOverlap")
}

/// Penalizes back-to-back furnace jobs that leave less than the required changeover gap.
pub(super) fn changeover_gap() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join((
            ConstraintFactory::<Plan, HardSoftScore>::new().assignments(),
            equal_bi(
                |left: &FurnaceAssignment| left.furnace_idx(),
                |right: &FurnaceAssignment| right.furnace_idx(),
            ),
        ))
        .filter(|left: &FurnaceAssignment, right: &FurnaceAssignment| {
            left.work_order_idx != right.work_order_idx
                && right.assignment.is_some()
                && left.end_minutes().unwrap() <= right.start_minutes().unwrap()
                && changeover_shortage(
                    left.end_minutes().unwrap(),
                    right.start_minutes().unwrap(),
                    calculate_changeover_cost_for_sequence(left, right) as usize,
                )
                .is_some()
        })
        .penalize(hard_weight(
            |left: &FurnaceAssignment, right: &FurnaceAssignment| {
                HardSoftScore::of_hard(
                    changeover_shortage(
                        left.end_minutes().unwrap(),
                        right.start_minutes().unwrap(),
                        calculate_changeover_cost_for_sequence(left, right) as usize,
                    )
                    .unwrap() as i64
                        * 10,
                )
            },
        ))
        .named("changeoverGap")
}

/// Requires load-build work to be owned by the shift covering that task interval.
pub(super) fn load_build_shift_ownership() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    task_shift_ownership(
        "loadBuildShiftOwnership",
        |assignment| assignment.load_build_interval(),
        |assignment| assignment.load_build_shift_identity(),
    )
}

/// Requires programming work to be owned by the shift covering that task interval.
pub(super) fn program_shift_ownership() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    task_shift_ownership(
        "programShiftOwnership",
        |assignment| assignment.program_interval(),
        |assignment| assignment.program_shift_identity(),
    )
}

/// Requires quench work to be owned by the shift covering that task interval.
pub(super) fn quench_shift_ownership() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    task_shift_ownership(
        "quenchShiftOwnership",
        |assignment| assignment.quench_interval(),
        |assignment| assignment.quench_shift_identity(),
    )
}

/// Requires unload work to be owned by the shift covering that task interval.
pub(super) fn unload_shift_ownership() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    task_shift_ownership(
        "unloadShiftOwnership",
        |assignment| assignment.unload_interval(),
        |assignment| assignment.unload_shift_identity(),
    )
}

/// Builds a hard constraint for tasks whose interval exists but lacks a covering shift.
fn task_shift_ownership<I, S>(
    name: &'static str,
    interval: I,
    shift: S,
) -> impl IncrementalConstraint<Plan, HardSoftScore>
where
    I: Fn(&FurnaceAssignment) -> Option<(usize, usize)> + Send + Sync + 'static,
    S: Fn(&FurnaceAssignment) -> Option<crate::domain::ShiftIdentity> + Send + Sync + 'static,
{
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .filter(move |assignment: &FurnaceAssignment| {
            interval(assignment).is_some() && shift(assignment).is_none()
        })
        .penalize(hard_weight(|_: &FurnaceAssignment| {
            HardSoftScore::of_hard(900)
        }))
        .named(name)
}

#[cfg(test)]
mod tests {
    use super::changeover_shortage;
    use crate::constraints::create_constraints;
    use crate::data::build_demo_problem;
    use crate::domain::calculate_changeover_cost_for_sequence;
    use crate::domain::Plan;
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

    fn first_compatible_assignment_plan() -> Plan {
        let mut plan = build_demo_problem();
        for assignment in &mut plan.assignments {
            assignment.assignment = assignment.compatible_assignments.first().copied();
        }
        plan
    }

    fn furnace_and_start(value: usize) -> (usize, usize) {
        (
            value / crate::domain::NUM_TIME_SLOTS,
            (value % crate::domain::NUM_TIME_SLOTS) * crate::domain::TIME_STEP,
        )
    }

    fn adjacent_pair(plan: &Plan) -> (usize, usize, usize, usize) {
        let by_furnace_start = assignment_values_by_furnace_start(plan);
        for left_index in 0..plan.assignments.len() {
            for left_value in &plan.assignments[left_index].compatible_assignments {
                let (furnace_id, left_start) = furnace_and_start(*left_value);
                let right_start = left_start + plan.assignments[left_index].duration_minutes();
                if let Some(candidates) = by_furnace_start.get(&(furnace_id, right_start)) {
                    if let Some((right_index, right_value)) = candidates
                        .iter()
                        .copied()
                        .find(|(right_index, _)| *right_index != left_index)
                    {
                        return (left_index, *left_value, right_index, right_value);
                    }
                }
            }
        }
        panic!("adjacent compatible assignment pair");
    }

    fn same_start_pair(plan: &Plan, same_furnace: bool) -> (usize, usize, usize, usize) {
        let by_furnace_start = assignment_values_by_furnace_start(plan);
        for entries in by_furnace_start.values() {
            if same_furnace && entries.len() >= 2 {
                return (entries[0].0, entries[0].1, entries[1].0, entries[1].1);
            }
        }
        if !same_furnace {
            let mut by_start: HashMap<usize, Vec<(usize, usize, usize)>> = HashMap::new();
            for ((furnace_id, start), entries) in by_furnace_start {
                for (assignment_index, value) in entries {
                    by_start
                        .entry(start)
                        .or_default()
                        .push((furnace_id, assignment_index, value));
                }
            }
            for entries in by_start.values() {
                for left in entries {
                    for right in entries {
                        if left.0 != right.0 && left.1 != right.1 {
                            return (left.1, left.2, right.1, right.2);
                        }
                    }
                }
            }
        }
        panic!("same-start compatible assignment pair");
    }

    fn assignment_values_by_furnace_start(
        plan: &Plan,
    ) -> HashMap<(usize, usize), Vec<(usize, usize)>> {
        let mut values = HashMap::new();
        for (assignment_index, assignment) in plan.assignments.iter().enumerate() {
            for value in &assignment.compatible_assignments {
                values
                    .entry(furnace_and_start(*value))
                    .or_insert_with(Vec::new)
                    .push((assignment_index, *value));
            }
        }
        values
    }

    fn reversed_work_order_changeover_shortage_pair(
        plan: &Plan,
    ) -> (usize, usize, usize, usize, usize) {
        for left_index in 0..plan.assignments.len() {
            for right_index in 0..plan.assignments.len() {
                if left_index <= right_index {
                    continue;
                }
                for left_value in &plan.assignments[left_index].compatible_assignments {
                    let (left_furnace, left_start) = furnace_and_start(*left_value);
                    let left_end = left_start + plan.assignments[left_index].duration_minutes();
                    for right_value in &plan.assignments[right_index].compatible_assignments {
                        let (right_furnace, right_start) = furnace_and_start(*right_value);
                        if left_furnace != right_furnace || left_end > right_start {
                            continue;
                        }

                        let mut candidate = plan.clone();
                        candidate.assignments[left_index].assignment = Some(*left_value);
                        candidate.assignments[right_index].assignment = Some(*right_value);
                        let shortage = changeover_shortage(
                            candidate.assignments[left_index].end_minutes().unwrap(),
                            candidate.assignments[right_index].start_minutes().unwrap(),
                            calculate_changeover_cost_for_sequence(
                                &candidate.assignments[left_index],
                                &candidate.assignments[right_index],
                            ) as usize,
                        );
                        if let Some(shortage) = shortage {
                            return (left_index, *left_value, right_index, *right_value, shortage);
                        }
                    }
                }
            }
        }
        panic!("reversed work-order changeover shortage pair");
    }

    #[test]
    fn generated_assignment_values_do_not_create_incompatible_process_matches() {
        let plan = first_compatible_assignment_plan();

        assert_eq!(hard_score(&plan, "incompatibleProcess"), 0);
    }

    #[test]
    fn generated_assignment_values_do_not_create_excessive_temperature_matches() {
        let plan = first_compatible_assignment_plan();

        assert_eq!(hard_score(&plan, "excessiveTemperature"), 0);
    }

    #[test]
    fn generated_assignment_values_do_not_create_excessive_load_matches() {
        let plan = first_compatible_assignment_plan();

        assert_eq!(hard_score(&plan, "excessiveLoad"), 0);
    }

    #[test]
    fn adjacent_same_furnace_intervals_are_allowed() {
        let mut plan = build_demo_problem();
        let (left_index, left_value, right_index, right_value) = adjacent_pair(&plan);
        plan.assignments[left_index].assignment = Some(left_value);
        plan.assignments[right_index].assignment = Some(right_value);

        assert_eq!(hard_score(&plan, "furnaceOverlap"), 0);
    }

    #[test]
    fn true_same_furnace_overlap_penalizes_overlap_minutes() {
        let mut plan = build_demo_problem();
        let (left_index, left_value, right_index, right_value) = same_start_pair(&plan, true);
        plan.assignments[left_index].assignment = Some(left_value);
        plan.assignments[right_index].assignment = Some(right_value);

        assert!(hard_score(&plan, "furnaceOverlap") < 0);
    }

    #[test]
    fn different_furnaces_never_overlap_match() {
        let mut plan = build_demo_problem();
        let (left_index, left_value, right_index, right_value) = same_start_pair(&plan, false);
        plan.assignments[left_index].assignment = Some(left_value);
        plan.assignments[right_index].assignment = Some(right_value);

        assert_eq!(hard_score(&plan, "furnaceOverlap"), 0);
    }

    #[test]
    fn changeover_gap_uses_temporal_order_not_work_order_order() {
        let mut plan = build_demo_problem();
        let (left_index, left_value, right_index, right_value, shortage) =
            reversed_work_order_changeover_shortage_pair(&plan);
        plan.assignments[left_index].assignment = Some(left_value);
        plan.assignments[right_index].assignment = Some(right_value);

        assert_eq!(hard_score(&plan, "changeoverGap"), -(shortage as i64) * 10);
    }
}
