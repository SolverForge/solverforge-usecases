//! Constraint assembly for lesson timetabling.
//!
//! Each child module owns one named timetable rule. `create_constraints()`
//! lists them in the order beginners should read the score analysis: assignment
//! completeness first, hard feasibility next, soft timetable quality last.

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

mod assemble;

pub use assemble::create_constraints;
