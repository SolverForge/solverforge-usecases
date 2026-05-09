use chrono::Timelike;

use crate::domain::{Employee, Shift};

use super::coverage::{candidate_redundancy_is_valid, public_candidate_counts};

/// Verifies that the exact public dataset we will ship still matches generator goals.
pub(super) fn validate_public_dataset(employees: &[Employee], shifts: &[Shift]) {
    let counts = public_candidate_counts(employees, shifts);
    let min_count = counts.iter().copied().min().unwrap_or(0);
    let three_plus = counts.iter().filter(|&&count| count >= 3).count();
    let weakest: Vec<String> = shifts
        .iter()
        .zip(counts.iter())
        .filter(|(_, &count)| count == min_count)
        .take(5)
        .map(|(shift, &count)| {
            format!(
                "{} {} {} -> {}",
                shift.location,
                shift.start.time().hour(),
                shift.required_skill,
                count
            )
        })
        .collect();
    assert!(
        candidate_redundancy_is_valid(employees, shifts),
        "public dataset should maintain candidate redundancy; min_count={min_count}, three_plus={three_plus}/{} weakest={weakest:?}",
        counts.len()
    );
}
