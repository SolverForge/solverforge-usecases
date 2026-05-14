#![cfg_attr(rustfmt, rustfmt_skip)]
//! Constraint assembly for lesson timetabling.
//!
//! Each child module owns one named timetable rule. `create_constraints()`
//! lists them in the order beginners should read the score analysis: assignment
//! completeness first, hard feasibility next, soft timetable quality last.

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

    /// Collects the full scoring model used by `Plan`.
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
