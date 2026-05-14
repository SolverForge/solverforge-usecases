use crate::domain::DeliveryKind;

pub(super) const VEHICLE_NAMES: [&str; 10] = [
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel", "India", "Juliet",
];

#[derive(Clone, Copy)]
pub(super) struct LocationData {
    pub name: &'static str,
    pub lat: f64,
    pub lng: f64,
    pub customer_type: CustomerType,
}

#[derive(Clone, Copy)]
pub(super) enum CustomerType {
    Residential,
    Business,
    Restaurant,
}

impl CustomerType {
    pub(super) fn profile(self) -> (DeliveryKind, i64, i64, (i32, i32), (i64, i64)) {
        match self {
            CustomerType::Residential => (
                DeliveryKind::Residential,
                17 * 3600,
                20 * 3600,
                (1, 2),
                (5 * 60, 10 * 60),
            ),
            CustomerType::Business => (
                DeliveryKind::Business,
                9 * 3600,
                17 * 3600,
                (3, 6),
                (15 * 60, 30 * 60),
            ),
            CustomerType::Restaurant => (
                DeliveryKind::Restaurant,
                6 * 3600,
                10 * 3600,
                (5, 10),
                (20 * 60, 40 * 60),
            ),
        }
    }
}
