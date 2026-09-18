use crate::domain::{
    owning_shift_identity_for_interval, FurnaceAssignment, MINUTES_PER_DAY, NUM_TIME_SLOTS,
    TIME_STEP,
};

pub(super) fn preferred_start_slots() -> Vec<usize> {
    let mut slots = (0..NUM_TIME_SLOTS).collect::<Vec<_>>();
    slots.sort_by_key(|slot| {
        let minute = slot * crate::domain::TIME_STEP;
        let minute_of_day = minute % MINUTES_PER_DAY;
        let roster_day = minute / MINUTES_PER_DAY;
        let night_penalty = if !(360..1320).contains(&minute_of_day) {
            1
        } else {
            0
        };
        let weekend_penalty = if roster_day >= 5 { 1 } else { 0 };
        (night_penalty, weekend_penalty, *slot)
    });
    slots
}

pub(super) fn slot_preserves_shift_ownership(
    start_minute: usize,
    duration_minutes: usize,
    requires_quench: bool,
) -> bool {
    let end_minute = start_minute + duration_minutes;

    let load_build = owning_shift_identity_for_interval(
        start_minute.saturating_sub(60),
        start_minute.saturating_sub(30),
    );
    let furnace_load = owning_shift_identity_for_interval(
        start_minute.saturating_sub(30),
        start_minute.saturating_sub(15),
    );
    let program = owning_shift_identity_for_interval(start_minute.saturating_sub(15), start_minute);
    let unload = owning_shift_identity_for_interval(end_minute + 15, end_minute + 30);
    let quench = !requires_quench
        || owning_shift_identity_for_interval(end_minute, end_minute + 15).is_some();

    load_build.is_some()
        && furnace_load.is_some()
        && program.is_some()
        && unload.is_some()
        && quench
}

pub(super) fn order_compatible_assignment_ranges(assignments: &mut [FurnaceAssignment]) {
    for assignment in assignments {
        let current_value = assignment.assignment;
        let duration = assignment.duration_minutes();
        let due = assignment.due_datetime_minutes;
        let furnace_count = assignment
            .compatible_assignments
            .iter()
            .map(|candidate| candidate / NUM_TIME_SLOTS)
            .max()
            .map(|max_furnace| max_furnace + 1)
            .unwrap_or(1);

        assignment.compatible_assignments.sort_by_key(|value| {
            let furnace_idx = value / NUM_TIME_SLOTS;
            let start = (value % NUM_TIME_SLOTS) * TIME_STEP;
            let end = start + duration;
            let due_distance = end.abs_diff(due);
            let shift_valid =
                slot_preserves_shift_ownership(start, duration, assignment.requires_quench);
            let rotated_furnace_rank = (furnace_idx + furnace_count
                - (assignment.work_order_idx % furnace_count))
                % furnace_count;
            (
                usize::from(Some(*value) != current_value),
                usize::from(!shift_valid),
                due_distance,
                start,
                rotated_furnace_rank,
            )
        });
        assignment.compatible_assignments.dedup();
    }
}
