#![cfg_attr(rustfmt, rustfmt_skip)]
/* Constraint definitions.

Add constraint modules with `solverforge generate constraint ...`.
The neutral shell starts with an empty constraint set. */

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

// @solverforge:begin constraint-modules
mod assign_room;
mod assign_timeslot;
mod group_availability;
mod late_lesson;
mod no_group_conflict;
mod no_room_conflict;
mod no_teacher_conflict;
mod repeated_subject_day;
mod room_capacity;
mod room_kind;
mod teacher_availability;
// @solverforge:end constraint-modules

mod assemble {
    use super::*;

    pub fn create_constraints() -> impl ConstraintSet<Plan, HardMediumSoftScore> {
        // @solverforge:begin constraint-calls
        (
            assign_timeslot::constraint(),
            assign_room::constraint(),
            teacher_availability::constraint(),
            group_availability::constraint(),
            room_kind::constraint(),
            room_capacity::constraint(),
            no_group_conflict::constraint(),
            no_teacher_conflict::constraint(),
            no_room_conflict::constraint(),
            late_lesson::constraint(),
            repeated_subject_day::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
