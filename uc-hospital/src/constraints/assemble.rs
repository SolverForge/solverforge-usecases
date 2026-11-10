//! Assembles the constraint set the solver scores every plan against.

use crate::domain::Plan;
use solverforge::prelude::*;

/// Collects the full scoring model used by `Plan`.
pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftDecimalScore> {
    // @solverforge:begin constraint-calls
    (
        super::assigned_shift::constraint(),
        super::required_skill::constraint(),
        super::overlapping_shift::constraint(),
        super::minimum_rest::constraint(),
        super::one_shift_per_day::constraint(),
        super::unavailable_employee::constraint(),
        super::undesired_day::constraint(),
        super::desired_day::constraint(),
        super::balance_assignments::constraint(),
    )
    // @solverforge:end constraint-calls
}
