use crate::domain::Group;

use super::vocabulary::{group_specs, weekly_availability};

/// Builds groups with varied availability patterns.
///
/// Creates groups with different availability to simulate
/// real-world constraints where groups may have restricted schedules.
pub(super) fn build_groups(count: usize) -> Vec<Group> {
    group_specs()
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, spec)| {
            let unavailable = [
                index % 8,
                8 + ((index + 2) % 8),
                16 + ((index + 4) % 8),
                24 + ((index + 6) % 8),
                32 + ((index + 1) % 8),
            ];

            Group::new(
                index,
                spec.name,
                spec.student_count,
                weekly_availability(&unavailable),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_groups_with_varied_availability() {
        let groups = build_groups(12);
        assert_eq!(groups.len(), 12);
        assert_eq!(groups[0].name, "Cohort 01");
        assert_eq!(groups[5].student_count, 34);
        assert_eq!(groups[0].availability.len(), 40);
        assert!(!groups[0].availability[0]);
        assert!(!groups[0].availability[10]);
        assert!(!groups[0].availability[20]);
        assert!(!groups[0].availability[30]);
        assert!(!groups[0].availability[33]);
        assert_eq!(
            groups[0]
                .availability
                .iter()
                .filter(|available| !**available)
                .count(),
            5
        );
    }
}
