mod bergamo_catalog;
mod bergamo_locations;
mod bergamo_profiles;
mod bergamo_technicians;
mod data_seed;

pub use data_seed::{
    available_demo_data, default_demo_data, generate, load_network, prepare_routing, DemoData,
    DemoDataError,
};
