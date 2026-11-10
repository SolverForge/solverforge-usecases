use super::shared::*;

/// Penalizes back-to-back furnace jobs that leave less than the required changeover gap.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("changeover_gap")
}
