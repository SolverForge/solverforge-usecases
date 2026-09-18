use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// A dated technician capacity change applied only inside its event window.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct TechnicianCapacityOverride {
    #[planning_id]
    pub id: String,
    pub pool_id: String,
    pub start_day: i32,
    pub end_day: i32,
    pub delta: i32,
}

impl TechnicianCapacityOverride {
    pub fn new(
        id: impl Into<String>,
        pool_id: impl Into<String>,
        start_day: i32,
        end_day: i32,
        delta: i32,
    ) -> Self {
        Self {
            id: id.into(),
            pool_id: pool_id.into(),
            start_day,
            end_day,
            delta,
        }
    }
}
