use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Precomputed travel fact between two locations.
///
/// Constraints read these facts while scoring a route. Keeping travel as problem
/// data makes scoring deterministic: the solver evaluates candidate visit
/// orders against the matrix already attached to the plan instead of calling the
/// map service during every move.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct TravelLeg {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub from_location_idx: usize,
    pub to_location_idx: usize,
    pub duration_seconds: i64,
    pub distance_meters: i64,
    pub reachable: bool,
}

/// Constructor payload for `TravelLeg`.
///
/// The route matrix has many similar numeric fields, so named initialization is
/// easier to audit than positional arguments.
#[derive(Debug, Clone)]
pub struct TravelLegInit {
    pub id: String,
    pub name: String,
    pub from_location_idx: usize,
    pub to_location_idx: usize,
    pub duration_seconds: i64,
    pub distance_meters: i64,
    pub reachable: bool,
}

impl TravelLeg {
    /// Builds one directed matrix entry from `from_location_idx` to `to_location_idx`.
    pub fn new(init: TravelLegInit) -> Self {
        Self {
            id: init.id,
            name: init.name,
            from_location_idx: init.from_location_idx,
            to_location_idx: init.to_location_idx,
            duration_seconds: init.duration_seconds,
            distance_meters: init.distance_meters,
            reachable: init.reachable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_travel_leg_construction() {
        let fact = TravelLeg::new(TravelLegInit {
            id: "test-id".to_string(),
            name: "test".to_string(),
            from_location_idx: Default::default(),
            to_location_idx: Default::default(),
            duration_seconds: Default::default(),
            distance_meters: Default::default(),
            reachable: false,
        });
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.from_location_idx;
        let _ = &fact.to_location_idx;
        let _ = &fact.duration_seconds;
        let _ = &fact.distance_meters;
        let _ = &fact.reachable;
    }
}
