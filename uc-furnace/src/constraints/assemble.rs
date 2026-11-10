//! Assembles the furnace planning constraint set.

use crate::domain::Plan;
use solverforge::prelude::*;

/// Builds the complete furnace planning constraint set.
pub fn create_constraints() -> impl ConstraintSet<Plan, HardSoftScore> {
    // @solverforge:begin constraint-calls
    (
        (
            super::unassigned_orders::constraint(),
            super::incompatible_process::constraint(),
            super::excessive_temperature::constraint(),
            super::excessive_load::constraint(),
            super::outside_week::constraint(),
            super::furnace_overlap::constraint(),
            super::changeover_gap::constraint(),
            super::load_build_shift_ownership::constraint(),
            super::program_shift_ownership::constraint(),
            super::quench_shift_ownership::constraint(),
            super::unload_shift_ownership::constraint(),
            super::assigned_tasks_require_scheduled_furnace::constraint(),
        ),
        (
            super::required_task_assigned::constraint(),
            super::task_operator_skilled::constraint(),
            super::task_operator_on_owning_shift::constraint(),
            super::operator_double_booked::constraint(),
            super::shift_role_coverage::constraint(),
            super::monitoring_capacity::constraint(),
            super::day_only_night::constraint(),
            super::minimum_rest::constraint(),
            super::visible_shift_limit::constraint(),
            super::consecutive_nights::constraint(),
            super::express_lateness::constraint(),
            super::urgent_lateness::constraint(),
        ),
        (
            super::standard_lateness::constraint(),
            super::thermal_changeover::constraint(),
            super::early_start_pressure::constraint(),
            super::night_start_preference::constraint(),
            super::overtime::constraint(),
            super::shift_load_balance::constraint(),
        ),
    )
    // @solverforge:end constraint-calls
}
