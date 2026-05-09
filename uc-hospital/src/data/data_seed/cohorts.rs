use std::cmp::Reverse;

use super::employees::EmployeeBlueprint;
use super::vocabulary::*;

/// Running totals for one weekday-off cohort.
///
/// We use this to distribute scarce specialties across the seven primary
/// off-day groups instead of accidentally clustering too many similar people on
/// the same day off.
#[derive(Default, Clone, Copy)]
struct CohortLoad {
    size: usize,
    doctors: usize,
    nurses: usize,
    ambulatory_doctors: usize,
    ambulatory_nurses: usize,
    neurology_doctors: usize,
    neurology_nurses: usize,
    critical_doctors: usize,
    critical_nurses: usize,
    pediatric_doctors: usize,
    pediatric_nurses: usize,
    surgery_doctors: usize,
    surgery_nurses: usize,
    outpatient_doctors: usize,
    outpatient_nurses: usize,
    radiology_day: usize,
    radiology_nurses: usize,
    radiology_call: usize,
    cardiology: usize,
    anaesthetics: usize,
}

impl CohortLoad {
    /// Updates the cohort totals after placing one blueprint into it.
    fn add(&mut self, blueprint: &EmployeeBlueprint) {
        self.size += 1;
        if blueprint.skills.contains(DOCTOR) {
            self.doctors += 1;
        }
        if blueprint.skills.contains(NURSE) {
            self.nurses += 1;
        }
        if blueprint.skills.contains(AMBULATORY_DOCTOR) {
            self.ambulatory_doctors += 1;
        }
        if blueprint.skills.contains(AMBULATORY_NURSE) {
            self.ambulatory_nurses += 1;
        }
        if blueprint.skills.contains(NEUROLOGY_DOCTOR) {
            self.neurology_doctors += 1;
        }
        if blueprint.skills.contains(NEUROLOGY_NURSE) {
            self.neurology_nurses += 1;
        }
        if blueprint.skills.contains(CRITICAL_DOCTOR) {
            self.critical_doctors += 1;
        }
        if blueprint.skills.contains(CRITICAL_NURSE) {
            self.critical_nurses += 1;
        }
        if blueprint.skills.contains(PEDIATRIC_DOCTOR) {
            self.pediatric_doctors += 1;
        }
        if blueprint.skills.contains(PEDIATRIC_NURSE) {
            self.pediatric_nurses += 1;
        }
        if blueprint.skills.contains(SURGERY_DOCTOR) {
            self.surgery_doctors += 1;
        }
        if blueprint.skills.contains(SURGERY_NURSE) {
            self.surgery_nurses += 1;
        }
        if blueprint.skills.contains(OUTPATIENT_DOCTOR) {
            self.outpatient_doctors += 1;
        }
        if blueprint.skills.contains(OUTPATIENT_NURSE) {
            self.outpatient_nurses += 1;
        }
        if blueprint.skills.contains(RADIOLOGY_DAY) {
            self.radiology_day += 1;
        }
        if blueprint.skills.contains(RADIOLOGY_NURSE) {
            self.radiology_nurses += 1;
        }
        if blueprint.skills.contains(RADIOLOGY_CALL) {
            self.radiology_call += 1;
        }
        if blueprint.skills.contains(CARDIOLOGY) {
            self.cardiology += 1;
        }
        if blueprint.skills.contains(ANAESTHETICS) {
            self.anaesthetics += 1;
        }
    }
}

/// Assigns each employee blueprint a stable primary off weekday.
pub(super) fn assign_primary_off_days(blueprints: &mut [EmployeeBlueprint]) {
    let mut order: Vec<usize> = (0..blueprints.len()).collect();
    order.sort_by_key(|&index| Reverse(blueprint_priority(&blueprints[index])));

    let mut loads = [CohortLoad::default(); 7];

    for employee_index in order {
        let cohort = (0..7)
            .filter(|&candidate| loads[candidate].size < PRIMARY_OFF_COHORT_SIZES[candidate])
            .min_by_key(|&candidate| cohort_score(&loads[candidate], &blueprints[employee_index]))
            .expect("cohort should have spare capacity");
        blueprints[employee_index].primary_off_weekday = cohort;
        loads[cohort].add(&blueprints[employee_index]);
    }
}

/// Scarcer or more specialized blueprints get placed first.
fn blueprint_priority(blueprint: &EmployeeBlueprint) -> (usize, usize, usize, usize, usize) {
    (
        blueprint.specialty_count(),
        usize::from(blueprint.skills.contains(DOCTOR)),
        usize::from(blueprint.skills.contains(CARDIOLOGY)),
        usize::from(blueprint.skills.contains(ANAESTHETICS)),
        usize::from(blueprint.skills.contains(RADIOLOGY_CALL)),
    )
}

/// Lower scores mean "this cohort needs this blueprint more".
fn cohort_score(load: &CohortLoad, blueprint: &EmployeeBlueprint) -> (usize, usize, usize, usize) {
    let line_pressure = weighted_line_load(load, blueprint);
    (
        line_pressure,
        if blueprint.skills.contains(DOCTOR) {
            load.doctors
        } else {
            load.nurses
        },
        load.size,
        blueprint.primary_off_weekday,
    )
}

/// Applies weighted pressure so rare specialties dominate balancing decisions.
fn weighted_line_load(load: &CohortLoad, blueprint: &EmployeeBlueprint) -> usize {
    let mut score = 0usize;
    for (skill, weight) in line_balance_weights() {
        if blueprint.has_skill(skill) {
            score += line_load_for_skill(load, skill) * weight;
        }
    }
    score
}

/// Manual weights that treat some specialties as harder to concentrate.
fn line_balance_weights() -> &'static [(&'static str, usize)] {
    &[
        (CARDIOLOGY, 8),
        (RADIOLOGY_CALL, 8),
        (ANAESTHETICS, 7),
        (SURGERY_NURSE, 6),
        (SURGERY_DOCTOR, 6),
        (NEUROLOGY_DOCTOR, 6),
        (RADIOLOGY_DAY, 5),
        (RADIOLOGY_NURSE, 5),
        (OUTPATIENT_DOCTOR, 5),
        (OUTPATIENT_NURSE, 5),
        (AMBULATORY_DOCTOR, 4),
        (AMBULATORY_NURSE, 4),
        (PEDIATRIC_DOCTOR, 4),
        (PEDIATRIC_NURSE, 4),
        (NEUROLOGY_NURSE, 4),
        (CRITICAL_DOCTOR, 3),
        (CRITICAL_NURSE, 3),
    ]
}

/// Reads the current cohort count for one specific skill.
fn line_load_for_skill(load: &CohortLoad, skill: &'static str) -> usize {
    match skill {
        AMBULATORY_DOCTOR => load.ambulatory_doctors,
        AMBULATORY_NURSE => load.ambulatory_nurses,
        NEUROLOGY_DOCTOR => load.neurology_doctors,
        NEUROLOGY_NURSE => load.neurology_nurses,
        CRITICAL_DOCTOR => load.critical_doctors,
        CRITICAL_NURSE => load.critical_nurses,
        PEDIATRIC_DOCTOR => load.pediatric_doctors,
        PEDIATRIC_NURSE => load.pediatric_nurses,
        SURGERY_DOCTOR => load.surgery_doctors,
        SURGERY_NURSE => load.surgery_nurses,
        OUTPATIENT_DOCTOR => load.outpatient_doctors,
        OUTPATIENT_NURSE => load.outpatient_nurses,
        RADIOLOGY_DAY => load.radiology_day,
        RADIOLOGY_NURSE => load.radiology_nurses,
        RADIOLOGY_CALL => load.radiology_call,
        CARDIOLOGY => load.cardiology,
        ANAESTHETICS => load.anaesthetics,
        _ => 0,
    }
}
