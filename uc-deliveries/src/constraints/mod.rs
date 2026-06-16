#![cfg_attr(rustfmt, rustfmt_skip)]
//! Constraint assembly for delivery routing.
//!
//! Each sibling file contributes one named rule. `create_constraints()` lists
//! them in the order we want beginners to see in score analysis output.

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

// @solverforge:begin constraint-modules
mod all_deliveries_assigned;
mod vehicle_capacity;
mod delivery_time_windows;
mod total_travel_time;
// @solverforge:end constraint-modules

mod assemble {
    use super::*;

    /// Collects the full scoring model used by `Plan`.
    pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
        // @solverforge:begin constraint-calls
        (
            all_deliveries_assigned::constraint(),
            vehicle_capacity::constraint(),
            delivery_time_windows::constraint(),
            total_travel_time::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
