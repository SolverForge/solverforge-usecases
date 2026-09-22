//! Constraint assembly for bucket feasibility, order incidence, and route distance.

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

// @solverforge:begin constraint-modules
mod order_incidence;
mod required_buckets;
mod route_distance;
// @solverforge:end constraint-modules

mod assemble {
    use super::*;

    pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
        // @solverforge:begin constraint-calls
        (
            order_incidence::constraint(),
            required_buckets::constraint(),
            route_distance::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
