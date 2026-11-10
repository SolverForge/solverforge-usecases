use super::shared::*;

/// Prevents an operator from being assigned to overlapping manual tasks.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("operator_double_booked")
}
