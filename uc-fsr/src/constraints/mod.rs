//! Constraint assembly for field-service routing.
//!
//! Each child module owns one business rule and uses stock SolverForge
//! `ConstraintFactory` streams. Route-level calculations are maintained as
//! domain shadow values so the scoring layer stays declarative.

use crate::domain::FieldServicePlan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

#[cfg(test)]
mod route_metrics_tests;

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

mod assemble {
    use super::*;

    /// Collects the full scoring model used by `FieldServicePlan`.
    pub fn create_constraints() -> impl ConstraintSet<FieldServicePlan, HardSoftScore> {
        // @solverforge:begin constraint-calls
        (
            assigned_visits::missing_visits(),
            assigned_visits::duplicate_assignments(),
            assigned_visits::invalid_assignments(),
            balance_workload::constraint(),
            minimize_travel::constraint(),
            priority_slack::constraint(),
            reachable_legs::constraint(),
            required_parts::constraint(),
            required_skills::constraint(),
            shift_capacity::constraint(),
            territory_affinity::constraint(),
            time_windows::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
