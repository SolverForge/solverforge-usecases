use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Customer job that must be inserted into exactly one technician route.
///
/// This is problem data, not a planning entity. The solver does not mutate the
/// visit itself; it mutates `TechnicianRoute.visits`, which stores indexes into
/// the `FieldServicePlan.service_visits` vector.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct ServiceVisit {
    #[planning_id]
    pub id: String,
    #[serde(skip, default)]
    pub index: usize,
    pub name: String,
    pub customer: String,
    pub location_idx: usize,
    pub duration_minutes: i32,
    pub earliest_minute: i32,
    pub latest_minute: i32,
    pub required_skill_mask: i64,
    pub required_parts_mask: i64,
    pub priority: i32,
    pub territory: String,
}

/// Constructor payload for `ServiceVisit`.
///
/// Keeping construction grouped avoids a long positional argument list where a
/// beginner could easily swap time windows, masks, or location indexes.
#[derive(Debug, Clone)]
pub struct ServiceVisitInit {
    pub id: String,
    pub name: String,
    pub customer: String,
    pub location_idx: usize,
    pub duration_minutes: i32,
    pub earliest_minute: i32,
    pub latest_minute: i32,
    pub required_skill_mask: i64,
    pub required_parts_mask: i64,
    pub priority: i32,
    pub territory: String,
}

impl ServiceVisit {
    /// Builds one immutable service-visit fact.
    pub fn new(init: ServiceVisitInit) -> Self {
        Self {
            id: init.id,
            index: 0,
            name: init.name,
            customer: init.customer,
            location_idx: init.location_idx,
            duration_minutes: init.duration_minutes,
            earliest_minute: init.earliest_minute,
            latest_minute: init.latest_minute,
            required_skill_mask: init.required_skill_mask,
            required_parts_mask: init.required_parts_mask,
            priority: init.priority,
            territory: init.territory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_visit_construction() {
        let fact = ServiceVisit::new(ServiceVisitInit {
            id: "test-id".to_string(),
            name: "test".to_string(),
            customer: "test".to_string(),
            location_idx: Default::default(),
            duration_minutes: Default::default(),
            earliest_minute: Default::default(),
            latest_minute: Default::default(),
            required_skill_mask: Default::default(),
            required_parts_mask: Default::default(),
            priority: Default::default(),
            territory: "test".to_string(),
        });
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.customer;
        let _ = &fact.location_idx;
        let _ = &fact.duration_minutes;
        let _ = &fact.earliest_minute;
        let _ = &fact.latest_minute;
        let _ = &fact.required_skill_mask;
        let _ = &fact.required_parts_mask;
        let _ = &fact.priority;
        let _ = &fact.territory;
    }
}
