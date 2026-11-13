//! Score rules: four hard rules a valid day plan must satisfy and one soft
//! rule that measures how good it is. Each module is one named constraint.

// @solverforge:begin constraint-modules
mod battery_shortfall;
mod customer_time_windows;
mod total_travel_time;
mod unreachable_legs;
mod vehicle_capacity;
// @solverforge:end constraint-modules

mod assemble;

pub use assemble::create_constraints;

#[cfg(test)]
mod tests;
