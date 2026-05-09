use chrono::{NaiveDate, Weekday};

use crate::domain::Employee;

use super::support::{can_mark_preference_date, mark_preference_date, PreferenceKind};
use crate::data::data_seed::employees::EmployeeBlueprint;
use crate::data::data_seed::time_utils::{choose_weekday_with_four_available_dates, weekday_dates};
use crate::data::data_seed::vocabulary::{TARGET_DESIRED_DATES, TARGET_UNDESIRED_DATES};

/// Fills the remaining preference slots with stable weekday-themed dates.
pub(super) fn top_up_preferences(
    employees: &mut [Employee],
    start_date: NaiveDate,
    blueprints: &[EmployeeBlueprint],
) {
    let dates_by_weekday = weekday_dates(start_date);

    for (employee_index, employee) in employees.iter_mut().enumerate() {
        let blueprint = &blueprints[employee_index];
        let primary_off = blueprint.primary_off_weekday;

        let preferred_weekday = choose_weekday_with_four_available_dates(
            &employee.unavailable_dates,
            primary_off,
            (0..7).map(|offset| (primary_off + 2 + offset) % 7),
        );
        let undesired_weekday = choose_weekday_with_four_available_dates(
            &employee.unavailable_dates,
            primary_off,
            (0..7).map(|offset| {
                let candidate = (primary_off + 4 + offset) % 7;
                if candidate == preferred_weekday {
                    (candidate + 1) % 7
                } else {
                    candidate
                }
            }),
        );

        for &date in &dates_by_weekday[preferred_weekday] {
            if employee.desired_dates.len() >= TARGET_DESIRED_DATES {
                break;
            }
            let _ = mark_preference_date(employee, date, PreferenceKind::Desired);
        }

        for &date in &dates_by_weekday[undesired_weekday] {
            if employee.undesired_dates.len() >= TARGET_UNDESIRED_DATES {
                break;
            }
            let _ = mark_preference_date(employee, date, PreferenceKind::Undesired);
        }
    }
}

/// Adds a tiny weekend bias to break some weekday-only symmetry.
pub(super) fn add_weekend_preference_bias(
    employees: &mut [Employee],
    start_date: NaiveDate,
    blueprints: &[EmployeeBlueprint],
) {
    let dates_by_weekday = weekday_dates(start_date);
    let saturdays = &dates_by_weekday[Weekday::Sat.num_days_from_monday() as usize];
    let sundays = &dates_by_weekday[Weekday::Sun.num_days_from_monday() as usize];

    for employee_index in 0..employees.len() {
        let weekend_mode = (blueprints[employee_index].specialty_count()
            + blueprints[employee_index].primary_off_weekday)
            % 3;

        match weekend_mode {
            0 if employees[employee_index].desired_dates.len() < TARGET_DESIRED_DATES => {
                if let Some(date) = saturdays
                    .iter()
                    .chain(sundays.iter())
                    .copied()
                    .find(|&date| {
                        can_mark_preference_date(
                            &employees[employee_index],
                            date,
                            PreferenceKind::Desired,
                        )
                    })
                {
                    let _ = mark_preference_date(
                        &mut employees[employee_index],
                        date,
                        PreferenceKind::Desired,
                    );
                }
            }
            1 if employees[employee_index].undesired_dates.len() < TARGET_UNDESIRED_DATES => {
                if let Some(date) = saturdays
                    .iter()
                    .chain(sundays.iter())
                    .copied()
                    .find(|&date| {
                        can_mark_preference_date(
                            &employees[employee_index],
                            date,
                            PreferenceKind::Undesired,
                        )
                    })
                {
                    let _ = mark_preference_date(
                        &mut employees[employee_index],
                        date,
                        PreferenceKind::Undesired,
                    );
                }
            }
            _ => {}
        }
    }
}
