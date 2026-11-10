use super::shared::*;

/// Penalizes assignments whose required temperature exceeds the selected furnace limit.
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
            !furnace.supports_temperature(assignment.temperature_celsius)
        })
        .penalize(hard_weight(
            |assignment: &FurnaceAssignment, furnace: &Furnace| {
                HardSoftScore::of_hard(
                    (assignment.temperature_celsius - furnace.max_temp_celsius) as i64 * 20,
                )
            },
        ))
        .named("excessive_temperature")
}
