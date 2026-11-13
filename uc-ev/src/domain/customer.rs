//! Customer problem facts.
//!
//! A customer is input data the solver never changes. The solver only decides
//! which vehicle visits it and in which position, by placing the customer's
//! index into one `Vehicle.visit_order` list.

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use solverforge_maps::{Coord, RoutingError};

use super::CoordValue;

/// A pharmacy that receives one delivery during the day.
#[problem_fact]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub lat: CoordValue,
    pub lng: CoordValue,
    /// Crates loaded onto the vehicle for this customer.
    pub demand: i32,
    /// Earliest service start, in seconds after midnight.
    pub min_start_time: i64,
    /// Latest service end, in seconds after midnight.
    pub max_end_time: i64,
    /// Seconds spent unloading at the customer.
    pub service_duration: i64,
}

impl Customer {
    /// Builds a customer from `(lat, lng)` and a `(earliest, latest)` window.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        coord: (f64, f64),
        demand: i32,
        window: (i64, i64),
        service_duration: i64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            lat: coord.0.into(),
            lng: coord.1.into(),
            demand,
            min_start_time: window.0,
            max_end_time: window.1,
            service_duration,
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
    fn test_customer_construction() {
        let fact = Customer::new("c1", "Farmacia", (45.7, 9.67), 3, (28_800, 43_200), 300);
        assert_eq!(fact.id, "c1");
        assert_eq!(fact.lat.get(), 45.7);
        assert_eq!(fact.max_end_time, 43_200);
        assert!(fact.coord().is_ok());
    }
}
