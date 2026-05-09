use serde::{Deserialize, Serialize};

/// Obligé de créer son propre enum car chrono n'a pas de Default et c'est utilisé dans la data generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Weekday {
    #[default]
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
}
