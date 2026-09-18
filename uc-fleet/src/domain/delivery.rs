use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed inbound part-delivery fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Delivery {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub part_id: String,
    pub quantity: i32,
    pub arrival_day: i32,
}

impl Delivery {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        part_id: impl Into<String>,
        quantity: i32,
        arrival_day: i32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            part_id: part_id.into(),
            quantity,
            arrival_day,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_construction() {
        let fact = Delivery::new("test-id", "test", "PROP-SEAL-KIT", 1, 19);
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.part_id;
        let _ = &fact.quantity;
        let _ = &fact.arrival_day;
    }
}
