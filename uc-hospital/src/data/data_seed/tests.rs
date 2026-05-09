use super::availability::add_extra_unavailability;
use super::cohorts::assign_primary_off_days;
use super::coverage::public_candidate_counts;
use super::demand::DEMAND_RULES;
use super::employees::{build_employee_blueprints, instantiate_employees};
use super::preferences::{
    add_preferences, eligible_employees_by_shift, shift_soft_preference_score,
};
use super::shifts::{build_public_shifts, prepare_shifts};
use super::time_utils::find_next_monday;
use super::vocabulary::*;
use super::witness::build_hidden_witness;
use super::*;
use chrono::{Datelike, Duration, NaiveDate, Timelike};
use rand::rngs::StdRng;
use rand::SeedableRng;
use solverforge::ConstraintSet;
use std::collections::{BTreeMap, BTreeSet};

use crate::domain::Plan;

// These tests lock down the generator contract: workforce shape, shift counts,
// feasibility margins, and the intended soft-score signal surface.

fn schedule() -> Plan {
    generate(DemoData::Large)
}

#[test]
fn test_generate_large() {
    let schedule = schedule();

    assert_eq!(schedule.employees.len(), 50);
    assert_eq!(schedule.shifts.len(), 688);
}

#[test]
fn test_exact_workforce_composition() {
    let schedule = schedule();
    let employees = &schedule.employees;

    let doctors = employees
        .iter()
        .filter(|employee| employee.skills.contains(DOCTOR))
        .count();
    let nurses = employees
        .iter()
        .filter(|employee| employee.skills.contains(NURSE))
        .count();
    let cardiology = employees
        .iter()
        .filter(|employee| employee.skills.contains(CARDIOLOGY))
        .count();
    let anaesthetics = employees
        .iter()
        .filter(|employee| employee.skills.contains(ANAESTHETICS))
        .count();
    let radiology_day = employees
        .iter()
        .filter(|employee| employee.skills.contains(RADIOLOGY_DAY))
        .count();
    let radiology_nurse = employees
        .iter()
        .filter(|employee| employee.skills.contains(RADIOLOGY_NURSE))
        .count();
    let radiology_call = employees
        .iter()
        .filter(|employee| employee.skills.contains(RADIOLOGY_CALL))
        .count();
    let ambulatory_doctors = employees
        .iter()
        .filter(|employee| employee.skills.contains(AMBULATORY_DOCTOR))
        .count();
    let ambulatory_nurses = employees
        .iter()
        .filter(|employee| employee.skills.contains(AMBULATORY_NURSE))
        .count();
    let critical_doctors = employees
        .iter()
        .filter(|employee| employee.skills.contains(CRITICAL_DOCTOR))
        .count();
    let critical_nurses = employees
        .iter()
        .filter(|employee| employee.skills.contains(CRITICAL_NURSE))
        .count();
    let outpatient_doctors = employees
        .iter()
        .filter(|employee| employee.skills.contains(OUTPATIENT_DOCTOR))
        .count();
    let outpatient_nurses = employees
        .iter()
        .filter(|employee| employee.skills.contains(OUTPATIENT_NURSE))
        .count();

    assert_eq!(doctors, 22);
    assert_eq!(nurses, 28);
    assert_eq!(cardiology, 4);
    assert_eq!(anaesthetics, 6);
    assert_eq!(radiology_day, 6);
    assert_eq!(radiology_nurse, 6);
    assert_eq!(radiology_call, 4);
    assert_eq!(ambulatory_doctors, 4);
    assert_eq!(ambulatory_nurses, 6);
    assert_eq!(critical_doctors, 6);
    assert_eq!(critical_nurses, 8);
    assert_eq!(outpatient_doctors, 7);
    assert_eq!(outpatient_nurses, 9);
}

#[test]
fn test_exact_shift_template_counts() {
    let schedule = schedule();
    let mut actual = BTreeMap::<(String, u32, String), usize>::new();
    for shift in &schedule.shifts {
        *actual
            .entry((
                shift.location.clone(),
                shift.start.time().hour(),
                shift.required_skill.clone(),
            ))
            .or_default() += 1;
    }

    let mut expected = BTreeMap::<(String, u32, String), usize>::new();
    let start_date = find_next_monday(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    for day in 0..DAYS_IN_SCHEDULE {
        let date = start_date + Duration::days(day);
        for rule in DEMAND_RULES {
            *expected
                .entry((
                    rule.location.to_string(),
                    rule.start_hour,
                    rule.required_skill.to_string(),
                ))
                .or_default() += rule.count_for_date(date.weekday());
        }
    }

    assert_eq!(actual, expected);
}

#[test]
fn test_preferences_are_disjoint_from_unavailability() {
    let schedule = schedule();
    for employee in &schedule.employees {
        assert!(employee
            .desired_dates
            .iter()
            .all(|date| !employee.unavailable_dates.contains(date)));
        assert!(employee
            .undesired_dates
            .iter()
            .all(|date| !employee.unavailable_dates.contains(date)));
        assert!((4..=MAX_DESIRED_DATES).contains(&employee.desired_dates.len()));
        assert!((4..=MAX_UNDESIRED_DATES).contains(&employee.undesired_dates.len()));
        assert!(employee
            .desired_dates
            .iter()
            .all(|date| !employee.undesired_dates.contains(date)));
    }
}

#[test]
fn test_preference_surface_has_one_move_signal() {
    let schedule = schedule();
    let witness = build_hidden_witness(&schedule.employees, &schedule.shifts);
    let candidate_lists = eligible_employees_by_shift(&schedule.employees, &schedule.shifts);
    let signal_shifts = schedule
        .shifts
        .iter()
        .enumerate()
        .filter(|(shift_index, shift)| {
            let holder = witness.assignments[*shift_index];
            let holder_score = shift_soft_preference_score(&schedule.employees[holder], shift);
            candidate_lists[*shift_index]
                .iter()
                .copied()
                .filter(|&candidate| candidate != holder)
                .any(|candidate| {
                    let candidate_score =
                        shift_soft_preference_score(&schedule.employees[candidate], shift);
                    candidate_score - holder_score >= 2
                })
        })
        .count();

    assert!(
        signal_shifts > 0,
        "there should still be some one-move soft improvements"
    );
}

#[test]
fn test_public_candidate_redundancy() {
    let schedule = schedule();
    let counts = public_candidate_counts(&schedule.employees, &schedule.shifts);
    assert!(counts.iter().all(|&count| count >= 2));
    assert!(counts.iter().filter(|&&count| count >= 3).count() * 4 >= counts.len());
}

#[test]
fn test_candidate_width_is_not_generic_role_wide() {
    let schedule = schedule();
    let mut counts = public_candidate_counts(&schedule.employees, &schedule.shifts);
    counts.sort_unstable();
    let median = counts[counts.len() / 2];
    let p90 = counts[counts.len() * 9 / 10];

    assert!(median <= 7, "median candidate count should stay narrow");
    assert!(
        p90 <= 9,
        "90th percentile candidate count should stay bounded"
    );
}

#[test]
fn test_hidden_witness_is_hard_feasible() {
    let mut rng = StdRng::seed_from_u64(0);
    let start_date = find_next_monday(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    let mut blueprints = build_employee_blueprints(&mut rng);
    assign_primary_off_days(&mut blueprints);
    let mut employees = instantiate_employees(&blueprints, start_date);
    let mut shifts = build_public_shifts(start_date);
    prepare_shifts(&mut shifts);

    let witness = build_hidden_witness(&employees, &shifts);
    add_extra_unavailability(&mut employees, &shifts, &witness.employee_touched_dates);
    add_preferences(&mut employees, start_date, &blueprints, &shifts, &witness);

    for (shift, employee_idx) in shifts.iter_mut().zip(witness.assignments.iter()) {
        shift.employee_idx = Some(*employee_idx);
    }

    let witness_schedule = Plan::new(employees, shifts);
    let score = crate::constraints::create_constraints().evaluate_all(&witness_schedule);
    assert_eq!(score.hard_score(), solverforge::HardSoftDecimalScore::ZERO);
}

#[test]
fn test_employees_have_skills() {
    let schedule = schedule();

    for employee in &schedule.employees {
        assert!(
            !employee.skills.is_empty(),
            "Employee {} has no skills",
            employee.name
        );
    }
}

#[test]
fn test_demo_data_from_str() {
    assert_eq!("LARGE".parse::<DemoData>(), Ok(DemoData::Large));
    assert_eq!("large".parse::<DemoData>(), Ok(DemoData::Large));
    assert!("invalid".parse::<DemoData>().is_err());
}

#[test]
fn test_medical_domain() {
    let schedule = schedule();

    let all_skills: BTreeSet<_> = schedule
        .employees
        .iter()
        .flat_map(|employee| employee.skills.iter())
        .map(|skill| skill.as_str())
        .collect();

    assert!(all_skills.contains(DOCTOR) || all_skills.contains(NURSE));
    let locations: BTreeSet<_> = schedule
        .shifts
        .iter()
        .map(|shift| shift.location.as_str())
        .collect();
    assert!(locations.contains("Ambulatory care") || locations.contains("Critical care"));
}

#[test]
fn test_empty_schedule_has_score() {
    let schedule = crate::domain::Plan::new(vec![], vec![]);
    let score = crate::constraints::create_constraints().evaluate_all(&schedule);
    assert_eq!(score.to_string(), "0hard/0soft");
}
