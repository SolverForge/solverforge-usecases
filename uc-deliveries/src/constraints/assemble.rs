//! Assembles the constraint set the solver scores every plan against.

use crate::domain::Plan;
use solverforge::prelude::*;

/// Collects the full scoring model used by `Plan`.
pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::all_deliveries_assigned::constraint(),
        super::vehicle_capacity::constraint(),
        super::delivery_time_windows::constraint(),
        super::total_travel_time::constraint(),
    )
    // @solverforge:end constraint-calls
}
