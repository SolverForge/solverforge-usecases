//! Charging-station problem facts.
//!
//! Chargers are not route elements: no planning variable points at them. The
//! route evaluator in `route_walk.rs` decides on which legs a vehicle detours
//! to a charger, so the solver only ever moves customers.

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use solverforge_maps::{Coord, RoutingError};

use super::CoordValue;

/// A public charging station any vehicle may use.
#[problem_fact]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Charger {
    #[planning_id]
    pub id: String,
    /// Operator or station name, shown on the map.
    pub name: String,
    pub lat: CoordValue,
    pub lng: CoordValue,
}

impl Charger {
    pub fn new(id: impl Into<String>, name: impl Into<String>, lat: f64, lng: f64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            lat: lat.into(),
            lng: lng.into(),
        }
    }

    /// The checked map coordinate used for road routing.
    pub fn coord(&self) -> Result<Coord, RoutingError> {
        Ok(Coord::try_new(self.lat.get(), self.lng.get())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charger_construction() {
        let fact = Charger::new("k1", "Enel X", 45.66, 9.78);
        assert_eq!(fact.id, "k1");
        assert_eq!(fact.name, "Enel X");
        assert!(fact.coord().is_ok());
    }
}
