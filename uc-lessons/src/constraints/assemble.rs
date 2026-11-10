//! Assembles the constraint set the solver scores every plan against.

use crate::domain::Plan;
use solverforge::prelude::*;

/// Collects the full scoring model used by `Plan`.
pub fn create_constraints() -> impl ConstraintSet<Plan, HardMediumSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::assign_timeslot::constraint(),
        super::assign_room::constraint(),
        super::teacher_availability::constraint(),
        super::group_availability::constraint(),
        super::room_kind::constraint(),
        super::room_capacity::constraint(),
        super::no_group_conflict::constraint(),
        super::no_teacher_conflict::constraint(),
        super::no_room_conflict::constraint(),
        super::late_lesson::constraint(),
        super::repeated_subject_day::constraint(),
    )
    // @solverforge:end constraint-calls
}
