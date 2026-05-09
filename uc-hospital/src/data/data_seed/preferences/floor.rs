use chrono::NaiveDate;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use crate::domain::Employee;

use super::support::{can_mark_preference_date, mark_preference_date, PreferenceKind};

/// Ensures every employee ends up with the minimum amount of preference signal.
pub(super) fn ensure_preference_floor(
    employees: &mut [Employee],
    witness_dates_by_employee: &[BTreeSet<NaiveDate>],
    coverable_dates_by_employee: &[BTreeSet<NaiveDate>],
    date_pressure: &BTreeMap<NaiveDate, usize>,
) {
    for employee_index in 0..employees.len() {
        while employees[employee_index].undesired_dates.len() < 4 {
            let candidate = witness_dates_by_employee[employee_index]
                .iter()
                .chain(coverable_dates_by_employee[employee_index].iter())
                .copied()
                .filter(|&date| {
                    can_mark_preference_date(
                        &employees[employee_index],
                        date,
                        PreferenceKind::Undesired,
                    )
                })
                .max_by_key(|date| (*date_pressure.get(date).unwrap_or(&0), Reverse(*date)));
            let Some(date) = candidate else {
                break;
            };
            let _ = mark_preference_date(
                &mut employees[employee_index],
                date,
                PreferenceKind::Undesired,
            );
        }

        while employees[employee_index].desired_dates.len() < 4 {
            let candidate = coverable_dates_by_employee[employee_index]
                .iter()
                .copied()
                .filter(|&date| !witness_dates_by_employee[employee_index].contains(&date))
                .filter(|&date| {
                    can_mark_preference_date(
                        &employees[employee_index],
                        date,
                        PreferenceKind::Desired,
                    )
                })
                .max_by_key(|date| (*date_pressure.get(date).unwrap_or(&0), Reverse(*date)))
                .or_else(|| {
                    coverable_dates_by_employee[employee_index]
                        .iter()
                        .copied()
                        .filter(|&date| {
                            can_mark_preference_date(
                                &employees[employee_index],
                                date,
                                PreferenceKind::Desired,
                            )
                        })
                        .max_by_key(|date| (*date_pressure.get(date).unwrap_or(&0), Reverse(*date)))
                });
            let Some(date) = candidate else {
                break;
            };
            let _ = mark_preference_date(
                &mut employees[employee_index],
                date,
                PreferenceKind::Desired,
            );
        }
    }
}
