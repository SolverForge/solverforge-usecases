//! Furnace planning constraints, one module per constraint.

// @solverforge:begin constraint-modules
mod assigned_tasks_require_scheduled_furnace;
mod changeover_gap;
mod consecutive_nights;
mod day_only_night;
mod early_start_pressure;
mod excessive_load;
mod excessive_temperature;
mod express_lateness;
mod furnace_overlap;
mod incompatible_process;
mod load_build_shift_ownership;
mod minimum_rest;
mod monitoring_capacity;
mod night_start_preference;
mod operator_double_booked;
mod outside_week;
mod overtime;
mod program_shift_ownership;
mod quench_shift_ownership;
mod required_task_assigned;
mod shift_load_balance;
mod shift_role_coverage;
mod standard_lateness;
mod task_operator_on_owning_shift;
mod task_operator_skilled;
mod thermal_changeover;
mod unassigned_orders;
mod unload_shift_ownership;
mod urgent_lateness;
mod visible_shift_limit;
// @solverforge:end constraint-modules

mod assemble;
mod shared;
#[cfg(test)]
mod tests;

pub use assemble::create_constraints;
