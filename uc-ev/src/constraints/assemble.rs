//! Assembles the project's constraint set.

use crate::domain::Plan;
use solverforge::prelude::*;

/// Builds the constraint set the solver scores every solution against.
pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::battery_shortfall::constraint(),
        super::vehicle_capacity::constraint(),
        super::customer_time_windows::constraint(),
        super::unreachable_legs::constraint(),
        super::total_travel_time::constraint(),
    )
    // @solverforge:end constraint-calls
}
