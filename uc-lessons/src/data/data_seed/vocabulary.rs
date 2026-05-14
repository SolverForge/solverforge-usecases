//! Generator constants and shared university vocabulary.

use crate::domain::RoomKind;

// Timeslot configuration
pub(super) const LESSON_DURATION_HOURS: u32 = 1;
pub(super) const DAY_START_HOUR: u32 = 8;
pub(super) const DAY_END_HOUR: u32 = 18;
pub(super) const LUNCH_BREAK_START: u32 = 12;
pub(super) const LUNCH_BREAK_END: u32 = 14;

// Large dataset sizes
pub(super) const TIMESLOT_COUNT: usize = 40;
pub(super) const GROUP_COUNT: usize = 12;
pub(super) const ROOM_COUNT: usize = 10;

// Lesson configuration
pub(super) const LESSON_DURATION_MINUTES: u32 = 60;

/// Subject configuration with weekly demand, qualified teachers, and room type.
#[derive(Debug, Clone, Copy)]
pub(super) struct Subject {
    pub name: &'static str,
    pub hours_per_week: usize,
    pub teachers: &'static [&'static str],
    pub room_kind: RoomKind,
}

/// Fixed room inventory used by the canonical benchmark instance.
#[derive(Debug, Clone, Copy)]
pub(super) struct RoomSpec {
    pub name: &'static str,
    pub kind: RoomKind,
    pub capacity: usize,
}

/// Fixed cohort inventory used by the canonical benchmark instance.
#[derive(Debug, Clone, Copy)]
pub(super) struct GroupSpec {
    pub name: &'static str,
    pub student_count: usize,
}

/// Ordered subject catalog.
///
/// The weekly load is 25 lessons per cohort. With 12 cohorts this produces 300
/// unassigned lessons for the solver to place.
pub(super) fn subjects() -> &'static [Subject] {
    &[
        Subject {
            name: "English",
            hours_per_week: 4,
            teachers: &[
                "Jane Austen",
                "William Shakespeare",
                "Chinua Achebe",
                "Mary Shelley",
            ],
            room_kind: RoomKind::Lecture,
        },
        Subject {
            name: "Mathematics",
            hours_per_week: 4,
            teachers: &[
                "Isaac Newton",
                "Emmy Noether",
                "Katherine Johnson",
                "Florence Nightingale",
            ],
            room_kind: RoomKind::Lecture,
        },
        Subject {
            name: "Physics",
            hours_per_week: 3,
            teachers: &["Marie Curie", "Albert Einstein", "Stephen Hawking"],
            room_kind: RoomKind::Lab,
        },
        Subject {
            name: "Chemistry",
            hours_per_week: 3,
            teachers: &["Marie Curie", "Albert Einstein", "Rosalind Franklin"],
            room_kind: RoomKind::Lab,
        },
        Subject {
            name: "Biology",
            hours_per_week: 3,
            teachers: &[
                "Rosalind Franklin",
                "Charles Darwin",
                "Jane Goodall",
                "Rachel Carson",
            ],
            room_kind: RoomKind::Lab,
        },
        Subject {
            name: "Computer Science",
            hours_per_week: 2,
            teachers: &[
                "Ada Lovelace",
                "Alan Turing",
                "Grace Hopper",
                "Donald Knuth",
            ],
            room_kind: RoomKind::Computer,
        },
        Subject {
            name: "History",
            hours_per_week: 2,
            teachers: &[
                "Jane Austen",
                "William Shakespeare",
                "Chinua Achebe",
                "Mary Shelley",
            ],
            room_kind: RoomKind::Lecture,
        },
        Subject {
            name: "Geography",
            hours_per_week: 2,
            teachers: &[
                "Charles Darwin",
                "Jane Goodall",
                "Rachel Carson",
                "Alexander von Humboldt",
            ],
            room_kind: RoomKind::Lecture,
        },
        Subject {
            name: "French",
            hours_per_week: 1,
            teachers: &["Chinua Achebe", "Mary Shelley"],
            room_kind: RoomKind::Language,
        },
        Subject {
            name: "German",
            hours_per_week: 1,
            teachers: &["William Shakespeare", "Alexander von Humboldt"],
            room_kind: RoomKind::Language,
        },
    ]
}

/// Returns teacher names in first-use catalog order.
pub(super) fn teacher_names() -> Vec<&'static str> {
    let mut names = Vec::new();
    for subject in subjects() {
        for teacher in subject.teachers {
            if !names.contains(teacher) {
                names.push(*teacher);
            }
        }
    }
    names
}

/// Returns the index of a teacher name in the generated teacher list.
pub(super) fn teacher_index(name: &str) -> usize {
    teacher_names().iter().position(|&n| n == name).unwrap()
}

/// Returns the rooms available in the generated instance.
pub(super) fn room_specs() -> &'static [RoomSpec] {
    &[
        RoomSpec {
            name: "Auditorium A",
            kind: RoomKind::Lecture,
            capacity: 120,
        },
        RoomSpec {
            name: "Auditorium B",
            kind: RoomKind::Lecture,
            capacity: 80,
        },
        RoomSpec {
            name: "Seminar 1",
            kind: RoomKind::Lecture,
            capacity: 40,
        },
        RoomSpec {
            name: "Seminar 2",
            kind: RoomKind::Lecture,
            capacity: 36,
        },
        RoomSpec {
            name: "Wet Lab 1",
            kind: RoomKind::Lab,
            capacity: 36,
        },
        RoomSpec {
            name: "Wet Lab 2",
            kind: RoomKind::Lab,
            capacity: 36,
        },
        RoomSpec {
            name: "Wet Lab 3",
            kind: RoomKind::Lab,
            capacity: 36,
        },
        RoomSpec {
            name: "Computer Lab",
            kind: RoomKind::Computer,
            capacity: 36,
        },
        RoomSpec {
            name: "Language Room A",
            kind: RoomKind::Language,
            capacity: 36,
        },
        RoomSpec {
            name: "Language Room B",
            kind: RoomKind::Language,
            capacity: 36,
        },
    ]
}

/// Returns the cohorts available in the generated instance.
pub(super) fn group_specs() -> &'static [GroupSpec] {
    &[
        GroupSpec {
            name: "Cohort 01",
            student_count: 24,
        },
        GroupSpec {
            name: "Cohort 02",
            student_count: 26,
        },
        GroupSpec {
            name: "Cohort 03",
            student_count: 28,
        },
        GroupSpec {
            name: "Cohort 04",
            student_count: 30,
        },
        GroupSpec {
            name: "Cohort 05",
            student_count: 32,
        },
        GroupSpec {
            name: "Cohort 06",
            student_count: 34,
        },
        GroupSpec {
            name: "Cohort 07",
            student_count: 22,
        },
        GroupSpec {
            name: "Cohort 08",
            student_count: 25,
        },
        GroupSpec {
            name: "Cohort 09",
            student_count: 29,
        },
        GroupSpec {
            name: "Cohort 10",
            student_count: 31,
        },
        GroupSpec {
            name: "Cohort 11",
            student_count: 33,
        },
        GroupSpec {
            name: "Cohort 12",
            student_count: 27,
        },
    ]
}

/// Builds a 40-slot weekly availability vector with selected unavailable slots.
pub(super) fn weekly_availability(unavailable_slots: &[usize]) -> Vec<bool> {
    let mut availability = vec![true; TIMESLOT_COUNT];
    for slot in unavailable_slots {
        if let Some(value) = availability.get_mut(*slot) {
            *value = false;
        }
    }
    availability
}
