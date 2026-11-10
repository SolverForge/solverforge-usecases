use super::shared::*;

/// Penalizes assignments whose load weight exceeds the selected furnace capacity.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("excessive_load")
}
