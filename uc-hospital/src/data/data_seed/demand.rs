use chrono::Weekday;

use super::vocabulary::*;

/// Reusable calendar patterns for demand templates.
#[derive(Clone, Copy)]
pub(super) enum WeekPattern {
    Weekdays,
    Weekends,
    Daily,
    MonWedFri,
    Saturday,
    Sunday,
}

#[derive(Clone, Copy)]
pub(super) struct DemandRule {
    pub(super) location: &'static str,
    pub(super) start_hour: u32,
    pub(super) required_skill: &'static str,
    pattern: WeekPattern,
    pub(super) count: usize,
}

// This is the published demand template for the 28-day benchmark. Each rule
// says "on matching weekdays, create `count` eight-hour shifts of this shape".
pub(super) const DEMAND_RULES: &[DemandRule] = &[
    DemandRule {
        location: "Ambulatory care",
        start_hour: 6,
        required_skill: AMBULATORY_DOCTOR,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Ambulatory care",
        start_hour: 14,
        required_skill: AMBULATORY_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Ambulatory care",
        start_hour: 6,
        required_skill: AMBULATORY_DOCTOR,
        pattern: WeekPattern::Weekends,
        count: 1,
    },
    DemandRule {
        location: "Ambulatory care",
        start_hour: 14,
        required_skill: AMBULATORY_NURSE,
        pattern: WeekPattern::Weekends,
        count: 2,
    },
    DemandRule {
        location: "Neurology",
        start_hour: 6,
        required_skill: NEUROLOGY_DOCTOR,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Neurology",
        start_hour: 14,
        required_skill: NEUROLOGY_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Neurology",
        start_hour: 6,
        required_skill: NEUROLOGY_DOCTOR,
        pattern: WeekPattern::Weekends,
        count: 1,
    },
    DemandRule {
        location: "Neurology",
        start_hour: 14,
        required_skill: NEUROLOGY_NURSE,
        pattern: WeekPattern::Weekends,
        count: 1,
    },
    DemandRule {
        location: "Neurology",
        start_hour: 22,
        required_skill: CARDIOLOGY,
        pattern: WeekPattern::MonWedFri,
        count: 1,
    },
    DemandRule {
        location: "Critical care",
        start_hour: 6,
        required_skill: CRITICAL_DOCTOR,
        pattern: WeekPattern::Daily,
        count: 2,
    },
    DemandRule {
        location: "Critical care",
        start_hour: 14,
        required_skill: CRITICAL_NURSE,
        pattern: WeekPattern::Daily,
        count: 2,
    },
    DemandRule {
        location: "Critical care",
        start_hour: 22,
        required_skill: CRITICAL_DOCTOR,
        pattern: WeekPattern::Daily,
        count: 1,
    },
    DemandRule {
        location: "Critical care",
        start_hour: 9,
        required_skill: CRITICAL_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Pediatric care",
        start_hour: 6,
        required_skill: PEDIATRIC_DOCTOR,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Pediatric care",
        start_hour: 14,
        required_skill: PEDIATRIC_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Pediatric care",
        start_hour: 6,
        required_skill: PEDIATRIC_DOCTOR,
        pattern: WeekPattern::Weekends,
        count: 1,
    },
    DemandRule {
        location: "Pediatric care",
        start_hour: 14,
        required_skill: PEDIATRIC_NURSE,
        pattern: WeekPattern::Weekends,
        count: 2,
    },
    DemandRule {
        location: "Surgery",
        start_hour: 6,
        required_skill: SURGERY_DOCTOR,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Surgery",
        start_hour: 14,
        required_skill: ANAESTHETICS,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Surgery",
        start_hour: 22,
        required_skill: SURGERY_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 6,
        required_skill: RADIOLOGY_DAY,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 9,
        required_skill: RADIOLOGY_DAY,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 14,
        required_skill: RADIOLOGY_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 22,
        required_skill: RADIOLOGY_CALL,
        pattern: WeekPattern::MonWedFri,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 6,
        required_skill: RADIOLOGY_DAY,
        pattern: WeekPattern::Saturday,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 9,
        required_skill: RADIOLOGY_DAY,
        pattern: WeekPattern::Saturday,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 14,
        required_skill: RADIOLOGY_NURSE,
        pattern: WeekPattern::Saturday,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 6,
        required_skill: RADIOLOGY_DAY,
        pattern: WeekPattern::Sunday,
        count: 1,
    },
    DemandRule {
        location: "Radiology",
        start_hour: 9,
        required_skill: RADIOLOGY_DAY,
        pattern: WeekPattern::Sunday,
        count: 1,
    },
    DemandRule {
        location: "Outpatient",
        start_hour: 6,
        required_skill: OUTPATIENT_NURSE,
        pattern: WeekPattern::Weekdays,
        count: 2,
    },
    DemandRule {
        location: "Outpatient",
        start_hour: 14,
        required_skill: OUTPATIENT_DOCTOR,
        pattern: WeekPattern::Weekdays,
        count: 1,
    },
];

impl DemandRule {
    /// Expands a rule into the number of shifts it contributes on a specific weekday.
    pub(super) fn count_for_date(&self, weekday: Weekday) -> usize {
        if self.pattern.matches(weekday) {
            self.count
        } else {
            0
        }
    }
}

impl WeekPattern {
    /// Returns whether the abstract pattern includes the given weekday.
    fn matches(self, weekday: Weekday) -> bool {
        match self {
            WeekPattern::Weekdays => matches!(
                weekday,
                Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri
            ),
            WeekPattern::Weekends => matches!(weekday, Weekday::Sat | Weekday::Sun),
            WeekPattern::Daily => true,
            WeekPattern::MonWedFri => matches!(weekday, Weekday::Mon | Weekday::Wed | Weekday::Fri),
            WeekPattern::Saturday => weekday == Weekday::Sat,
            WeekPattern::Sunday => weekday == Weekday::Sun,
        }
    }
}
