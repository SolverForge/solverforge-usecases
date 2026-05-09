use super::Weekday;
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// TODO — describe this fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Timeslot {
    #[planning_id]
    pub id: String,
    #[serde(skip)]
    pub index: usize, // the solver-facing join key
    pub day_of_week: Weekday,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

// Implémentation manuelle pour satisfaire le macro #[problem_fact]
impl Default for Timeslot {
    fn default() -> Self {
        Self {
            id: String::new(),
            index: Default::default(),
            day_of_week: Default::default(),
            start_time: Default::default(),
            end_time: Default::default(),
        }
    }
}

impl Timeslot {
    pub fn new(
        index: usize,
        day_of_week: Weekday,
        start_time: NaiveTime,
        end_time: NaiveTime,
    ) -> Self {
        Self {
            id: format!("timeslot-{index}"),
            index,
            day_of_week,
            start_time,
            end_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeslot_construction() {
        let fact = Timeslot::new(0, Weekday::Mon, Default::default(), Default::default());
        assert_eq!(fact.index, 0);
        assert_eq!(fact.id, "timeslot-0");

        let _ = &fact.day_of_week;
        let _ = &fact.start_time;
        let _ = &fact.end_time;
    }
}
