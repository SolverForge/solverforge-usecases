use crate::domain::Location;

#[derive(Clone, Copy)]
pub(super) struct LocationSeed {
    pub id: &'static str,
    pub label: &'static str,
    pub lat: f64,
    pub lng: f64,
    pub territory: &'static str,
}

impl LocationSeed {
    pub(super) fn to_location(self, kind: &'static str) -> Location {
        Location::new(
            self.id,
            self.label,
            self.label.to_string(),
            coord_e6(self.lat),
            coord_e6(self.lng),
            kind.to_string(),
        )
    }
}

#[derive(Clone, Copy)]
pub(super) struct VisitProfile {
    pub name: &'static str,
    pub duration_minutes: i32,
    pub earliest_minute: i32,
    pub latest_minute: i32,
    pub required_skill_mask: i64,
    pub required_parts_mask: i64,
    pub priority: i32,
}

#[derive(Clone, Copy)]
pub(super) struct TechnicianSeed {
    pub id: &'static str,
    pub name: &'static str,
    pub color: &'static str,
    pub start_location_idx: usize,
    pub end_location_idx: usize,
    pub skill_mask: i64,
    pub inventory_mask: i64,
    pub territory: &'static str,
}

pub(super) const SKILL_HVAC: i64 = 0b0001;
pub(super) const SKILL_ELECTRICAL: i64 = 0b0010;
pub(super) const SKILL_PLUMBING: i64 = 0b0100;
pub(super) const SKILL_ELEVATOR: i64 = 0b1000;

pub(super) const PART_FILTERS: i64 = 0b0001;
pub(super) const PART_RELAYS: i64 = 0b0010;
pub(super) const PART_VALVES: i64 = 0b0100;
pub(super) const PART_SENSORS: i64 = 0b1000;

fn coord_e6(value: f64) -> i32 {
    (value * 1_000_000.0).round() as i32
}
