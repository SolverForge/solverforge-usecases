//! Stable demo-data boundary for the lesson-timetabling app.
//!
//! Other layers import from `crate::data` instead of seed-specific files. That
//! keeps demo-id parsing and timetable generation behind one small interface.

mod data_seed;

pub use data_seed::{available_demo_data, default_demo_data, generate, DemoData};
