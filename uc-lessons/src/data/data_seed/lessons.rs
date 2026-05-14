use crate::domain::Lesson;

use super::vocabulary::{group_specs, subjects, teacher_index, LESSON_DURATION_MINUTES};

/// Builds lessons for all groups based on subject configuration.
///
/// For each group, creates lessons for all subjects with their allocated hours_per_week.
/// Each lesson is assigned to a group, a teacher, and a required room kind.
///
/// Teacher assignment: for a given subject, group 0 gets teachers[0], group 1 gets teachers[1],
/// etc. (wrapping around if there are more groups than teachers for that subject).
///
/// Total lessons = GROUP_COUNT * sum(hours_per_week for all subjects).
/// With 12 groups and current config: 12 * 25 = 300 lessons.
pub(super) fn build_lessons(group_count: usize) -> Vec<Lesson> {
    let mut lessons = Vec::new();
    let mut lesson_index = 0;

    // For each group
    for group_idx in 0..group_count {
        let student_count = group_specs()[group_idx].student_count;

        // For each subject, create hours_per_week lessons
        for subject_config in subjects() {
            // Determine which teacher to use for this subject and group
            // Use round-robin: group_idx % number_of_teachers_for_subject
            for lesson_num in 0..subject_config.hours_per_week {
                let teacher_name = subject_config.teachers
                    [(group_idx + lesson_num) % subject_config.teachers.len()];
                let teacher_idx = teacher_index(teacher_name);

                lessons.push(Lesson::with_details(
                    lesson_index,
                    subject_config.name.to_string(),
                    group_idx,
                    student_count,
                    Some(teacher_idx),
                    LESSON_DURATION_MINUTES,
                    subject_config.room_kind,
                ));
                lesson_index += 1;
            }
        }
    }

    lessons
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_lessons() {
        let lessons = build_lessons(12);
        assert_eq!(lessons.len(), 300);
    }

    #[test]
    fn test_all_groups_have_all_subjects() {
        let lessons = build_lessons(12);
        let subject_names: Vec<&str> = subjects().iter().map(|subject| subject.name).collect();

        // Check each group has lessons for all subjects
        for group_idx in 0..12 {
            let group_lessons: Vec<_> = lessons
                .iter()
                .filter(|l| l.group_idx == group_idx)
                .collect();

            for subject_name in &subject_names {
                let subject_lessons: Vec<_> = group_lessons
                    .iter()
                    .filter(|l| l.subject == *subject_name)
                    .collect();

                let expected_count = subjects()
                    .iter()
                    .find(|subject| subject.name == *subject_name)
                    .unwrap()
                    .hours_per_week;
                assert_eq!(
                    subject_lessons.len(),
                    expected_count,
                    "Group {} should have {} lessons for {}",
                    group_idx,
                    expected_count,
                    subject_name
                );
            }
        }
    }

    #[test]
    fn test_teacher_assignment() {
        let lessons = build_lessons(12);

        // The generated catalog has four English teachers, assigned round-robin.
        let english_lessons: Vec<_> = lessons.iter().filter(|l| l.subject == "English").collect();

        assert_eq!(english_lessons.len(), 48);

        // Check teacher assignment pattern for English
        for group_idx in 0..12 {
            let group_english: Vec<_> = english_lessons
                .iter()
                .filter(|l| l.group_idx == group_idx)
                .collect();

            let expected_teacher_idx = teacher_index(
                [
                    "Jane Austen",
                    "William Shakespeare",
                    "Chinua Achebe",
                    "Mary Shelley",
                ][group_idx % 4],
            );

            assert_eq!(group_english[0].teacher_idx, Some(expected_teacher_idx));
        }
    }

    #[test]
    fn test_lesson_duration() {
        let lessons = build_lessons(12);
        for lesson in &lessons {
            assert_eq!(lesson.duration, LESSON_DURATION_MINUTES);
        }
    }

    #[test]
    fn test_subjects_are_classic_uk_school() {
        let subject_names: Vec<&str> = subjects().iter().map(|subject| subject.name).collect();

        assert!(subject_names.contains(&"Mathematics"));
        assert!(subject_names.contains(&"Physics"));
        assert!(subject_names.contains(&"Chemistry"));
        assert!(subject_names.contains(&"Biology"));
        assert!(subject_names.contains(&"Computer Science"));
        assert!(subject_names.contains(&"English"));
        assert!(subject_names.contains(&"History"));
        assert!(subject_names.contains(&"Geography"));
        assert!(subject_names.contains(&"French"));
        assert!(subject_names.contains(&"German"));
        assert_eq!(subject_names.len(), 10);
    }

    #[test]
    fn test_total_lessons_per_group() {
        let lessons = build_lessons(12);
        // Each group should have 25 lessons (sum of hours_per_week)
        for group_idx in 0..12 {
            let group_lessons: Vec<_> = lessons
                .iter()
                .filter(|l| l.group_idx == group_idx)
                .collect();
            assert_eq!(group_lessons.len(), 25);
        }
    }

    #[test]
    fn test_all_lessons_have_teacher() {
        let lessons = build_lessons(12);
        for lesson in &lessons {
            assert!(
                lesson.teacher_idx.is_some(),
                "Lesson {} has no teacher",
                lesson.id
            );
        }
    }
}
