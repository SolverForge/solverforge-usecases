//! Stable public entrypoint for demo data.
//!
//! Other modules should import from `crate::data` instead of reaching directly
//! into `data_seed/`. That keeps the app's public data surface small even though
//! the generator itself is split across many focused files.

mod data_seed;

pub use data_seed::{generate, list_demo_data, DemoData};
