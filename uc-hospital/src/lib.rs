//! SolverForge Hospital example
//!
//! This crate is the "teachable core" of the example app.
//!
//! Beginners usually need three things when they open a SolverForge example:
//! 1. the planning model (`domain`)
//! 2. the scoring rules (`constraints`)
//! 3. the transport/runtime layer that turns a model into a running web app
//!
//! The modules below are arranged in that same order so the source reads like a
//! guided tour from optimization concepts to HTTP/UI integration.

pub mod api;
pub mod constraints;
pub mod data;
pub mod domain;
pub mod solver;
