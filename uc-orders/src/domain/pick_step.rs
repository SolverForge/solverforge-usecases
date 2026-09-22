use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

use super::WarehouseLocation;

/// One immutable order item and the warehouse stop where it is picked.
#[problem_fact]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickStep {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub product_id: String,
    pub order_id: String,
    pub order_item_id: String,
    pub volume_cm3: i64,
    pub location: WarehouseLocation,
}

impl PickStep {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        product_id: impl Into<String>,
        order_id: impl Into<String>,
        order_item_id: impl Into<String>,
        volume_cm3: i64,
        location: WarehouseLocation,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            product_id: product_id.into(),
            order_id: order_id.into(),
            order_item_id: order_item_id.into(),
            volume_cm3,
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_step_construction() {
        let fact = PickStep::new(
            "step-order-1-item-1",
            "Tea",
            "2",
            "1",
            "order-1-item-1",
            180,
            WarehouseLocation::new("(D,2)", super::super::Side::Right, 7),
        );
        assert_eq!(fact.id, "step-order-1-item-1");
        assert_eq!(fact.order_item_id, "order-1-item-1");
    }
}
