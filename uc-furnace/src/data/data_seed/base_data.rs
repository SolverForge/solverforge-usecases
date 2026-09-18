use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::domain::{
    furnace_accepts_work_order, Furnace, FurnaceAssignment, FurnaceType, HeatTreatmentProcess,
    Operator, OperatorRole, OperatorSkill, Shift, ShiftCoverageDemand, ShiftType, SkillMask,
    WorkOrder, CARRY_OUT_END_MINUTE, MINUTES_PER_DAY, NUM_DAYS, NUM_TIME_SLOTS,
    STAFFING_ROSTER_DAY_END, STAFFING_ROSTER_DAY_START, TIME_STEP,
};
use HeatTreatmentProcess::*;

use super::catalog::{DemoDataProfile, DemoSeed};
use super::furnace_seed::{
    order_compatible_assignment_ranges, preferred_start_slots, slot_preserves_shift_ownership,
};
use super::operators::build_operators;
use super::orders::build_work_orders;

pub(super) fn build_demo(profile: DemoDataProfile) -> DemoSeed {
    let furnaces = build_furnaces();
    let mut rng = SmallRng::seed_from_u64(profile.seed);
    let work_orders = build_work_orders(&mut rng, profile);
    let operators = build_operators(profile);
    let shifts = build_shifts();
    let shift_coverage_demands = build_shift_coverage_demands(&shifts);
    let mut assignments = build_assignments(&furnaces, &work_orders, &operators);

    order_compatible_assignment_ranges(&mut assignments);

    DemoSeed {
        furnaces,
        work_orders,
        operators,
        shifts,
        shift_coverage_demands,
        assignments,
    }
}

fn build_furnaces() -> Vec<Furnace> {
    vec![
        Furnace {
            id: 0,
            name: "Chamber Furnace 1",
            max_temp_celsius: 1050,
            max_load_kg: 1600,
            furnace_type: FurnaceType::Chamber,
            heating_rate_c_per_minute: 10,
            cooling_rate_c_per_minute: 4,
            supported_processes: &[
                Quenching,
                Tempering,
                Annealing,
                Normalizing,
                StressRelieving,
            ],
        },
        Furnace {
            id: 1,
            name: "Chamber Furnace 2",
            max_temp_celsius: 1050,
            max_load_kg: 1200,
            furnace_type: FurnaceType::Chamber,
            heating_rate_c_per_minute: 9,
            cooling_rate_c_per_minute: 4,
            supported_processes: &[
                Quenching,
                Tempering,
                Annealing,
                Normalizing,
                StressRelieving,
            ],
        },
        Furnace {
            id: 2,
            name: "Chamber Furnace 3",
            max_temp_celsius: 1100,
            max_load_kg: 1800,
            furnace_type: FurnaceType::Chamber,
            heating_rate_c_per_minute: 11,
            cooling_rate_c_per_minute: 5,
            supported_processes: &[
                Quenching,
                Tempering,
                Annealing,
                Normalizing,
                StressRelieving,
            ],
        },
        Furnace {
            id: 3,
            name: "Chamber Furnace 4",
            max_temp_celsius: 1050,
            max_load_kg: 1000,
            furnace_type: FurnaceType::Chamber,
            heating_rate_c_per_minute: 8,
            cooling_rate_c_per_minute: 3,
            supported_processes: &[
                Quenching,
                Tempering,
                Annealing,
                Normalizing,
                StressRelieving,
            ],
        },
        Furnace {
            id: 4,
            name: "Chamber Furnace 5",
            max_temp_celsius: 1050,
            max_load_kg: 900,
            furnace_type: FurnaceType::Chamber,
            heating_rate_c_per_minute: 8,
            cooling_rate_c_per_minute: 3,
            supported_processes: &[
                Quenching,
                Tempering,
                Annealing,
                Normalizing,
                StressRelieving,
            ],
        },
        Furnace {
            id: 5,
            name: "Carburizing Pit 1",
            max_temp_celsius: 950,
            max_load_kg: 3000,
            furnace_type: FurnaceType::Pit,
            heating_rate_c_per_minute: 6,
            cooling_rate_c_per_minute: 2,
            supported_processes: &[Carburizing, Quenching],
        },
        Furnace {
            id: 6,
            name: "Carburizing Pit 2",
            max_temp_celsius: 960,
            max_load_kg: 2600,
            furnace_type: FurnaceType::Pit,
            heating_rate_c_per_minute: 6,
            cooling_rate_c_per_minute: 2,
            supported_processes: &[Carburizing, Quenching],
        },
        Furnace {
            id: 7,
            name: "Nitriding Pit 1",
            max_temp_celsius: 580,
            max_load_kg: 3000,
            furnace_type: FurnaceType::Pit,
            heating_rate_c_per_minute: 5,
            cooling_rate_c_per_minute: 2,
            supported_processes: &[Nitriding],
        },
        Furnace {
            id: 8,
            name: "Nitriding Pit 2",
            max_temp_celsius: 580,
            max_load_kg: 2400,
            furnace_type: FurnaceType::Pit,
            heating_rate_c_per_minute: 5,
            cooling_rate_c_per_minute: 2,
            supported_processes: &[Nitriding],
        },
        Furnace {
            id: 9,
            name: "Car Hearth Furnace",
            max_temp_celsius: 1050,
            max_load_kg: 5000,
            furnace_type: FurnaceType::CarHearth,
            heating_rate_c_per_minute: 5,
            cooling_rate_c_per_minute: 2,
            supported_processes: &[
                Quenching,
                Tempering,
                Annealing,
                Normalizing,
                StressRelieving,
            ],
        },
        Furnace {
            id: 10,
            name: "Aluminum Furnace",
            max_temp_celsius: 600,
            max_load_kg: 1000,
            furnace_type: FurnaceType::Aluminum,
            heating_rate_c_per_minute: 10,
            cooling_rate_c_per_minute: 5,
            supported_processes: &[SolutionTreating, Aging],
        },
    ]
}

fn build_shifts() -> Vec<Shift> {
    let mut shifts = Vec::with_capacity(22);
    shifts.push(Shift {
        id: shifts.len(),
        roster_day: STAFFING_ROSTER_DAY_START,
        shift_type: ShiftType::Night,
        start_minute: 0,
        end_minute: 360,
    });

    for roster_day in 0..NUM_DAYS as i32 {
        let day_offset = roster_day as usize * MINUTES_PER_DAY;

        shifts.push(Shift {
            id: shifts.len(),
            roster_day,
            shift_type: ShiftType::Morning,
            start_minute: day_offset + 360,
            end_minute: day_offset + 840,
        });

        shifts.push(Shift {
            id: shifts.len(),
            roster_day,
            shift_type: ShiftType::Afternoon,
            start_minute: day_offset + 840,
            end_minute: day_offset + 1320,
        });

        shifts.push(Shift {
            id: shifts.len(),
            roster_day,
            shift_type: ShiftType::Night,
            start_minute: day_offset + 1320,
            end_minute: if roster_day == STAFFING_ROSTER_DAY_END {
                CARRY_OUT_END_MINUTE
            } else {
                day_offset + MINUTES_PER_DAY + 360
            },
        });
    }

    shifts
}

fn build_shift_coverage_demands(shifts: &[Shift]) -> Vec<ShiftCoverageDemand> {
    let mut demands = Vec::new();
    for shift in shifts {
        demands.push(ShiftCoverageDemand {
            id: demands.len(),
            shift_id: shift.id,
            roster_day: shift.roster_day,
            shift_type: shift.shift_type,
            role: OperatorRole::ShiftLead,
            required_count: 1,
        });
        demands.push(ShiftCoverageDemand {
            id: demands.len(),
            shift_id: shift.id,
            roster_day: shift.roster_day,
            shift_type: shift.shift_type,
            role: OperatorRole::FurnaceOperator,
            required_count: if shift.shift_type == ShiftType::Night {
                1
            } else {
                2
            },
        });
    }
    demands
}

fn build_assignments(
    furnaces: &[Furnace],
    work_orders: &[WorkOrder],
    operators: &[Operator],
) -> Vec<FurnaceAssignment> {
    let preferred_slots = preferred_start_slots();

    work_orders
        .iter()
        .enumerate()
        .map(|(work_order_idx, work_order)| {
            let load_build_skills = SkillMask::empty().insert(OperatorSkill::LoadBuild);
            let program_skills = SkillMask::empty().insert(OperatorSkill::FurnaceProgram);
            let quench_skills = SkillMask::empty().insert(OperatorSkill::QuenchOperate);
            let unload_skills = if work_order.load_weight_kg > 500 {
                SkillMask::empty()
                    .insert(OperatorSkill::LoadBuild)
                    .insert(OperatorSkill::CraneForklift)
            } else {
                SkillMask::empty().insert(OperatorSkill::LoadBuild)
            };

            FurnaceAssignment {
                compatible_assignments: furnaces
                    .iter()
                    .enumerate()
                    .filter(|(_, furnace)| furnace_accepts_work_order(furnace, work_order))
                    .flat_map(|(furnace_idx, _)| {
                        let base = furnace_idx * NUM_TIME_SLOTS;
                        preferred_slots
                            .iter()
                            .copied()
                            .filter(move |slot| {
                                let start = slot * TIME_STEP;
                                let end = start + work_order.soak_time_minutes as usize;
                                end <= NUM_DAYS * MINUTES_PER_DAY
                                    && slot_preserves_shift_ownership(
                                        start,
                                        work_order.soak_time_minutes as usize,
                                        work_order.requires_quench,
                                    )
                            })
                            .map(move |slot| base + slot)
                    })
                    .collect(),
                work_order_idx,
                process: work_order.process,
                temperature_celsius: work_order.temperature_celsius,
                soak_time_minutes: work_order.soak_time_minutes,
                load_weight_kg: work_order.load_weight_kg,
                due_datetime_minutes: work_order.due_datetime_minutes,
                priority: work_order.priority,
                requires_quench: work_order.requires_quench,
                assignment: None,
                eligible_load_build_operator_ids: eligible_operator_ids(
                    operators,
                    load_build_skills,
                ),
                eligible_program_operator_ids: eligible_operator_ids(operators, program_skills),
                eligible_quench_operator_ids: if work_order.requires_quench {
                    eligible_operator_ids(operators, quench_skills)
                } else {
                    Vec::new()
                },
                eligible_unload_operator_ids: eligible_operator_ids(operators, unload_skills),
                load_build_operator_id: None,
                program_operator_id: None,
                quench_operator_id: None,
                unload_operator_id: None,
            }
        })
        .collect()
}

fn eligible_operator_ids(operators: &[Operator], required_skills: SkillMask) -> Vec<usize> {
    operators
        .iter()
        .filter(|operator| operator.skill_mask.contains_all(required_skills))
        .map(|operator| operator.id)
        .collect()
}
