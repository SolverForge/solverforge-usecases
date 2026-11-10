use super::shared::*;

/// Penalizes assignments scheduled on furnaces that cannot run the required process.
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
            !furnace.supports_process(assignment.process)
        })
        .penalize(hard_weight(|_: &FurnaceAssignment, _: &Furnace| {
            HardSoftScore::of_hard(4_000)
        }))
        .named("incompatible_process")
}
