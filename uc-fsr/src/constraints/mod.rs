//! Constraint assembly for field-service routing.
//!
//! Each child module owns one business rule and uses stock SolverForge
//! `ConstraintFactory` streams. Route-level calculations are maintained as
//! domain shadow values so the scoring layer stays declarative.

// @solverforge:begin constraint-modules
mod assigned_visits;
mod balance_workload;
mod minimize_travel;
mod priority_slack;
mod reachable_legs;
mod required_parts;
mod required_skills;
mod shift_capacity;
mod territory_affinity;
mod time_windows;
// @solverforge:end constraint-modules

mod assemble;
#[cfg(test)]
mod route_metrics_tests;

pub use assemble::create_constraints;
