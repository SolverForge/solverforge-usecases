/* Constraint definitions.

Add constraint modules with `solverforge generate constraint ...`.
The neutral shell starts with an empty constraint set. */

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

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

mod assemble {
    use super::*;

    pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
        // @solverforge:begin constraint-calls
        (
            assigned_crew::constraint(),
            consecutive_long_haul_recovery::constraint(),
            employee_unavailable::constraint(),
            extended_recovery_rest::constraint(),
            first_assignment_home::constraint(),
            flight_conflict::constraint(),
            last_assignment_home::constraint(),
            long_haul_recovery::constraint(),
            minimum_rest_away::constraint(),
            minimum_rest_home::constraint(),
            required_skill::constraint(),
            transfer_continuity::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
