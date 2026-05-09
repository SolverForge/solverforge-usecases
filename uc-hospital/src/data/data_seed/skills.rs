use super::vocabulary::*;

/// Returns whether the skill is one of the scarcer specialty signals.
pub(super) fn is_specialty_skill(skill: &str) -> bool {
    matches!(
        skill,
        CARDIOLOGY | ANAESTHETICS | RADIOLOGY_DAY | RADIOLOGY_CALL
    )
}

/// Groups service-line skills under the broad "doctor-family" umbrella.
pub(super) fn is_doctor_family_skill(skill: &str) -> bool {
    matches!(
        skill,
        DOCTOR
            | AMBULATORY_DOCTOR
            | NEUROLOGY_DOCTOR
            | CRITICAL_DOCTOR
            | PEDIATRIC_DOCTOR
            | SURGERY_DOCTOR
            | OUTPATIENT_DOCTOR
            | RADIOLOGY_CALL
            | CARDIOLOGY
            | ANAESTHETICS
    )
}

/// Groups service-line skills under the broad "nurse-family" umbrella.
pub(super) fn is_nurse_family_skill(skill: &str) -> bool {
    matches!(
        skill,
        NURSE
            | AMBULATORY_NURSE
            | NEUROLOGY_NURSE
            | CRITICAL_NURSE
            | PEDIATRIC_NURSE
            | SURGERY_NURSE
            | OUTPATIENT_NURSE
            | RADIOLOGY_NURSE
            | RADIOLOGY_DAY
    )
}
