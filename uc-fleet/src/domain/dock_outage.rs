use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// A dated dock closure that repair solves must treat as hard state.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct DockOutage {
    #[planning_id]
    pub id: String,
    pub dock_id: String,
    pub start_day: i32,
    pub end_day: i32,
}

impl DockOutage {
    pub fn new(id: impl Into<String>, dock_id: impl Into<String>, start_day: i32, end_day: i32) -> Self {
        Self {
            id: id.into(),
            dock_id: dock_id.into(),
            start_day,
            end_day,
        }
    }
}
