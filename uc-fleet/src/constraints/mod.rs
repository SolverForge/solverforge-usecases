/* Constraint definitions.

Add constraint modules with `solverforge generate constraint ...`.
The neutral shell starts with an empty constraint set. */

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

// @solverforge:begin constraint-modules
mod dock_capacity;
mod dock_compatibility;
mod dock_outage;
mod inspection_training_precedence;
mod maximize_ready_days;
mod minimize_churn;
mod minimize_deferral;
mod minimize_inspection_lateness;
mod minimize_overtime;
mod minimize_readiness_shortfall;
mod parts_availability;
mod required_assignments;
mod technician_capacity;
mod weekly_readiness_floor;
mod work_window;
// @solverforge:end constraint-modules

mod assemble {
    use super::*;

    pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
        // @solverforge:begin constraint-calls
        (
            dock_capacity::constraint(),
            dock_compatibility::constraint(),
            dock_outage::constraint(),
            inspection_training_precedence::constraint(),
            maximize_ready_days::constraint(),
            minimize_churn::constraint(),
            minimize_deferral::constraint(),
            minimize_inspection_lateness::constraint(),
            minimize_overtime::constraint(),
            minimize_readiness_shortfall::constraint(),
            parts_availability::constraint(),
            required_assignments::constraint(),
            technician_capacity::constraint(),
            weekly_readiness_floor::constraint(),
            work_window::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}

pub(crate) mod support;
