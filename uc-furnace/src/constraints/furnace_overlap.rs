use super::shared::*;

/// Penalizes overlapping jobs assigned to the same furnace.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("furnace_overlap")
}
