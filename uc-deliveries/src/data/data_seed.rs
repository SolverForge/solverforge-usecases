//! Deterministic delivery demo-data modules.
//!
//! `entrypoints` owns the public dataset ids, while each city module owns its
//! depots and stops. The solver receives ordinary `Plan` values; there is no
//! hidden runtime data source behind these seeds.

mod entrypoints;
mod firenze;
mod hartford;
mod philadelphia;
mod types;

pub use entrypoints::{generate, DemoData};

#[cfg(test)]
mod tests;
