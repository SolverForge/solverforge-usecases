use super::enums::HeatTreatmentProcess;
use super::furnace::FurnaceAssignment;

pub fn changeover_minutes(from_temp: u32, to_temp: u32) -> u32 {
    if from_temp == to_temp {
        return 0;
    }

    let temperature_delta = from_temp.abs_diff(to_temp);
    let transition = if to_temp > from_temp {
        temperature_delta.div_ceil(9)
    } else {
        temperature_delta.div_ceil(4)
    };

    transition + 20
}

pub fn calculate_changeover_cost_for_sequence(
    left: &FurnaceAssignment,
    right: &FurnaceAssignment,
) -> u32 {
    calculate_changeover_cost(
        left.temperature_celsius,
        left.process,
        right.temperature_celsius,
        right.process,
    )
}

pub fn calculate_changeover_cost(
    left_temperature_celsius: u32,
    left_process: HeatTreatmentProcess,
    right_temperature_celsius: u32,
    right_process: HeatTreatmentProcess,
) -> u32 {
    let base = changeover_minutes(left_temperature_celsius, right_temperature_celsius);
    let family_penalty = process_family_penalty(left_process, right_process);
    base + family_penalty
}

fn process_family_penalty(left: HeatTreatmentProcess, right: HeatTreatmentProcess) -> u32 {
    match (left, right) {
        (HeatTreatmentProcess::Carburizing, HeatTreatmentProcess::Quenching)
        | (HeatTreatmentProcess::Quenching, HeatTreatmentProcess::Carburizing) => 35,
        (HeatTreatmentProcess::Quenching, HeatTreatmentProcess::Tempering)
        | (HeatTreatmentProcess::Tempering, HeatTreatmentProcess::Quenching) => 15,
        (HeatTreatmentProcess::SolutionTreating, HeatTreatmentProcess::Aging)
        | (HeatTreatmentProcess::Aging, HeatTreatmentProcess::SolutionTreating) => 10,
        (left, right) if left == right => 0,
        _ => 20,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FurnaceAssignment, PriorityBand};

    #[test]
    fn same_temperature_is_zero() {
        assert_eq!(changeover_minutes(850, 850), 0);
    }

    #[test]
    fn heating_is_faster_than_cooling() {
        let heat = changeover_minutes(200, 900);
        let cool = changeover_minutes(900, 200);
        assert!(heat < cool, "heating {heat} should be < cooling {cool}");
    }

    #[test]
    fn family_penalty_changes_sequence_cost() {
        let left = FurnaceAssignment {
            work_order_idx: 0,
            process: HeatTreatmentProcess::Quenching,
            temperature_celsius: 850,
            soak_time_minutes: 90,
            load_weight_kg: 500,
            due_datetime_minutes: 0,
            priority: PriorityBand::Standard,
            requires_quench: true,
            compatible_assignments: vec![0],
            assignment: Some(0),
            eligible_load_build_operator_ids: vec![0],
            eligible_program_operator_ids: vec![0],
            eligible_quench_operator_ids: vec![0],
            eligible_unload_operator_ids: vec![0],
            load_build_operator_id: None,
            program_operator_id: None,
            quench_operator_id: None,
            unload_operator_id: None,
        };
        let right = FurnaceAssignment {
            work_order_idx: 1,
            process: HeatTreatmentProcess::Tempering,
            temperature_celsius: 210,
            soak_time_minutes: 60,
            load_weight_kg: 400,
            due_datetime_minutes: 0,
            priority: PriorityBand::Standard,
            requires_quench: false,
            compatible_assignments: vec![1],
            assignment: Some(1),
            eligible_load_build_operator_ids: vec![0],
            eligible_program_operator_ids: vec![0],
            eligible_quench_operator_ids: vec![0],
            eligible_unload_operator_ids: vec![0],
            load_build_operator_id: None,
            program_operator_id: None,
            quench_operator_id: None,
            unload_operator_id: None,
        };

        assert!(calculate_changeover_cost_for_sequence(&left, &right) > 0);
    }
}
