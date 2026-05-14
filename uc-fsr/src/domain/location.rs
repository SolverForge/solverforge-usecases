use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Depot or customer site used by the routing model.
///
/// SolverForge treats a `Location` as read-only problem data. Routes refer to
/// locations by vector index so constraints and map rendering can cheaply look
/// up coordinates without copying place records into every visit.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Location {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub label: String,
    pub lat_e6: i32,
    pub lng_e6: i32,
    pub kind: String,
}

impl Location {
    /// Builds one location fact from seed data or transport input.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        label: String,
        lat_e6: i32,
        lng_e6: i32,
        kind: String,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            label,
            lat_e6,
            lng_e6,
            kind,
        }
    }

    /// Returns latitude in degrees from the integer microdegree storage format.
    pub fn lat(&self) -> f64 {
        f64::from(self.lat_e6) / 1_000_000.0
    }

    /// Returns longitude in degrees from the integer microdegree storage format.
    pub fn lng(&self) -> f64 {
        f64::from(self.lng_e6) / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_construction() {
        let fact = Location::new(
            "test-id",
            "test",
            "test".to_string(),
            0,
            0,
            "test".to_string(),
        );
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.label;
        let _ = &fact.lat_e6;
        let _ = &fact.lng_e6;
        let _ = &fact.kind;
    }
}
