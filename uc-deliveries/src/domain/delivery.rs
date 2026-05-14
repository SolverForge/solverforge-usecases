//! Delivery problem facts.
//!
//! A delivery is input data, not something SolverForge mutates directly. The
//! solver places delivery ids into each vehicle's list variable.

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use solverforge_maps::{Coord, RoutingError};

use super::CoordValue;

/// A delivery stop that can be assigned into a vehicle route.
#[problem_fact]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    #[planning_id]
    pub id: usize,
    pub label: String,
    pub kind: DeliveryKind,
    /// Latitude in decimal degrees, wrapped so derived equality stays stable.
    pub lat: CoordValue,
    /// Longitude in decimal degrees, wrapped so derived equality stays stable.
    pub lng: CoordValue,
    /// Load consumed from the assigned vehicle capacity.
    pub demand: i32,
    /// Earliest allowed service start, expressed as seconds after midnight.
    pub min_start_time: i64,
    /// Latest allowed service end, expressed as seconds after midnight.
    pub max_end_time: i64,
    /// Time spent at the stop after arrival.
    pub service_duration: i64,
}

/// Coarse stop type used to shape demo-data demand and UI icons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryKind {
    Residential,
    Business,
    Restaurant,
    #[default]
    Other,
}

impl Delivery {
    /// Creates one problem fact from transport-friendly primitive values.
    pub fn new(
        id: usize,
        label: impl Into<String>,
        kind: DeliveryKind,
        coord: (f64, f64),
        demand: i32,
        time_window: (i64, i64),
        service_duration: i64,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            kind,
            lat: coord.0.into(),
            lng: coord.1.into(),
            demand,
            min_start_time: time_window.0,
            max_end_time: time_window.1,
            service_duration,
        }
    }

    /// Converts the serialized coordinates into the map library's checked type.
    pub fn coord(&self) -> Result<Coord, RoutingError> {
        Ok(Coord::try_new(self.lat.get(), self.lng.get())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_construction() {
        let fact = Delivery::new(
            3,
            "Test stop",
            DeliveryKind::Business,
            (43.77, 11.25),
            4,
            (9 * 3600, 17 * 3600),
            20 * 60,
        );
        assert_eq!(fact.id, 3);
        assert_eq!(fact.label, "Test stop");
        assert_eq!(fact.kind, DeliveryKind::Business);
    }
}
