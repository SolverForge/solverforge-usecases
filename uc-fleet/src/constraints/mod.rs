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

mod assemble;
pub(crate) mod support;

pub use assemble::create_constraints;
