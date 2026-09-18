mod base_data;
mod catalog;
mod furnace_seed;
mod operators;
mod orders;
mod problem;
#[cfg(test)]
mod tests;

pub use catalog::{generate, DemoData};

#[cfg(test)]
pub(crate) use problem::{build_demo_plan, build_demo_plan_for, build_demo_problem};
