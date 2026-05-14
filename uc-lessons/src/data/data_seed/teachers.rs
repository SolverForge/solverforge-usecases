use crate::domain::Teacher;

use super::vocabulary::{teacher_names, weekly_availability};

/// Builds teachers with varied availability patterns.
/// Each teacher is created from the teacher_names() list with their full name.
pub(super) fn build_teachers() -> Vec<Teacher> {
    teacher_names()
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let unavailable = [
                index % 8,
                8 + ((index * 3) % 8),
                16 + ((index * 5) % 8),
                32 + ((index * 7) % 8),
            ];

            Teacher::new(index, name.to_string(), weekly_availability(&unavailable))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_teachers() {
        let teachers = build_teachers();
        let teacher_names_list = teacher_names();
        assert_eq!(teachers.len(), teacher_names_list.len());

        for (index, teacher) in teachers.iter().enumerate() {
            assert_eq!(teacher.availability.len(), 40);
            assert!(!teacher.availability[index % 8]);
            assert_eq!(
                teacher
                    .availability
                    .iter()
                    .filter(|available| !**available)
                    .count(),
                4
            );
        }
    }

    #[test]
    fn test_teacher_names() {
        let teachers = build_teachers();
        assert_eq!(teachers.len(), 20);
        assert_eq!(teachers[0].name, "Jane Austen");
        assert_eq!(teachers[1].name, "William Shakespeare");
        assert_eq!(teachers[15].name, "Ada Lovelace");
        assert_eq!(teachers[16].name, "Alan Turing");
        assert_eq!(teachers[19].name, "Alexander von Humboldt");
    }

    #[test]
    fn test_teacher_names_from_subjects() {
        let names = teacher_names();
        assert_eq!(names.len(), 20);
        assert!(names.contains(&"Marie Curie"));
        assert!(names.contains(&"Alan Turing"));
        assert!(names.contains(&"William Shakespeare"));
    }
}
