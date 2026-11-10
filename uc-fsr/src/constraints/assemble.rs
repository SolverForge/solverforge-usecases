//! Assembles the constraint set the solver scores every plan against.

use crate::domain::FieldServicePlan;
use solverforge::prelude::*;

/// Collects the full scoring model used by `FieldServicePlan`.
pub fn create_constraints() -> impl ConstraintSet<FieldServicePlan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        super::assigned_visits::constraint(),
        super::balance_workload::constraint(),
        super::minimize_travel::constraint(),
        super::priority_slack::constraint(),
        super::reachable_legs::constraint(),
        super::required_parts::constraint(),
        super::required_skills::constraint(),
        super::shift_capacity::constraint(),
        super::territory_affinity::constraint(),
        super::time_windows::constraint(),
    )
    // @solverforge:end constraint-calls
}
