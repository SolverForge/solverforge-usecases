use chrono::{Duration, NaiveDate};
use rand::rngs::StdRng;
use std::collections::BTreeSet;

use crate::domain::{CareHub, Employee};

use super::time_utils::generate_name_permutations;
use super::vocabulary::*;

/// Draft workforce record used before we instantiate full `Employee` facts.
#[derive(Clone)]
pub(super) struct EmployeeBlueprint {
    pub(super) name: String,
    pub(super) skills: BTreeSet<String>,
    pub(super) home_hub: CareHub,
    pub(super) primary_off_weekday: usize,
}

/// Builds the fixed workforce composition for the public demo dataset.
pub(super) fn build_employee_blueprints(rng: &mut StdRng) -> Vec<EmployeeBlueprint> {
    let names = generate_name_permutations(rng);
    let mut skill_sets: Vec<Vec<&'static str>> = Vec::with_capacity(EMPLOYEE_COUNT);

    // The generator used to hand almost every day shift to a generic Doctor or
    // Nurse pool. That made most legal assignments interchangeable and flattened
    // local search almost immediately. The redesign keeps the same workforce
    // size, but assigns each employee to one or two service lines so every
    // shift has a smaller, more meaningful candidate set.
    //
    // We still retain the base DOCTOR/NURSE tags so the witness builder can
    // reason about role families, but public shifts now require service-line
    // skills such as `Critical care doctor` or `Outpatient nurse`.
    push_skill_sets(&mut skill_sets, 4, &[DOCTOR, CRITICAL_DOCTOR]);
    push_skill_sets(
        &mut skill_sets,
        2,
        &[DOCTOR, CRITICAL_DOCTOR, OUTPATIENT_DOCTOR],
    );
    push_skill_sets(&mut skill_sets, 4, &[DOCTOR, NEUROLOGY_DOCTOR, CARDIOLOGY]);
    push_skill_sets(
        &mut skill_sets,
        3,
        &[DOCTOR, AMBULATORY_DOCTOR, PEDIATRIC_DOCTOR],
    );
    push_skill_sets(&mut skill_sets, 4, &[DOCTOR, SURGERY_DOCTOR, ANAESTHETICS]);
    push_skill_sets(
        &mut skill_sets,
        1,
        &[DOCTOR, OUTPATIENT_DOCTOR, AMBULATORY_DOCTOR],
    );
    push_skill_sets(
        &mut skill_sets,
        4,
        &[DOCTOR, RADIOLOGY_CALL, OUTPATIENT_DOCTOR],
    );

    push_skill_sets(&mut skill_sets, 5, &[NURSE, CRITICAL_NURSE]);
    push_skill_sets(
        &mut skill_sets,
        3,
        &[NURSE, CRITICAL_NURSE, OUTPATIENT_NURSE],
    );
    push_skill_sets(
        &mut skill_sets,
        4,
        &[NURSE, AMBULATORY_NURSE, PEDIATRIC_NURSE],
    );
    push_skill_sets(
        &mut skill_sets,
        4,
        &[NURSE, NEUROLOGY_NURSE, PEDIATRIC_NURSE],
    );
    push_skill_sets(
        &mut skill_sets,
        4,
        &[NURSE, SURGERY_NURSE, OUTPATIENT_NURSE],
    );
    push_skill_sets(&mut skill_sets, 4, &[NURSE, RADIOLOGY_DAY, RADIOLOGY_NURSE]);
    push_skill_sets(
        &mut skill_sets,
        2,
        &[NURSE, RADIOLOGY_DAY, RADIOLOGY_NURSE, ANAESTHETICS],
    );
    push_skill_sets(
        &mut skill_sets,
        2,
        &[NURSE, AMBULATORY_NURSE, OUTPATIENT_NURSE],
    );

    assert_eq!(
        skill_sets.len(),
        EMPLOYEE_COUNT,
        "employee blueprint count should match workforce target"
    );

    skill_sets
        .into_iter()
        .enumerate()
        .map(|(index, skills)| {
            let home_hub = CareHub::infer_from_skills(skills.iter().copied());
            EmployeeBlueprint {
                name: names[index].clone(),
                skills: skills.into_iter().map(str::to_string).collect(),
                home_hub,
                primary_off_weekday: 0,
            }
        })
        .collect()
}

/// Appends `count` identical skill bundles to the blueprint list.
fn push_skill_sets(target: &mut Vec<Vec<&'static str>>, count: usize, skills: &[&'static str]) {
    for _ in 0..count {
        target.push(skills.to_vec());
    }
}

impl EmployeeBlueprint {
    /// Tiny convenience helper used by balancing heuristics.
    pub(super) fn has_skill(&self, skill: &'static str) -> bool {
        self.skills.contains(skill)
    }

    /// Counts the specialties that are intentionally scarce in this dataset.
    pub(super) fn specialty_count(&self) -> usize {
        usize::from(self.skills.contains(CARDIOLOGY))
            + usize::from(self.skills.contains(ANAESTHETICS))
            + usize::from(self.skills.contains(RADIOLOGY_CALL))
            + usize::from(self.skills.contains(RADIOLOGY_DAY))
    }
}

/// Turns the blueprints into the actual `Employee` facts published by the app.
pub(super) fn instantiate_employees(
    blueprints: &[EmployeeBlueprint],
    start_date: NaiveDate,
) -> Vec<Employee> {
    let mut employees = Vec::with_capacity(blueprints.len());
    for (index, blueprint) in blueprints.iter().enumerate() {
        let mut employee = Employee::new(index, blueprint.name.clone())
            .with_home_hub(blueprint.home_hub)
            .with_skills(blueprint.skills.iter().map(|skill| skill.as_str()));
        for week in 0..(DAYS_IN_SCHEDULE / 7) {
            let date = start_date + Duration::days(week * 7 + blueprint.primary_off_weekday as i64);
            employee.unavailable_dates.insert(date);
        }
        employees.push(employee);
    }
    employees
}
