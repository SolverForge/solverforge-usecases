//! Public demo-data surface for the school timetable example.
//!
//! Keep this file intentionally thin. The rest of the application imports
//! `crate::data::{generate, available_demo_data, DemoData}` as a stable boundary, so
//! the detailed dataset design lives in sibling modules where it can evolve
//! without making the top-level data surface noisy.

mod entrypoints;
mod groups;
mod large;
mod lessons;
mod rooms;
#[cfg(test)]
mod solve_tests;
mod teachers;
mod timeslots;
mod vocabulary;

pub use entrypoints::{available_demo_data, default_demo_data, generate, DemoData};
