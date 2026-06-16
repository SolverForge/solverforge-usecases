#![cfg_attr(rustfmt, rustfmt_skip)]
//! Constraint assembly for employee scheduling.
//!
//! Each sibling module contributes one named rule. `create_constraints()`
//! simply lists them in the order we want them to appear in analysis output.

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

// @solverforge:begin constraint-modules
mod assigned_shift;
mod required_skill;
mod overlapping_shift;
mod minimum_rest;
mod one_shift_per_day;
mod unavailable_employee;
mod undesired_day;
mod desired_day;
mod balance_assignments;
// @solverforge:end constraint-modules

mod assemble {
    use super::*;

    /// Collects the full scoring model used by `Plan`.
    pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftDecimalScore> {
        // @solverforge:begin constraint-calls
        (
            assigned_shift::constraint(),
            required_skill::constraint(),
            overlapping_shift::constraint(),
            minimum_rest::constraint(),
            one_shift_per_day::constraint(),
            unavailable_employee::constraint(),
            undesired_day::constraint(),
            desired_day::constraint(),
            balance_assignments::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
