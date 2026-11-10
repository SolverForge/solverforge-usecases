//! Assembles the constraint set the solver scores every plan against.

use crate::domain::Plan;
use solverforge::prelude::*;

pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::assigned_crew::constraint(),
        super::consecutive_long_haul_recovery::constraint(),
        super::employee_unavailable::constraint(),
        super::extended_recovery_rest::constraint(),
        super::first_assignment_home::constraint(),
        super::flight_conflict::constraint(),
        super::last_assignment_home::constraint(),
        super::long_haul_recovery::constraint(),
        super::minimum_rest_away::constraint(),
        super::minimum_rest_home::constraint(),
        super::required_skill::constraint(),
        super::transfer_continuity::constraint(),
    )
    // @solverforge:end constraint-calls
}
