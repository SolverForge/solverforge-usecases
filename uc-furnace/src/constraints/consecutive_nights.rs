use super::shared::*;

/// Caps consecutive night-shift windows for each operator.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .project(OperatorNightWindowEntries)
        .group_by(
            |entry: &NightWindowEntry| (entry.operator_id, entry.window_start_day),
            sum(|entry: &NightWindowEntry| entry.delta),
        )
        .penalize(hard_weight(|_window: &(usize, i32), night_shifts: &i64| {
            HardSoftScore::of_hard((*night_shifts - MAX_CONSECUTIVE_NIGHTS as i64).max(0) * 900)
        }))
        .named("consecutive_nights")
}
