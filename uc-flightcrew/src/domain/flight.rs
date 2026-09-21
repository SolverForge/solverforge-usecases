use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Immutable flight leg that determines the time and location of each crew seat.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Flight {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub departure_airport_idx: usize,
    pub arrival_airport_idx: usize,
    pub departure_minute: i64,
    pub arrival_minute: i64,
    #[serde(skip)]
    pub index: usize,
}

impl Flight {
    pub fn new(id: impl Into<String>, name: impl Into<String>, departure_airport_idx: usize, arrival_airport_idx: usize, departure_minute: i64, arrival_minute: i64) -> Self {
        Self { id: id.into(), name: name.into(), departure_airport_idx, arrival_airport_idx, departure_minute, arrival_minute, index: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_construction() {
        let fact = Flight::new("test-id", "test", Default::default(), Default::default(), Default::default(), Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.departure_airport_idx;
        let _ = &fact.arrival_airport_idx;
        let _ = &fact.departure_minute;
        let _ = &fact.arrival_minute;
    }
}
