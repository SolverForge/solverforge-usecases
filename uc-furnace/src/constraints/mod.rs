mod furnace_hard;
mod furnace_soft;
mod monitoring_hard;
mod roster_hard;
mod roster_soft;
mod shared;
mod staffing_hard;

use solverforge::prelude::*;

/// Builds the complete furnace planning constraint set.
pub fn create_constraints() -> impl ConstraintSet<crate::domain::Plan, HardSoftScore> {
    (
        furnace_hard::unassigned_orders(),
        furnace_hard::incompatible_process(),
        furnace_hard::excessive_temperature(),
        furnace_hard::excessive_load(),
        furnace_hard::outside_week(),
        furnace_hard::furnace_overlap(),
        furnace_hard::changeover_gap(),
        furnace_hard::load_build_shift_ownership(),
        furnace_hard::program_shift_ownership(),
        furnace_hard::quench_shift_ownership(),
        furnace_hard::unload_shift_ownership(),
        staffing_hard::assigned_tasks_require_scheduled_furnace(),
        staffing_hard::required_task_assigned(),
        staffing_hard::task_operator_skilled(),
        staffing_hard::task_operator_on_owning_shift(),
        staffing_hard::operator_double_booked(),
        staffing_hard::shift_role_coverage(),
        monitoring_hard::monitoring_capacity(),
        roster_hard::day_only_night(),
        roster_hard::minimum_rest(),
        roster_hard::visible_shift_limit(),
        roster_hard::consecutive_nights(),
        furnace_soft::express_lateness(),
        furnace_soft::urgent_lateness(),
        furnace_soft::standard_lateness(),
        furnace_soft::thermal_changeover(),
        furnace_soft::early_start_pressure(),
        furnace_soft::night_start_preference(),
        roster_soft::overtime(),
        roster_soft::shift_load_balance(),
    )
}
