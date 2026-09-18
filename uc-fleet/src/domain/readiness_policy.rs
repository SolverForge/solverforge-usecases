use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed weekly readiness floor policy.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct ReadinessPolicy {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub min_ready_overall_per_week: i32,
    pub min_ready_patrol_cutter: i32,
    pub min_ready_frigate_like: i32,
}

impl ReadinessPolicy {
    pub fn new(id: impl Into<String>, name: impl Into<String>, min_ready_overall_per_week: i32, min_ready_patrol_cutter: i32, min_ready_frigate_like: i32) -> Self {
        Self { id: id.into(), name: name.into(), min_ready_overall_per_week, min_ready_patrol_cutter, min_ready_frigate_like }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readiness_policy_construction() {
        let fact = ReadinessPolicy::new("test-id", "test", Default::default(), Default::default(), Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.min_ready_overall_per_week;
        let _ = &fact.min_ready_patrol_cutter;
        let _ = &fact.min_ready_frigate_like;
    }
}
