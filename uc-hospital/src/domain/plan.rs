//! Domain model for the hospital employee scheduling problem.

use chrono::{NaiveDate, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use solverforge::prelude::*;

use super::{CareHub, Employee};

/// Work item that the solver must assign to exactly one employee or leave open.
///
/// In this example a shift is the only planning entity, which keeps the
/// beginner mental model simple: SolverForge is choosing `employee_idx` values
/// for each `Shift`.
#[planning_entity]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shift {
    #[planning_id]
    pub id: String,
    #[serde(skip)]
    pub index: usize,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub location: String,
    #[serde(default)]
    pub care_hub: CareHub,
    pub required_skill: String,
    #[serde(skip)]
    pub touched_dates: Vec<NaiveDate>,
    // SolverForge mutates this scalar slot. The value is an index into
    // `Plan.employees`; `Employee.id` remains transport identity for API/UI use.
    #[planning_variable(
        value_range_provider = "employees",
        allows_unassigned = true,
        candidate_values = "shift_employee_candidates",
        nearby_value_candidates = "shift_nearby_employee_candidates",
        nearby_entity_candidates = "shift_nearby_shift_candidates",
        nearby_value_distance_meter = "shift_to_employee_nearby_distance",
        nearby_entity_distance_meter = "shift_to_shift_nearby_distance"
    )]
    pub employee_idx: Option<usize>,
}

impl Shift {
    /// Creates a new unassigned shift and derives its first-pass care hub.
    pub fn new(
        id: impl Into<String>,
        start: NaiveDateTime,
        end: NaiveDateTime,
        location: impl Into<String>,
        required_skill: impl Into<String>,
    ) -> Self {
        let location = location.into();
        Self {
            id: id.into(),
            index: 0,
            start,
            end,
            care_hub: CareHub::from_location(&location),
            location,
            required_skill: required_skill.into(),
            touched_dates: Vec::new(),
            employee_idx: None,
        }
    }

    /// Returns every calendar day touched by the shift, including overnight end days.
    pub fn touched_dates(&self) -> &[NaiveDate] {
        self.touched_dates.as_slice()
    }

    /// Convenience helper used by tests and data exploration.
    pub fn duration_hours(&self) -> f64 {
        (self.end - self.start).num_minutes() as f64 / 60.0
    }
}

/// Full planning solution published to the solver runtime and the HTTP API.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    #[problem_fact_collection]
    pub employees: Vec<Employee>,
    #[planning_entity_collection]
    pub shifts: Vec<Shift>,
    #[planning_score]
    pub score: Option<HardSoftDecimalScore>,
    #[serde(skip)]
    employee_indices: Vec<usize>,
    #[serde(skip)]
    shift_indices: Vec<usize>,
}

impl Plan {
    /// Builds a plan and immediately restores all derived runtime helpers.
    pub fn new(employees: Vec<Employee>, shifts: Vec<Shift>) -> Self {
        let mut schedule = Self {
            employees,
            shifts,
            score: None,
            employee_indices: Vec::new(),
            shift_indices: Vec::new(),
        };
        schedule.rebuild_derived_fields();
        schedule
    }

    /// Recomputes indexes, inferred hubs, touched dates, and range-safe assignments.
    ///
    /// This runs after generation and after transport decoding so the domain
    /// model always reaches the solver in a normalized state.
    pub fn rebuild_derived_fields(&mut self) {
        for (index, employee) in self.employees.iter_mut().enumerate() {
            employee.index = index;
            employee.finalize();
        }

        for (index, shift) in self.shifts.iter_mut().enumerate() {
            shift.index = index;
            if shift.care_hub == CareHub::Unknown {
                shift.care_hub = CareHub::from_location(&shift.location);
            }
            shift.touched_dates = dates_touched_by_span(shift.start, shift.end);
            shift.employee_idx = shift
                .employee_idx
                .filter(|employee_idx| *employee_idx < self.employees.len());
        }

        self.employee_indices = (0..self.employees.len()).collect();
        self.shift_indices = (0..self.shifts.len()).collect();
    }

    /// Converts the domain model into a flat JSON-object field map for transport DTOs.
    pub fn to_transport_fields(&self) -> Map<String, Value> {
        match serde_json::to_value(self).expect("failed to serialize employee schedule") {
            Value::Object(fields) => fields,
            _ => Map::new(),
        }
    }

    /// Rebuilds a domain plan from the transport field map used by `PlanDto`.
    pub fn from_transport_fields(fields: Map<String, Value>) -> Result<Self, serde_json::Error> {
        let mut schedule: Self = serde_json::from_value(Value::Object(fields))?;
        schedule.rebuild_derived_fields();
        Ok(schedule)
    }

    /// Safe index lookup used by nearby meters and constraint helpers.
    #[inline]
    pub fn get_employee(&self, idx: usize) -> Option<&Employee> {
        self.employees.get(idx)
    }

    /// Convenience accessor used by tests and diagnostics.
    #[inline]
    pub fn employee_count(&self) -> usize {
        self.employees.len()
    }

    /// Named slice accessor used by joins and generated transport code.
    #[inline]
    pub fn employees_slice(&self) -> &[Employee] {
        self.employees.as_slice()
    }
}

// Scalar candidate hooks return borrowed index slices so move generation can
// stay allocation-free while the detailed distance meters rank the choices.
pub(super) fn shift_employee_candidates(
    solution: &Plan,
    _entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    solution.employee_indices.as_slice()
}

pub(super) fn shift_nearby_employee_candidates(
    solution: &Plan,
    entity_index: usize,
    variable_index: usize,
) -> &[usize] {
    shift_employee_candidates(solution, entity_index, variable_index)
}

pub(super) fn shift_nearby_shift_candidates(
    solution: &Plan,
    _entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    solution.shift_indices.as_slice()
}

// This nearby meter is deliberately cheap. It is not a feasibility oracle; it
// just nudges the selector toward promising employees before the real
// constraints do the exact scoring work.
pub(super) fn shift_to_employee_nearby_distance(
    solution: &Plan,
    shift: &Shift,
    employee_index: usize,
) -> f64 {
    let Some(employee) = solution.get_employee(employee_index) else {
        return f64::INFINITY;
    };

    // Nearby meters run during move generation, so keep this intentionally
    // cheap and mostly static. Hard feasibility is evaluated by constraints.
    let mut distance = 10.0 * care_hub_distance(shift.care_hub, employee.home_hub);

    if !employee.skills.contains(&shift.required_skill) {
        distance += 10_000.0;
    } else if CareHub::from_skill(&shift.required_skill) != Some(employee.home_hub) {
        distance += 12.0;
    }

    if shift
        .touched_dates()
        .iter()
        .any(|date| employee.unavailable_dates.contains(date))
    {
        distance += 2_000.0;
    }

    distance
}

// Shift-to-shift proximity helps nearby swap selectors stay within roughly
// compatible service lines and time bands.
pub(super) fn shift_to_shift_nearby_distance(_solution: &Plan, left: &Shift, right: &Shift) -> f64 {
    10.0 * care_hub_distance(left.care_hub, right.care_hub)
        + start_band_distance(left.start.time().hour(), right.start.time().hour())
}

/// Places care hubs on a tiny hand-authored grid so Manhattan distance is easy to explain.
fn care_hub_distance(left: CareHub, right: CareHub) -> f64 {
    let (lx, ly) = care_hub_position(left);
    let (rx, ry) = care_hub_position(right);
    ((lx - rx).abs() + (ly - ry).abs()) as f64
}

/// Provides the synthetic coordinates used by `care_hub_distance`.
fn care_hub_position(hub: CareHub) -> (i32, i32) {
    match hub {
        CareHub::Ambulatory => (0, 0),
        CareHub::Outpatient => (1, 0),
        CareHub::PediatricCare => (0, 1),
        CareHub::Neurology => (1, 1),
        CareHub::CriticalCare => (2, 1),
        CareHub::Surgery => (2, 2),
        CareHub::Radiology => (3, 2),
        CareHub::Unknown => (4, 4),
    }
}

/// Groups start times into broad bands so swaps prefer similar shift shapes.
fn start_band_distance(left_hour: u32, right_hour: u32) -> f64 {
    let left_band = start_band_index(left_hour);
    let right_band = start_band_index(right_hour);
    (left_band.abs_diff(right_band).min(2)) as f64
}

/// Maps a wall-clock hour to the coarse start-time band used above.
fn start_band_index(hour: u32) -> u32 {
    match hour {
        0..=7 => 0,
        8..=12 => 1,
        13..=17 => 2,
        _ => 3,
    }
}

/// Expands a shift into the set of calendar dates it touches.
fn dates_touched_by_span(start: NaiveDateTime, end: NaiveDateTime) -> Vec<NaiveDate> {
    let mut touched_dates = Vec::new();
    let mut date = start.date();

    while date <= end.date() {
        if overlap_minutes_for_day(start, end, date) > 0 {
            touched_dates.push(date);
        }

        let Some(next_date) = date.succ_opt() else {
            break;
        };
        date = next_date;
    }

    touched_dates
}

/// Measures how many minutes of a shift fall inside one specific calendar day.
fn overlap_minutes_for_day(start: NaiveDateTime, end: NaiveDateTime, date: NaiveDate) -> i64 {
    let day_start = date.and_hms_opt(0, 0, 0).unwrap();
    let day_end = date
        .succ_opt()
        .unwrap_or(date)
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let overlap_start = start.max(day_start);
    let overlap_end = end.min(day_end);

    if overlap_start < overlap_end {
        (overlap_end - overlap_start).num_minutes()
    } else {
        0
    }
}

#[cfg(test)]
mod tests;
