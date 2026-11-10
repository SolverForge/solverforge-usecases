// @solverforge:begin constraint-modules
mod assigned_crew;
mod consecutive_long_haul_recovery;
mod employee_unavailable;
mod extended_recovery_rest;
mod first_assignment_home;
mod flight_conflict;
mod last_assignment_home;
mod long_haul_recovery;
mod minimum_rest_away;
mod minimum_rest_home;
mod required_skill;
mod transfer_continuity;
// @solverforge:end constraint-modules

mod assemble;

pub use assemble::create_constraints;
