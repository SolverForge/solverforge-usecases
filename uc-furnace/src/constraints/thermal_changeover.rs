use super::shared::*;

/// Penalizes adjacent jobs on the same furnace for thermal changeover cost.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .join(equal(|assignment: &FurnaceAssignment| {
            assignment.furnace_idx()
        }))
        .filter(|left: &FurnaceAssignment, right: &FurnaceAssignment| {
            left.work_order_idx != right.work_order_idx
                && right.start_minutes().unwrap() == left.end_minutes().unwrap()
        })
        .penalize(|left: &FurnaceAssignment, right: &FurnaceAssignment| {
            HardSoftScore::of_soft(calculate_changeover_cost_for_sequence(left, right) as i64)
        })
        .named("thermal_changeover")
}
