use serde::{Deserialize, Serialize};

/// School-day enum used by generated data and SolverForge planning facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Weekday {
    #[default]
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
}
