//! Stable demo-data boundary for the deliveries app.
//!
//! Other layers should import `generate` and `DemoData` from here instead of
//! reaching into city-specific seed modules. That keeps the public data surface
//! small while the Philadelphia, Hartford, and Firenze fixtures can stay split
//! for readability.

mod data_seed;

pub use data_seed::{generate, DemoData};
