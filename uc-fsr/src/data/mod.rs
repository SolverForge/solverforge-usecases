//! Stable demo-data boundary for the FSR app.
//!
//! Other layers import from `crate::data` instead of city-specific files. That
//! keeps routing preparation and demo-id parsing behind one small interface.

mod bergamo_catalog;
mod bergamo_locations;
mod bergamo_profiles;
mod bergamo_technicians;
mod data_seed;

pub use data_seed::{
    available_demo_data, default_demo_data, generate, load_network, prepare_routing, DemoData,
    DemoDataError,
};
