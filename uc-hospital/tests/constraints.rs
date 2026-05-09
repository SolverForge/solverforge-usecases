use chrono::{NaiveDate, NaiveDateTime};
use solverforge::prelude::*;
use solverforge_hospital::constraints::create_constraints;
use solverforge_hospital::domain::{Employee, Plan, Shift};

// These tests are intentionally tiny and direct: each one isolates one rule so
// a beginner can see how the domain model turns into score changes.

const SCORE_SCALE: i64 = 100_000;
const UNASSIGNED_SHIFT_HARD_UNITS: i64 = 1;
const REQUIRED_SKILL_HARD_UNITS: i64 = 10;
const STRUCTURAL_FIXED_HARD_UNITS: i64 = 20;
const STRUCTURAL_MINUTE_HARD_UNITS: i64 = 20;

// Short helper that keeps the test data readable.
fn dt(day: u32, hour: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2024, 1, day)
        .unwrap()
        .and_hms_opt(hour, 0, 0)
        .unwrap()
}

// Builds a hard score using the same scaling constants as the production constraints.
fn hard_units(units: i64) -> HardSoftDecimalScore {
    HardSoftDecimalScore::of_hard_scaled(units * SCORE_SCALE)
}

// Builds a minute-weighted hard score for overlap/rest style rules.
fn structural_hard_minutes(minutes: i64) -> HardSoftDecimalScore {
    HardSoftDecimalScore::of_hard_scaled(minutes * STRUCTURAL_MINUTE_HARD_UNITS * SCORE_SCALE)
}

#[test]
fn self_joins_count_each_pair_once() {
    let employees = vec![Employee::new(0, "A").with_skill("Doctor")];
    let mut shifts = vec![
        Shift::new("1", dt(1, 8), dt(1, 16), "Ward", "Doctor"),
        Shift::new("2", dt(1, 12), dt(1, 20), "Ward", "Doctor"),
    ];
    shifts[0].employee_idx = Some(0);
    shifts[1].employee_idx = Some(0);

    let schedule = Plan::new(employees, shifts);
    let constraints = create_constraints();
    let analyses = constraints.evaluate_detailed(&schedule);
    let overlap = analyses
        .into_iter()
        .find(|analysis| analysis.constraint_ref.name == "Overlapping shift")
        .unwrap();

    assert_eq!(overlap.matches.len(), 1);
}

#[test]
fn unassigned_shift_is_a_hard_violation() {
    let schedule = Plan::new(
        vec![],
        vec![Shift::new("1", dt(1, 8), dt(1, 16), "Ward", "Doctor")],
    );

    let score = create_constraints().evaluate_all(&schedule);

    assert_eq!(score.hard_score(), hard_units(-UNASSIGNED_SHIFT_HARD_UNITS));
}

#[test]
fn overnight_shift_penalizes_unavailable_end_date() {
    let mut employee = Employee::new(0, "Night nurse")
        .with_skill("Nurse")
        .with_unavailable_date(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
    employee.finalize();
    assert_eq!(employee.unavailable_days.len(), 1);

    let mut shift = Shift::new("night-1", dt(1, 22), dt(2, 6), "Ward", "Nurse");
    shift.employee_idx = Some(0);

    assert_eq!(
        {
            let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
            let day_start = date.and_hms_opt(0, 0, 0).unwrap();
            let day_end = date
                .succ_opt()
                .unwrap_or(date)
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let start = shift.start.max(day_start);
            let end = shift.end.min(day_end);
            if start < end {
                (end - start).num_minutes()
            } else {
                0
            }
        },
        360
    );

    let schedule = Plan::new(vec![employee], vec![shift]);
    let constraints = create_constraints();
    let analyses = constraints.evaluate_detailed(&schedule);
    let unavailable = analyses
        .into_iter()
        .find(|analysis| analysis.constraint_ref.name == "Unavailable employee")
        .unwrap();

    assert_eq!(
        unavailable.score.hard_score(),
        structural_hard_minutes(-360)
    );
}

#[test]
fn overnight_shift_rewards_desired_end_date() {
    let mut employee = Employee::new(0, "Night nurse")
        .with_skill("Nurse")
        .with_desired_date(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
    employee.finalize();
    assert_eq!(employee.desired_days.len(), 1);

    let mut shift = Shift::new("night-1", dt(1, 22), dt(2, 6), "Ward", "Nurse");
    shift.employee_idx = Some(0);

    assert!({
        let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let day_start = date.and_hms_opt(0, 0, 0).unwrap();
        let day_end = date
            .succ_opt()
            .unwrap_or(date)
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let start = shift.start.max(day_start);
        let end = shift.end.min(day_end);
        start < end && (end - start).num_minutes() > 0
    });

    let schedule = Plan::new(vec![employee], vec![shift]);
    let constraints = create_constraints();
    let analyses = constraints.evaluate_detailed(&schedule);
    let desired = analyses
        .into_iter()
        .find(|analysis| analysis.constraint_ref.name == "Desired day for employee")
        .unwrap();

    assert_eq!(desired.score.soft_score(), HardSoftDecimalScore::of_soft(1));
}

#[test]
fn employee_joined_analysis_reports_required_skill_match() {
    let employees = vec![Employee::new(0, "Taylor").with_skill("Nurse")];
    let mut shift = Shift::new("1", dt(1, 8), dt(1, 16), "Ward", "Doctor");
    shift.employee_idx = Some(0);

    let schedule = Plan::new(employees, vec![shift]);
    let constraints = create_constraints();
    let analyses = constraints.evaluate_detailed(&schedule);
    let required_skill = analyses
        .into_iter()
        .find(|analysis| analysis.constraint_ref.name == "Required skill")
        .unwrap();

    assert_eq!(required_skill.matches.len(), 1);
    assert_eq!(
        required_skill.score.hard_score(),
        hard_units(-REQUIRED_SKILL_HARD_UNITS)
    );
}

#[test]
fn overnight_and_next_day_shift_violate_one_shift_per_day() {
    let employees = vec![Employee::new(0, "A").with_skill("Doctor")];
    let mut overnight = Shift::new("night", dt(1, 22), dt(2, 6), "Ward", "Doctor");
    overnight.employee_idx = Some(0);
    let mut evening = Shift::new("late", dt(2, 18), dt(2, 22), "Ward", "Doctor");
    evening.employee_idx = Some(0);

    let schedule = Plan::new(employees, vec![overnight, evening]);
    let constraints = create_constraints();
    let analyses = constraints.evaluate_detailed(&schedule);
    let one_per_day = analyses
        .into_iter()
        .find(|analysis| analysis.constraint_ref.name == "One shift per day")
        .unwrap();

    assert_eq!(one_per_day.matches.len(), 1);
    assert_eq!(
        one_per_day.score.hard_score(),
        hard_units(-STRUCTURAL_FIXED_HARD_UNITS)
    );
}

#[test]
fn required_skill_is_worse_than_unassigned() {
    let unassigned = create_constraints().evaluate_all(&Plan::new(
        vec![],
        vec![Shift::new("missing", dt(1, 8), dt(1, 16), "Ward", "Doctor")],
    ));

    let employees = vec![Employee::new(0, "Taylor").with_skill("Nurse")];
    let mut shift = Shift::new("wrong-skill", dt(1, 8), dt(1, 16), "Ward", "Doctor");
    shift.employee_idx = Some(0);
    let wrong_skill = create_constraints().evaluate_all(&Plan::new(employees, vec![shift]));

    assert!(wrong_skill < unassigned);
    assert_eq!(
        unassigned.hard_score(),
        hard_units(-UNASSIGNED_SHIFT_HARD_UNITS)
    );
    assert_eq!(
        wrong_skill.hard_score(),
        hard_units(-REQUIRED_SKILL_HARD_UNITS)
    );
}

#[test]
fn one_shift_per_day_does_not_cross_employee_boundaries() {
    let employees = vec![
        Employee::new(0, "A").with_skill("Doctor"),
        Employee::new(1, "B").with_skill("Doctor"),
    ];
    let mut overnight = Shift::new("night", dt(1, 22), dt(2, 6), "Ward", "Doctor");
    overnight.employee_idx = Some(0);
    let mut evening = Shift::new("late", dt(2, 18), dt(2, 22), "Ward", "Doctor");
    evening.employee_idx = Some(1);

    let schedule = Plan::new(employees, vec![overnight, evening]);
    let constraints = create_constraints();
    let analyses = constraints.evaluate_detailed(&schedule);
    let one_per_day = analyses
        .into_iter()
        .find(|analysis| analysis.constraint_ref.name == "One shift per day")
        .unwrap();

    assert!(one_per_day.matches.is_empty());
    assert_eq!(one_per_day.score, HardSoftDecimalScore::ZERO);
}
