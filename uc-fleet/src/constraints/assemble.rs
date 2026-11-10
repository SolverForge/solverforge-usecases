//! Assembles the constraint set the solver scores every plan against.

use crate::domain::Plan;
use solverforge::prelude::*;

pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::dock_capacity::constraint(),
        super::dock_compatibility::constraint(),
        super::dock_outage::constraint(),
        super::inspection_training_precedence::constraint(),
        super::maximize_ready_days::constraint(),
        super::minimize_churn::constraint(),
        super::minimize_deferral::constraint(),
        super::minimize_inspection_lateness::constraint(),
        super::minimize_overtime::constraint(),
        super::minimize_readiness_shortfall::constraint(),
        super::parts_availability::constraint(),
        super::required_assignments::constraint(),
        super::technician_capacity::constraint(),
        super::weekly_readiness_floor::constraint(),
        super::work_window::constraint(),
    )
    // @solverforge:end constraint-calls
}
