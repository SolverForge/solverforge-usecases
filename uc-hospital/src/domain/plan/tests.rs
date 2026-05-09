//! Tests for plan invariants, transport normalization, and embedded solver config.

use super::*;
use crate::api::PlanDto;
use solverforge::{SolverEvent, SolverManager, SolverTerminalReason};
use std::fs;
use std::sync::{Mutex, OnceLock};

#[test]
fn scalar_variable_uses_solution_level_employee_range_and_nearby_hooks() {
    let descriptor = Plan::descriptor();
    let shift_descriptor = descriptor
        .find_entity_descriptor("Shift")
        .expect("Shift descriptor should exist");
    let variable = shift_descriptor
        .find_variable("employee_idx")
        .expect("employee_idx variable should exist");

    assert_eq!(variable.value_range_provider, Some("employees"));
    assert!(variable.candidate_values.is_some());
    assert!(variable.nearby_value_candidates.is_some());
    assert!(variable.nearby_entity_candidates.is_some());
    assert!(variable.nearby_value_distance_meter.is_some());
    assert!(variable.nearby_entity_distance_meter.is_some());
    assert!(variable.construction_entity_order_key.is_none());
    assert!(variable.construction_value_order_key.is_none());
}

#[test]
fn plan_dto_round_trip_rebuilds_domain_invariants() {
    let schedule = Plan::new(
        vec![
            Employee::new(0, "Alex")
                .with_skill("Doctor")
                .with_unavailable_date(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap())
                .with_undesired_date(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap())
                .with_desired_date(NaiveDate::from_ymd_opt(2024, 1, 4).unwrap()),
            Employee::new(1, "Taylor").with_skill("Nurse"),
        ],
        vec![{
            let mut shift = Shift::new(
                "shift-1",
                NaiveDate::from_ymd_opt(2024, 1, 2)
                    .unwrap()
                    .and_hms_opt(8, 0, 0)
                    .unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 2)
                    .unwrap()
                    .and_hms_opt(16, 0, 0)
                    .unwrap(),
                "ER",
                "Nurse",
            );
            shift.employee_idx = Some(1);
            shift
        }],
    );

    let dto = PlanDto::from_plan(&schedule);
    let json = serde_json::to_value(&dto).unwrap();

    assert_eq!(json["shifts"][0]["employeeIdx"], 1);
    assert!(json["shifts"][0].get("assignedEmployeeId").is_none());
    assert!(json["shifts"][0].get("employeeRange").is_none());
    assert!(json["employees"][0].get("index").is_none());
    assert!(json["employees"][0].get("unavailableDays").is_none());
    assert_eq!(json["employees"][0]["unavailableDates"][0], "2024-01-02");
    assert_eq!(json["employees"][0]["undesiredDates"][0], "2024-01-03");
    assert_eq!(json["employees"][0]["desiredDates"][0], "2024-01-04");

    let round_tripped = dto.to_domain().unwrap();
    assert_eq!(round_tripped.shifts[0].employee_idx, Some(1));
    assert_eq!(round_tripped.shifts[0].index, 0);
    assert_eq!(
        round_tripped.shifts[0].touched_dates,
        vec![NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()]
    );
    assert_eq!(round_tripped.employees[0].index, 0);
    assert_eq!(
        round_tripped.employees[0].unavailable_days,
        vec![NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()]
    );
}

#[test]
fn overnight_shift_tracks_each_touched_date_once() {
    let schedule = Plan::new(
        vec![Employee::new(0, "Alex").with_skill("Doctor")],
        vec![Shift::new(
            "night-1",
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(22, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(6, 0, 0)
                .unwrap(),
            "ER",
            "Doctor",
        )],
    );

    assert_eq!(
        schedule.shifts[0].touched_dates(),
        &[
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        ]
    );
}

fn cwd_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TempSolverConfigDir {
    original_dir: std::path::PathBuf,
    temp_dir: std::path::PathBuf,
}

impl TempSolverConfigDir {
    fn new(contents: &str) -> Self {
        let original_dir = std::env::current_dir().expect("current directory should be readable");
        let temp_dir = std::env::temp_dir().join(format!(
            "solverforge-hospital-config-test-{}",
            std::process::id()
        ));

        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("temp solver directory should be created");
        fs::write(temp_dir.join("solver.toml"), contents)
            .expect("temp solver.toml should be written");
        std::env::set_current_dir(&temp_dir).expect("current directory should switch to temp");

        Self {
            original_dir,
            temp_dir,
        }
    }
}

impl Drop for TempSolverConfigDir {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir)
            .expect("current directory should restore after test");
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn retained_runtime_uses_embedded_solver_toml_instead_of_cwd_defaults() {
    static MANAGER: SolverManager<Plan> = SolverManager::new();

    let _cwd_lock = cwd_test_lock().lock().expect("cwd lock should be acquired");
    let _temp_solver_dir = TempSolverConfigDir::new("this = definitely not valid toml = [");

    let (job_id, mut receiver) = MANAGER
        .solve(Plan::new(Vec::new(), Vec::new()))
        .expect("job should start");

    let mut completed = false;

    while let Some(event) = receiver.blocking_recv() {
        match event {
            SolverEvent::BestSolution { .. } => {}
            SolverEvent::Completed { metadata, solution } => {
                assert_eq!(
                    metadata.terminal_reason,
                    Some(SolverTerminalReason::Completed)
                );
                assert_eq!(solution.score, Some(HardSoftDecimalScore::ZERO));
                completed = true;
                break;
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    assert!(completed, "expected a completed event");
    MANAGER.delete(job_id).expect("delete completed job");
}

#[test]
fn retained_runtime_assigns_single_shift_when_compatible_candidate_exists() {
    static MANAGER: SolverManager<Plan> = SolverManager::new();

    let schedule = Plan::new(
        vec![
            Employee::new(0, "Unavailable doctor")
                .with_skill("Doctor")
                .with_unavailable_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Employee::new(1, "Available doctor").with_skill("Doctor"),
        ],
        vec![Shift::new(
            "shift-1",
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(8, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(16, 0, 0)
                .unwrap(),
            "ER",
            "Doctor",
        )],
    );

    let (job_id, mut receiver) = MANAGER.solve(schedule).expect("job should start");
    let mut completed_solution = None;

    while let Some(event) = receiver.blocking_recv() {
        match event {
            SolverEvent::BestSolution { .. } => {}
            SolverEvent::Completed { solution, .. } => {
                completed_solution = Some(solution);
                break;
            }
            SolverEvent::Failed { error, .. } => {
                panic!("retained solve failed unexpectedly: {error}");
            }
            _ => {}
        }
    }

    let solution = completed_solution.expect("expected a completed solution");
    assert_eq!(solution.shifts[0].employee_idx, Some(1));
    assert_eq!(solution.score, Some(HardSoftDecimalScore::ZERO));

    MANAGER.delete(job_id).expect("delete completed job");
}
