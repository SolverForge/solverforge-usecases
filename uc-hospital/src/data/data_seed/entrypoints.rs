use std::str::FromStr;

use crate::domain::Plan;

use super::large::generate_large;

/// Public demo-data identifiers exposed through the HTTP API.
///
/// The hospital app currently ships one serious benchmark instance rather than a
/// menu of toy presets, so the surface stays explicit instead of pretending that
/// multiple sizes exist when they do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoData {
    Large,
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

impl DemoData {
    /// Returns the canonical uppercase id used by the HTTP API.
    pub fn as_str(&self) -> &'static str {
        match self {
            DemoData::Large => "LARGE",
        }
    }
}

/// Lists the demo identifiers accepted by `/demo-data/{id}`.
pub fn list_demo_data() -> Vec<&'static str> {
    vec![DemoData::Large.as_str()]
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
