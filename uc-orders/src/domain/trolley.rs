use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

use super::WarehouseLocation;

/// A four-bucket picking trolley that owns one ordered route of pick-step indexes.
#[planning_entity]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trolley {
    #[planning_id]
    pub id: String,
    pub bucket_count: i64,
    pub bucket_capacity: i64,
    pub location: WarehouseLocation,
    // @solverforge:begin entity-variables
    #[planning_list_variable(element_collection = "pick_steps")]
    pub step_order: Vec<usize>,
    // @solverforge:end entity-variables
    #[cascading_update_shadow_variable]
    #[serde(default)]
    pub required_bucket_count: i64,
    #[cascading_update_shadow_variable]
    #[serde(default)]
    pub distinct_order_count: i64,
    #[cascading_update_shadow_variable]
    #[serde(default)]
    pub route_distance_meters: i64,
}

impl Trolley {
    pub fn new(id: impl Into<String>, bucket_count: i64, bucket_capacity: i64, location: WarehouseLocation) -> Self {
        Self {
            id: id.into(),
            bucket_count,
            bucket_capacity,
            location,
            // @solverforge:begin entity-variable-init
            step_order: Vec::new(),
            // @solverforge:end entity-variable-init
            required_bucket_count: 0,
            distinct_order_count: 0,
            route_distance_meters: 0,
        }
    }

    pub fn excess_buckets(&self) -> i64 {
        (self.required_bucket_count - self.bucket_count).max(0)
    }

    pub fn order_incidence_penalty(&self) -> i64 {
        self.distinct_order_count * 1_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trolley_construction() {
        let entity = Trolley::new(
            "test-id",
            4,
            48_000,
            WarehouseLocation::new("(A,1)", super::super::Side::Left, 0),
        );
        assert_eq!(entity.id, "test-id");
        assert_eq!(entity.bucket_count, 4);
    }
}
