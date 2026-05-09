use std::sync::OnceLock;

use chrono::NaiveDate;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::domain::Plan;

use super::availability::add_extra_unavailability;
use super::cohorts::assign_primary_off_days;
use super::employees::{build_employee_blueprints, instantiate_employees};
use super::preferences::add_preferences;
use super::shifts::{build_public_shifts, prepare_shifts};
use super::time_utils::find_next_monday;
use super::validation::validate_public_dataset;
use super::witness::build_hidden_witness;

/// Materializes the canonical hospital benchmark dataset.
///
/// We cache the built plan because demo data is immutable and deterministic.
/// Reusing the same constructed instance avoids paying generator cost on every
/// API request while still returning an owned `Plan` to each caller.
pub fn generate_large() -> Plan {
    static SCHEDULE: OnceLock<Plan> = OnceLock::new();
    SCHEDULE.get_or_init(build_large_schedule).clone()
}

/// Builds the single published benchmark instance from scratch.
fn build_large_schedule() -> Plan {
    let mut rng = StdRng::seed_from_u64(0);
    let start_date = find_next_monday(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

    // Workforce blueprints are the stable source of truth for skill mix and
    // cohort identity. We shape off-days at the blueprint level so the later
    // instantiated employees inherit the intended coverage structure.
    let mut blueprints = build_employee_blueprints(&mut rng);
    assign_primary_off_days(&mut blueprints);

    // The public problem is what the solver sees: employees plus currently
    // unassigned shifts. We construct that surface before adding preference
    // pressure so all later shaping is anchored to the real published dataset.
    let mut employees = instantiate_employees(&blueprints, start_date);
    let mut shifts = build_public_shifts(start_date);
    prepare_shifts(&mut shifts);

    // The witness roster is the generator's internal "known feasible" schedule.
    // We never expose it to the solver. We use it only to shape calendars and
    // preferences so the public problem stays feasible while still containing
    // soft-pressure opportunities that construction does not get for free.
    let witness = build_hidden_witness(&employees, &shifts);
    add_extra_unavailability(&mut employees, &shifts, &witness.employee_touched_dates);
    add_preferences(&mut employees, start_date, &blueprints, &shifts, &witness);

    // Validation is the last step on purpose: it checks the exact public dataset
    // that the API will serve rather than an earlier intermediate state.
    validate_public_dataset(&employees, &shifts);

    Plan::new(employees, shifts)
}
