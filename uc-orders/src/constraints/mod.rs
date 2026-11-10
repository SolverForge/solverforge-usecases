//! Constraint assembly for bucket feasibility, order incidence, and route distance.

// @solverforge:begin constraint-modules
mod order_incidence;
mod required_buckets;
mod route_distance;
// @solverforge:end constraint-modules

mod assemble;

pub use assemble::create_constraints;
