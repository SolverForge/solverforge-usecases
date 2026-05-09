use serde::{Deserialize, Serialize};

/// Coarse service-line grouping used to make nearby search meaningful.
///
/// The solver does not understand "hospital geography" by itself. We therefore
/// encode a lightweight domain signal that says which locations and employee
/// skill bundles are close to one another.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CareHub {
    Ambulatory,
    Neurology,
    CriticalCare,
    PediatricCare,
    Surgery,
    Radiology,
    Outpatient,
    #[default]
    Unknown,
}

impl CareHub {
    /// Maps a published shift location label to the hub used by nearby search.
    pub fn from_location(location: &str) -> Self {
        match location {
            "Ambulatory care" => Self::Ambulatory,
            "Neurology" => Self::Neurology,
            "Critical care" => Self::CriticalCare,
            "Pediatric care" => Self::PediatricCare,
            "Surgery" => Self::Surgery,
            "Radiology" => Self::Radiology,
            "Outpatient" => Self::Outpatient,
            _ => Self::Unknown,
        }
    }

    /// Maps a required skill to the hub that most naturally owns that work.
    pub fn from_skill(skill: &str) -> Option<Self> {
        match skill {
            "Ambulatory doctor" | "Ambulatory nurse" => Some(Self::Ambulatory),
            "Neurology doctor" | "Neurology nurse" | "Cardiology" => Some(Self::Neurology),
            "Critical care doctor" | "Critical care nurse" => Some(Self::CriticalCare),
            "Pediatric doctor" | "Pediatric nurse" => Some(Self::PediatricCare),
            "Surgery doctor" | "Surgery nurse" | "Anaesthetics" => Some(Self::Surgery),
            "Radiology day" | "Radiology nurse" | "Radiology call" => Some(Self::Radiology),
            "Outpatient doctor" | "Outpatient nurse" => Some(Self::Outpatient),
            _ => None,
        }
    }

    /// Guesses an employee's home hub from the service-line skills they carry.
    ///
    /// This is only a fallback for generated or decoded employees that did not
    /// set `home_hub` explicitly.
    pub fn infer_from_skills<'a>(skills: impl IntoIterator<Item = &'a str>) -> Self {
        let mut counts = [0usize; 7];
        for skill in skills {
            match Self::from_skill(skill) {
                Some(Self::Ambulatory) => counts[0] += 1,
                Some(Self::Neurology) => counts[1] += 1,
                Some(Self::CriticalCare) => counts[2] += 1,
                Some(Self::PediatricCare) => counts[3] += 1,
                Some(Self::Surgery) => counts[4] += 1,
                Some(Self::Radiology) => counts[5] += 1,
                Some(Self::Outpatient) => counts[6] += 1,
                Some(Self::Unknown) | None => {}
            }
        }

        let Some((best_index, best_count)) = counts
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|&(index, count)| (count, index))
        else {
            return Self::Unknown;
        };

        if best_count == 0 {
            Self::Unknown
        } else {
            match best_index {
                0 => Self::Ambulatory,
                1 => Self::Neurology,
                2 => Self::CriticalCare,
                3 => Self::PediatricCare,
                4 => Self::Surgery,
                5 => Self::Radiology,
                6 => Self::Outpatient,
                _ => Self::Unknown,
            }
        }
    }
}
