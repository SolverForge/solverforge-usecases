//! Public demo-data surface for the hospital example.
//!
//! Keep this file intentionally thin. The rest of the application imports
//! `crate::data::{generate, list_demo_data, DemoData}` as a stable boundary, so
//! the detailed dataset design lives in sibling modules where it can evolve
//! without making the top-level data surface noisy.

mod availability;
mod cohorts;
mod coverage;
mod demand;
mod employees;
mod entrypoints;
mod large;
mod preferences;
mod shifts;
mod skills;
mod time_utils;
mod validation;
mod vocabulary;
mod witness;

#[cfg(test)]
mod solve_tests;
#[cfg(test)]
mod tests;

pub use entrypoints::{generate, list_demo_data, DemoData};
