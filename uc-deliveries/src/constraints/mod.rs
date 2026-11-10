//! Constraint assembly for delivery routing.
//!
//! Each sibling file contributes one named rule. `create_constraints()` lists
//! them in the order we want beginners to see in score analysis output.

// @solverforge:begin constraint-modules
mod all_deliveries_assigned;
mod delivery_time_windows;
mod total_travel_time;
mod vehicle_capacity;
// @solverforge:end constraint-modules

mod assemble;

pub use assemble::create_constraints;
