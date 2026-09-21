use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Airport used for crew home bases, flight endpoints, and transfer times.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Airport {
    #[planning_id]
    pub id: String,
    pub name: String,
    /// Latitude in ten-thousandths of a degree for exact fact equality.
    pub latitude_e4: i32,
    /// Longitude in ten-thousandths of a degree for exact fact equality.
    pub longitude_e4: i32,
    pub taxi_minutes: Vec<u32>,
    #[serde(skip)]
    pub index: usize,
}

impl Airport {
    pub fn new(id: impl Into<String>, name: impl Into<String>, latitude_e4: i32, longitude_e4: i32, taxi_minutes: Vec<u32>) -> Self {
        Self { id: id.into(), name: name.into(), latitude_e4, longitude_e4, taxi_minutes, index: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airport_construction() {
        let fact = Airport::new("test-id", "test", 0, 0, Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.latitude_e4;
        let _ = &fact.longitude_e4;
        let _ = &fact.taxi_minutes;
    }
}
