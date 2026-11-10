//! Assembles the constraint set the solver scores every plan against.

use crate::domain::Plan;
use solverforge::prelude::*;

pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::order_incidence::constraint(),
        super::required_buckets::constraint(),
        super::route_distance::constraint(),
    )
    // @solverforge:end constraint-calls
}
