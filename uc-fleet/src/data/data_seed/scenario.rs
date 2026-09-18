use super::{builders, catalog};
use crate::domain::{
    InspectionAssignment, InspectionRequirement, Plan, TrainingAssignment, TrainingRequirement,
    Vessel, WorkPackage,
};

pub fn baseline_plan() -> Plan {
    let vessels = catalog::vessels();
    let work_packages = catalog::work_packages();
    let inspection_requirements = inspection_requirements(&vessels, &work_packages);
    let mut inspection_assignments =
        inspection_assignments(&inspection_requirements, &work_packages);
    clamp_inspection_days(&mut inspection_assignments);
    let training_requirements = training_requirements(&vessels, &inspection_assignments);
    let training_assignments =
        training_assignments(&training_requirements, &inspection_assignments);

    let mut plan = Plan::new(
        vessels,
        catalog::docks(),
        builders::days(),
        catalog::technician_pools(),
        catalog::parts(),
        catalog::deliveries(),
        catalog::readiness_policies(),
        inspection_requirements,
        training_requirements,
        work_packages,
        inspection_assignments,
        training_assignments,
    );
    builders::populate_candidate_indices(&mut plan);
    plan
}

fn inspection_requirements(
    vessels: &[Vessel],
    work_packages: &[WorkPackage],
) -> Vec<InspectionRequirement> {
    vessels
        .iter()
        .map(|vessel| {
            let package = package_for_vessel(work_packages, &vessel.id);
            let earliest = package.baseline_start_day + package.duration_days;
            InspectionRequirement::new(
                format!("INSP-{}", vessel.id),
                format!("{} inspection", vessel.name),
                vessel.id.clone(),
                1,
                earliest,
                (earliest + 6).min(56),
                1,
            )
        })
        .collect()
}

fn inspection_assignments(
    requirements: &[InspectionRequirement],
    work_packages: &[WorkPackage],
) -> Vec<InspectionAssignment> {
    requirements
        .iter()
        .map(|requirement| {
            let package = package_for_vessel(work_packages, &requirement.vessel_id);
            builders::baseline_inspection_assignment(
                requirement.id.clone(),
                requirement.vessel_id.clone(),
                package.id.clone(),
                package.baseline_start_day + package.duration_days + 1,
            )
        })
        .collect()
}

fn training_requirements(
    vessels: &[Vessel],
    inspections: &[InspectionAssignment],
) -> Vec<TrainingRequirement> {
    vessels
        .iter()
        .filter(|vessel| {
            matches!(
                vessel.vessel_class.as_str(),
                "patrol_cutter" | "frigate_like"
            )
        })
        .map(|vessel| {
            let inspection = inspection_for_vessel(inspections, &vessel.id);
            TrainingRequirement::new(
                format!("TRN-{}", vessel.id),
                format!("{} recertification", vessel.name),
                vessel.id.clone(),
                1,
                inspection.baseline_day + 1,
                (inspection.baseline_day + 8).min(56),
                1,
            )
        })
        .collect()
}

fn training_assignments(
    requirements: &[TrainingRequirement],
    inspections: &[InspectionAssignment],
) -> Vec<TrainingAssignment> {
    requirements
        .iter()
        .map(|requirement| {
            let inspection = inspection_for_vessel(inspections, &requirement.vessel_id);
            builders::baseline_training_assignment(
                requirement.id.clone(),
                requirement.vessel_id.clone(),
                inspection.id.clone(),
                inspection.baseline_day + 1,
            )
        })
        .collect()
}

fn clamp_inspection_days(assignments: &mut [InspectionAssignment]) {
    for assignment in assignments {
        if assignment.baseline_day > 56 {
            assignment.baseline_day = 56;
        }
    }
}

fn package_for_vessel<'a>(packages: &'a [WorkPackage], vessel_id: &str) -> &'a WorkPackage {
    packages
        .iter()
        .find(|package| package.vessel_id == vessel_id)
        .expect("one package per vessel")
}

fn inspection_for_vessel<'a>(
    assignments: &'a [InspectionAssignment],
    vessel_id: &str,
) -> &'a InspectionAssignment {
    assignments
        .iter()
        .find(|assignment| assignment.vessel_id == vessel_id)
        .expect("inspection assignment")
}
