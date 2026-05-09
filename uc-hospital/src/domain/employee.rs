use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use std::collections::BTreeSet;

use super::CareHub;

/// Hospital staff member published as a SolverForge problem fact.
///
/// A few fields are "authoritative transport state" (`*_dates`), while others
/// are precomputed runtime helpers (`index`, `*_days`). `finalize()` keeps those
/// two views in sync after generation or JSON decoding.
#[problem_fact]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Employee {
    pub id: String,
    #[serde(skip)]
    pub index: usize,
    pub name: String,
    #[serde(default)]
    pub home_hub: CareHub,
    #[serde(default)]
    pub skills: BTreeSet<String>,
    #[serde(default)]
    pub unavailable_dates: BTreeSet<NaiveDate>,
    #[serde(default)]
    pub undesired_dates: BTreeSet<NaiveDate>,
    #[serde(default)]
    pub desired_dates: BTreeSet<NaiveDate>,
    #[serde(skip)]
    pub unavailable_days: Vec<NaiveDate>,
    #[serde(skip)]
    pub undesired_days: Vec<NaiveDate>,
    #[serde(skip)]
    pub desired_days: Vec<NaiveDate>,
}

impl Employee {
    /// Creates a beginner-friendly builder seed with stable defaults.
    pub fn new(index: usize, name: impl Into<String>) -> Self {
        Self {
            id: format!("employee-{index}"),
            index,
            name: name.into(),
            home_hub: CareHub::Unknown,
            skills: BTreeSet::new(),
            unavailable_dates: BTreeSet::new(),
            undesired_dates: BTreeSet::new(),
            desired_dates: BTreeSet::new(),
            unavailable_days: Vec::new(),
            undesired_days: Vec::new(),
            desired_days: Vec::new(),
        }
    }

    /// Overrides the transport-visible identifier.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the employee's home service line used by nearby search.
    pub fn with_home_hub(mut self, home_hub: CareHub) -> Self {
        self.home_hub = home_hub;
        self
    }

    /// Rebuilds the derived caches the solver reads frequently.
    ///
    /// The serialized `BTreeSet`s are the stable truth for transport. The
    /// `Vec`s are just pre-expanded, iteration-friendly mirrors used by
    /// constraints and heuristics.
    pub fn finalize(&mut self) {
        if self.home_hub == CareHub::Unknown {
            self.home_hub = CareHub::infer_from_skills(self.skills.iter().map(String::as_str));
        }
        self.unavailable_days = self.unavailable_dates.iter().copied().collect();
        self.undesired_days = self.undesired_dates.iter().copied().collect();
        self.desired_days = self.desired_dates.iter().copied().collect();
    }

    /// Adds one service-line skill to the employee.
    pub fn with_skill(mut self, skill: impl Into<String>) -> Self {
        self.skills.insert(skill.into());
        self
    }

    /// Adds several skills in one builder step.
    pub fn with_skills(mut self, skills: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for skill in skills {
            self.skills.insert(skill.into());
        }
        self
    }

    /// Marks a day as completely unavailable.
    pub fn with_unavailable_date(mut self, date: NaiveDate) -> Self {
        self.unavailable_dates.insert(date);
        self
    }

    /// Marks a day the employee would prefer to avoid.
    pub fn with_undesired_date(mut self, date: NaiveDate) -> Self {
        self.undesired_dates.insert(date);
        self
    }

    /// Marks a day the employee would actively like to work.
    pub fn with_desired_date(mut self, date: NaiveDate) -> Self {
        self.desired_dates.insert(date);
        self
    }
}
