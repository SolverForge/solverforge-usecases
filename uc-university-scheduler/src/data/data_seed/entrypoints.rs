use std::str::FromStr;

use crate::domain::Plan;

use super::large::generate_large;

/// Public demo-data identifiers exposed through the HTTP API.
///
/// The university app currently ships one serious benchmark instance rather than a
/// menu of toy presets, so the surface stays explicit instead of pretending that
/// multiple sizes exist when they do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoData {
    Large,
}

impl DemoData {
    /// Returns the canonical uppercase id used by the HTTP API.
    pub fn id(self) -> &'static str {
        match self {
            DemoData::Large => "LARGE",
        }
    }

    /// Returns the default demo data.
    pub fn default_demo_data() -> Self {
        DemoData::Large
    }

    /// Returns all available demo data identifiers.
    pub fn available_demo_data() -> &'static [Self] {
        &[DemoData::Large]
    }
}

/// Returns the default demo data.
pub fn default_demo_data() -> DemoData {
    DemoData::Large
}

/// Returns the list of available demo data identifiers.
pub fn available_demo_data() -> &'static [DemoData] {
    DemoData::available_demo_data()
}

impl FromStr for DemoData {
    type Err = ();

    /// Parses the case-insensitive demo id exposed over HTTP.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LARGE" => Ok(DemoData::Large),
            _ => Err(()),
        }
    }
}

/// Generates the requested demo dataset.
///
/// Dispatch stays here so callers see the supported public variants in one
/// place, while the dataset assembly itself remains hidden in the per-instance
/// modules.
pub fn generate(demo: DemoData) -> Plan {
    match demo {
        DemoData::Large => generate_large(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::create_constraints;
    use solverforge::{ConstraintSet, HardMediumSoftScore};

    #[test]
    fn test_generate_large() {
        let plan = generate(DemoData::Large);
        assert_eq!(plan.timeslots.len(), 40);
        assert_eq!(plan.teachers.len(), 20);
        assert_eq!(plan.groups.len(), 12);
        assert_eq!(plan.lessons.len(), 300);
        assert_eq!(plan.rooms.len(), 10);
        assert!(plan
            .lessons
            .iter()
            .all(|lesson| lesson.timeslot_idx.is_none()));
        assert!(plan.lessons.iter().all(|lesson| lesson.room_idx.is_none()));
    }

    #[test]
    fn test_generate_large_initial_score() {
        let plan = generate(DemoData::Large);
        let score = create_constraints().evaluate_all(&plan);

        assert_eq!(score, HardMediumSoftScore::of_medium(-600));
    }

    #[test]
    fn test_demo_data_from_str() {
        assert_eq!("LARGE".parse::<DemoData>().ok(), Some(DemoData::Large));
        assert_eq!("large".parse::<DemoData>().ok(), Some(DemoData::Large));
        assert_eq!("INVALID".parse::<DemoData>().ok(), None);
    }

    #[test]
    fn test_demo_data_id() {
        assert_eq!(DemoData::Large.id(), "LARGE");
    }

    #[test]
    fn test_default_demo_data() {
        assert_eq!(default_demo_data(), DemoData::Large);
        assert_eq!(DemoData::default_demo_data(), DemoData::Large);
    }

    #[test]
    fn test_available_demo_data() {
        assert_eq!(available_demo_data(), &[DemoData::Large]);
        assert_eq!(DemoData::available_demo_data(), &[DemoData::Large]);
    }
}
