//! Constraint assembly for employee scheduling.
//!
//! Each sibling module contributes one named rule. `create_constraints()`
//! simply lists them in the order we want them to appear in analysis output.

// @solverforge:begin constraint-modules
mod assigned_shift;
mod balance_assignments;
mod desired_day;
mod minimum_rest;
mod one_shift_per_day;
mod overlapping_shift;
mod required_skill;
mod unavailable_employee;
mod undesired_day;
// @solverforge:end constraint-modules

mod assemble;

pub use assemble::create_constraints;
