//! Generator constants and shared medical vocabulary.

pub(super) const DAYS_IN_SCHEDULE: i64 = 28;
pub(super) const EMPLOYEE_COUNT: usize = 50;
pub(super) const EXTRA_UNAVAILABLE_COUNT: usize = 6;
pub(super) const PRIMARY_OFF_COHORT_SIZES: [usize; 7] = [7, 7, 7, 7, 7, 7, 8];
pub(super) const TARGET_DESIRED_DATES: usize = 4;
pub(super) const TARGET_UNDESIRED_DATES: usize = 4;
pub(super) const MAX_DESIRED_DATES: usize = 5;
pub(super) const MAX_UNDESIRED_DATES: usize = 5;
// `None` disables the witness-relative exchange phase entirely. Using an
// explicit option keeps the policy honest; `0` would be a sentinel value with
// different semantics disguised as a normal numeric limit.
pub(super) const EXCHANGE_MARK_LIMIT_PER_EMPLOYEE: Option<usize> = None;

pub(super) const DOCTOR: &str = "Doctor";
pub(super) const NURSE: &str = "Nurse";
pub(super) const AMBULATORY_DOCTOR: &str = "Ambulatory doctor";
pub(super) const AMBULATORY_NURSE: &str = "Ambulatory nurse";
pub(super) const NEUROLOGY_DOCTOR: &str = "Neurology doctor";
pub(super) const NEUROLOGY_NURSE: &str = "Neurology nurse";
pub(super) const CRITICAL_DOCTOR: &str = "Critical care doctor";
pub(super) const CRITICAL_NURSE: &str = "Critical care nurse";
pub(super) const PEDIATRIC_DOCTOR: &str = "Pediatric doctor";
pub(super) const PEDIATRIC_NURSE: &str = "Pediatric nurse";
pub(super) const SURGERY_DOCTOR: &str = "Surgery doctor";
pub(super) const SURGERY_NURSE: &str = "Surgery nurse";
pub(super) const OUTPATIENT_DOCTOR: &str = "Outpatient doctor";
pub(super) const OUTPATIENT_NURSE: &str = "Outpatient nurse";
pub(super) const RADIOLOGY_DAY: &str = "Radiology day";
pub(super) const RADIOLOGY_NURSE: &str = "Radiology nurse";
pub(super) const RADIOLOGY_CALL: &str = "Radiology call";
pub(super) const CARDIOLOGY: &str = "Cardiology";
pub(super) const ANAESTHETICS: &str = "Anaesthetics";

pub(super) const FIRST_NAMES: &[&str] = &[
    "Amy", "Beth", "Carl", "Dan", "Elsa", "Flo", "Gus", "Hugo", "Ivy", "Jay",
];
pub(super) const LAST_NAMES: &[&str] = &[
    "Cole", "Fox", "Green", "Jones", "King", "Li", "Poe", "Rye", "Smith", "Watt",
];
