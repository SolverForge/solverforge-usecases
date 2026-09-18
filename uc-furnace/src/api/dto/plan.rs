use serde::{Deserialize, Serialize};

use crate::domain::{
    canonical_interval_label, canonical_minute_label, furnace_accepts_work_order, Furnace,
    FurnaceAssignment, FurnaceType, HeatTreatmentProcess, Operator, OperatorRole,
    OperatorShiftAssignment, OperatorSkill, Plan, PriorityBand, Shift, ShiftCoverageDemand,
    ShiftType, SkillMask, WorkOrder,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FurnaceDto {
    pub id: usize,
    pub name: String,
    pub max_temp_celsius: u32,
    pub max_load_kg: u32,
    pub furnace_type: String,
    pub heating_rate_c_per_minute: u32,
    pub cooling_rate_c_per_minute: u32,
    pub supported_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderDto {
    pub id: usize,
    pub order_code: String,
    pub customer: String,
    pub part_description: String,
    pub material: String,
    pub process: String,
    pub temperature_celsius: u32,
    pub soak_time_minutes: u32,
    pub load_weight_kg: u32,
    pub due_datetime_minutes: usize,
    pub due_label: String,
    pub priority: String,
    pub requires_quench: bool,
    pub compatible_furnaces: Vec<CompatibleFurnaceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompatibleFurnaceDto {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentDto {
    pub work_order_id: usize,
    pub order_code: String,
    pub furnace_id: usize,
    pub furnace_name: String,
    pub start_minutes: usize,
    pub end_minutes: usize,
    pub start_label: String,
    pub end_label: String,
    pub window_label: String,
    pub temperature_celsius: u32,
    pub process: String,
    pub customer: String,
    pub priority: String,
    pub late: bool,
    pub late_minutes: usize,
    pub load_weight_kg: u32,
    pub requires_quench: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FurnaceAssignmentDto {
    pub work_order_idx: usize,
    pub process: String,
    pub temperature_celsius: u32,
    pub soak_time_minutes: u32,
    pub load_weight_kg: u32,
    pub due_datetime_minutes: usize,
    pub priority: String,
    pub requires_quench: bool,
    pub compatible_assignments: Vec<usize>,
    pub assignment: Option<usize>,
    pub eligible_load_build_operator_ids: Vec<usize>,
    pub eligible_program_operator_ids: Vec<usize>,
    pub eligible_quench_operator_ids: Vec<usize>,
    pub eligible_unload_operator_ids: Vec<usize>,
    pub load_build_operator_id: Option<usize>,
    pub program_operator_id: Option<usize>,
    pub quench_operator_id: Option<usize>,
    pub unload_operator_id: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperatorDto {
    pub id: usize,
    pub name: String,
    pub role: String,
    pub day_only: bool,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShiftDto {
    pub id: usize,
    pub roster_day: i32,
    pub shift_type: String,
    pub start_minute: usize,
    pub end_minute: usize,
    pub label: String,
    pub time_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShiftCoverageDemandDto {
    pub id: usize,
    pub shift_id: usize,
    pub roster_day: i32,
    pub shift_type: String,
    pub role: String,
    pub required_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperatorShiftAssignmentDto {
    pub id: usize,
    pub operator_id: usize,
    pub operator_name: String,
    pub role: String,
    pub day_only: bool,
    pub roster_day: i32,
    pub allowed_shift_types: Vec<usize>,
    pub shift_type_value: Option<usize>,
    pub shift_id: Option<usize>,
    pub shift_type: Option<String>,
    pub shift_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanDto {
    pub furnaces: Vec<FurnaceDto>,
    pub work_orders: Vec<WorkOrderDto>,
    pub assignments: Vec<AssignmentDto>,
    pub furnace_assignments: Vec<FurnaceAssignmentDto>,
    pub operators: Vec<OperatorDto>,
    pub shifts: Vec<ShiftDto>,
    #[serde(default)]
    pub shift_coverage_demands: Vec<ShiftCoverageDemandDto>,
    pub operator_shift_assignments: Vec<OperatorShiftAssignmentDto>,
}

impl PlanDto {
    pub fn to_plan(&self) -> Result<Plan, String> {
        if self.furnace_assignments.is_empty() {
            return Err("canonical furnaceAssignments are required".to_string());
        }

        let furnaces = self
            .furnaces
            .iter()
            .map(|furnace| {
                Ok(Furnace {
                    id: furnace.id,
                    name: leak_string(&furnace.name),
                    max_temp_celsius: furnace.max_temp_celsius,
                    max_load_kg: furnace.max_load_kg,
                    furnace_type: parse_furnace_type(&furnace.furnace_type)?,
                    heating_rate_c_per_minute: furnace.heating_rate_c_per_minute,
                    cooling_rate_c_per_minute: furnace.cooling_rate_c_per_minute,
                    supported_processes: leak_slice(
                        furnace
                            .supported_processes
                            .iter()
                            .map(|value| parse_process(value))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let work_orders = self
            .work_orders
            .iter()
            .map(|work_order| {
                Ok(WorkOrder {
                    id: work_order.id,
                    order_code: leak_string(&work_order.order_code),
                    customer: leak_string(&work_order.customer),
                    part_description: leak_string(&work_order.part_description),
                    material: leak_string(&work_order.material),
                    process: parse_process(&work_order.process)?,
                    temperature_celsius: work_order.temperature_celsius,
                    soak_time_minutes: work_order.soak_time_minutes,
                    load_weight_kg: work_order.load_weight_kg,
                    due_datetime_minutes: work_order.due_datetime_minutes,
                    priority: parse_priority(&work_order.priority)?,
                    requires_quench: work_order.requires_quench,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let operators = self
            .operators
            .iter()
            .map(|operator| {
                let skills = operator
                    .skills
                    .iter()
                    .map(|value| parse_operator_skill(value))
                    .collect::<Result<Vec<_>, _>>()?;
                let skill_mask = SkillMask::from_slice(&skills);
                Ok(Operator {
                    id: operator.id,
                    name: leak_string(&operator.name),
                    role: parse_operator_role(&operator.role)?,
                    day_only: operator.day_only,
                    skills: leak_slice(skills),
                    skill_mask,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let shifts = self
            .shifts
            .iter()
            .map(|shift| {
                Ok(Shift {
                    id: shift.id,
                    roster_day: shift.roster_day,
                    shift_type: parse_shift_type(&shift.shift_type)?,
                    start_minute: shift.start_minute,
                    end_minute: shift.end_minute,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let assignments = self
            .furnace_assignments
            .iter()
            .map(|assignment| {
                Ok(FurnaceAssignment {
                    work_order_idx: assignment.work_order_idx,
                    process: parse_process(&assignment.process)?,
                    temperature_celsius: assignment.temperature_celsius,
                    soak_time_minutes: assignment.soak_time_minutes,
                    load_weight_kg: assignment.load_weight_kg,
                    due_datetime_minutes: assignment.due_datetime_minutes,
                    priority: parse_priority(&assignment.priority)?,
                    requires_quench: assignment.requires_quench,
                    compatible_assignments: assignment.compatible_assignments.clone(),
                    assignment: assignment.assignment,
                    eligible_load_build_operator_ids: assignment
                        .eligible_load_build_operator_ids
                        .clone(),
                    eligible_program_operator_ids: assignment.eligible_program_operator_ids.clone(),
                    eligible_quench_operator_ids: assignment.eligible_quench_operator_ids.clone(),
                    eligible_unload_operator_ids: assignment.eligible_unload_operator_ids.clone(),
                    load_build_operator_id: assignment.load_build_operator_id,
                    program_operator_id: assignment.program_operator_id,
                    quench_operator_id: assignment.quench_operator_id,
                    unload_operator_id: assignment.unload_operator_id,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let operator_skill_by_id = operators
            .iter()
            .map(|operator| (operator.id, operator.skill_mask))
            .collect::<HashMap<_, _>>();

        let shift_coverage_demands = self
            .shift_coverage_demands
            .iter()
            .map(|demand| {
                Ok(ShiftCoverageDemand {
                    id: demand.id,
                    shift_id: demand.shift_id,
                    roster_day: demand.roster_day,
                    shift_type: parse_shift_type(&demand.shift_type)?,
                    role: parse_operator_role(&demand.role)?,
                    required_count: demand.required_count,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let operator_shift_assignments = self
            .operator_shift_assignments
            .iter()
            .map(|assignment| {
                Ok(OperatorShiftAssignment {
                    id: assignment.id,
                    operator_idx: assignment.operator_id,
                    operator_name: leak_string(&assignment.operator_name),
                    role: parse_operator_role(&assignment.role)?,
                    day_only: assignment.day_only,
                    roster_day: assignment.roster_day,
                    skill_mask: *operator_skill_by_id
                        .get(&assignment.operator_id)
                        .ok_or_else(|| {
                            format!(
                                "operatorShiftAssignments[{}] references unknown operator {}",
                                assignment.id, assignment.operator_id
                            )
                        })?,
                    allowed_shift_types: assignment.allowed_shift_types.clone(),
                    shift_type: assignment.shift_type_value,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        validate_assignment_references(&assignments, &operators, &furnaces, &work_orders)?;
        validate_shift_coverage_demands(&shift_coverage_demands, &shifts)?;
        validate_operator_shift_assignments(&operator_shift_assignments)?;

        Ok(Plan {
            furnaces,
            work_orders,
            operators,
            shifts,
            shift_coverage_demands,
            assignments,
            operator_shift_assignments,
            score: None,
        })
    }

    pub fn from_plan(plan: &Plan) -> Self {
        Self {
            furnaces: plan
                .furnaces
                .iter()
                .map(|furnace| FurnaceDto {
                    id: furnace.id,
                    name: furnace.name.to_string(),
                    max_temp_celsius: furnace.max_temp_celsius,
                    max_load_kg: furnace.max_load_kg,
                    furnace_type: furnace.furnace_type.to_string(),
                    heating_rate_c_per_minute: furnace.heating_rate_c_per_minute,
                    cooling_rate_c_per_minute: furnace.cooling_rate_c_per_minute,
                    supported_processes: furnace
                        .supported_processes
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                })
                .collect(),
            work_orders: plan
                .work_orders
                .iter()
                .map(|work_order| WorkOrderDto {
                    id: work_order.id,
                    order_code: work_order.order_code.to_string(),
                    customer: work_order.customer.to_string(),
                    part_description: work_order.part_description.to_string(),
                    material: work_order.material.to_string(),
                    process: work_order.process.to_string(),
                    temperature_celsius: work_order.temperature_celsius,
                    soak_time_minutes: work_order.soak_time_minutes,
                    load_weight_kg: work_order.load_weight_kg,
                    due_datetime_minutes: work_order.due_datetime_minutes,
                    due_label: canonical_minute_label(work_order.due_datetime_minutes),
                    priority: work_order.priority.to_string(),
                    requires_quench: work_order.requires_quench,
                    compatible_furnaces: plan
                        .furnaces
                        .iter()
                        .filter(|furnace| furnace_accepts_work_order(furnace, work_order))
                        .map(|furnace| CompatibleFurnaceDto {
                            id: furnace.id,
                            name: furnace.name.to_string(),
                        })
                        .collect(),
                })
                .collect(),
            assignments: plan
                .assignments
                .iter()
                .filter(|assignment| assignment.assignment.is_some())
                .map(|assignment| {
                    let work_order = plan.work_order(assignment.work_order_idx);
                    let furnace = plan.furnace(assignment.furnace_idx().unwrap());
                    let start_minutes = assignment.start_minutes().unwrap();
                    let end_minutes = assignment.end_minutes().unwrap();
                    let late = end_minutes > assignment.due_datetime_minutes;
                    let late_minutes = if late {
                        end_minutes - assignment.due_datetime_minutes
                    } else {
                        0
                    };

                    AssignmentDto {
                        work_order_id: work_order.id,
                        order_code: work_order.order_code.to_string(),
                        furnace_id: furnace.id,
                        furnace_name: furnace.name.to_string(),
                        start_minutes,
                        end_minutes,
                        start_label: canonical_minute_label(start_minutes),
                        end_label: canonical_minute_label(end_minutes),
                        window_label: canonical_interval_label(start_minutes, end_minutes),
                        temperature_celsius: assignment.temperature_celsius,
                        process: assignment.process.to_string(),
                        customer: work_order.customer.to_string(),
                        priority: assignment.priority.to_string(),
                        late,
                        late_minutes,
                        load_weight_kg: assignment.load_weight_kg,
                        requires_quench: assignment.requires_quench,
                    }
                })
                .collect(),
            furnace_assignments: plan
                .assignments
                .iter()
                .map(furnace_assignment_to_dto)
                .collect(),
            operators: plan
                .operators
                .iter()
                .map(|operator| OperatorDto {
                    id: operator.id,
                    name: operator.name.to_string(),
                    role: operator.role.to_string(),
                    day_only: operator.day_only,
                    skills: operator.skills.iter().map(ToString::to_string).collect(),
                })
                .collect(),
            shifts: plan
                .shifts
                .iter()
                .map(|shift| ShiftDto {
                    id: shift.id,
                    roster_day: shift.roster_day,
                    shift_type: shift.shift_type.to_string(),
                    start_minute: shift.start_minute,
                    end_minute: shift.end_minute,
                    label: shift.label(),
                    time_label: shift.time_label(),
                })
                .collect(),
            shift_coverage_demands: plan
                .shift_coverage_demands
                .iter()
                .map(|demand| ShiftCoverageDemandDto {
                    id: demand.id,
                    shift_id: demand.shift_id,
                    roster_day: demand.roster_day,
                    shift_type: demand.shift_type.to_string(),
                    role: demand.role.to_string(),
                    required_count: demand.required_count,
                })
                .collect(),
            operator_shift_assignments: plan
                .operator_shift_assignments
                .iter()
                .map(|assignment| operator_shift_assignment_to_dto(plan, assignment))
                .collect(),
        }
    }
}

pub fn solution_to_dto(plan: &Plan) -> PlanDto {
    PlanDto::from_plan(plan)
}

fn operator_shift_assignment_to_dto(
    plan: &Plan,
    assignment: &OperatorShiftAssignment,
) -> OperatorShiftAssignmentDto {
    let shift_id = assignment.shift_id();
    let shift_label = shift_id
        .and_then(|id| plan.shift_by_id(id))
        .map(|shift| shift.label());

    OperatorShiftAssignmentDto {
        id: assignment.id,
        operator_id: assignment.operator_idx,
        operator_name: assignment.operator_name.to_string(),
        role: assignment.role.to_string(),
        day_only: assignment.day_only,
        roster_day: assignment.roster_day,
        allowed_shift_types: assignment.allowed_shift_types.clone(),
        shift_type_value: assignment.shift_type,
        shift_id,
        shift_type: assignment
            .shift_type_enum()
            .map(|shift_type| shift_type.to_string()),
        shift_label,
    }
}

fn furnace_assignment_to_dto(assignment: &FurnaceAssignment) -> FurnaceAssignmentDto {
    FurnaceAssignmentDto {
        work_order_idx: assignment.work_order_idx,
        process: assignment.process.to_string(),
        temperature_celsius: assignment.temperature_celsius,
        soak_time_minutes: assignment.soak_time_minutes,
        load_weight_kg: assignment.load_weight_kg,
        due_datetime_minutes: assignment.due_datetime_minutes,
        priority: assignment.priority.to_string(),
        requires_quench: assignment.requires_quench,
        compatible_assignments: assignment.compatible_assignments.clone(),
        assignment: assignment.assignment,
        eligible_load_build_operator_ids: assignment.eligible_load_build_operator_ids.clone(),
        eligible_program_operator_ids: assignment.eligible_program_operator_ids.clone(),
        eligible_quench_operator_ids: assignment.eligible_quench_operator_ids.clone(),
        eligible_unload_operator_ids: assignment.eligible_unload_operator_ids.clone(),
        load_build_operator_id: assignment.load_build_operator_id,
        program_operator_id: assignment.program_operator_id,
        quench_operator_id: assignment.quench_operator_id,
        unload_operator_id: assignment.unload_operator_id,
    }
}

fn validate_assignment_references(
    assignments: &[FurnaceAssignment],
    operators: &[Operator],
    furnaces: &[Furnace],
    work_orders: &[WorkOrder],
) -> Result<(), String> {
    let operator_ids = operators
        .iter()
        .map(|operator| operator.id)
        .collect::<HashSet<_>>();
    let furnace_by_id = furnaces
        .iter()
        .map(|furnace| (furnace.id, furnace))
        .collect::<HashMap<_, _>>();
    for assignment in assignments {
        let work_order = work_orders.get(assignment.work_order_idx).ok_or_else(|| {
            format!(
                "furnaceAssignments[{}] references unknown work order",
                assignment.work_order_idx
            )
        })?;
        if !furnace_accepts_any_assignment(assignment, furnaces, work_order) {
            return Err(format!(
                "furnaceAssignments[{}] has no valid compatible furnace values",
                assignment.work_order_idx
            ));
        }
        if let Some(value) = assignment.assignment {
            if !assignment.compatible_assignments.contains(&value) {
                return Err(format!(
                    "furnaceAssignments[{}].assignment is outside its value range",
                    assignment.work_order_idx
                ));
            }
            let furnace_idx = value / crate::domain::NUM_TIME_SLOTS;
            let Some(furnace) = furnace_by_id.get(&furnace_idx).copied() else {
                return Err(format!(
                    "furnaceAssignments[{}].assignment references unknown furnace {}",
                    assignment.work_order_idx, furnace_idx
                ));
            };
            if !furnace_accepts_work_order(furnace, work_order) {
                return Err(format!(
                    "furnaceAssignments[{}].assignment references incompatible furnace {}",
                    assignment.work_order_idx, furnace_idx
                ));
            }
        }

        validate_task_operator(
            assignment.work_order_idx,
            "loadBuildOperatorId",
            assignment.load_build_operator_id,
            &assignment.eligible_load_build_operator_ids,
            &operator_ids,
        )?;
        validate_task_operator(
            assignment.work_order_idx,
            "programOperatorId",
            assignment.program_operator_id,
            &assignment.eligible_program_operator_ids,
            &operator_ids,
        )?;
        validate_task_operator(
            assignment.work_order_idx,
            "quenchOperatorId",
            assignment.quench_operator_id,
            &assignment.eligible_quench_operator_ids,
            &operator_ids,
        )?;
        validate_task_operator(
            assignment.work_order_idx,
            "unloadOperatorId",
            assignment.unload_operator_id,
            &assignment.eligible_unload_operator_ids,
            &operator_ids,
        )?;
    }
    Ok(())
}

fn furnace_accepts_any_assignment(
    assignment: &FurnaceAssignment,
    furnaces: &[Furnace],
    work_order: &WorkOrder,
) -> bool {
    assignment.compatible_assignments.iter().any(|value| {
        let furnace_idx = value / crate::domain::NUM_TIME_SLOTS;
        furnaces.iter().any(|furnace| {
            furnace.id == furnace_idx && furnace_accepts_work_order(furnace, work_order)
        })
    })
}

fn validate_shift_coverage_demands(
    demands: &[ShiftCoverageDemand],
    shifts: &[Shift],
) -> Result<(), String> {
    let shift_ids = shifts.iter().map(|shift| shift.id).collect::<HashSet<_>>();
    for demand in demands {
        if !shift_ids.contains(&demand.shift_id) {
            return Err(format!(
                "shiftCoverageDemands[{}] references unknown shift {}",
                demand.id, demand.shift_id
            ));
        }
        if demand.required_count < 0 {
            return Err(format!(
                "shiftCoverageDemands[{}].requiredCount cannot be negative",
                demand.id
            ));
        }
    }
    Ok(())
}

fn validate_operator_shift_assignments(
    assignments: &[OperatorShiftAssignment],
) -> Result<(), String> {
    for assignment in assignments {
        if let Some(shift_type) = assignment.shift_type {
            if !assignment.allowed_shift_types.contains(&shift_type) {
                return Err(format!(
                    "operatorShiftAssignments[{}].shiftTypeValue is outside its value range",
                    assignment.id
                ));
            }
            if assignment.day_only && shift_type == ShiftType::Night.index() {
                return Err(format!(
                    "operatorShiftAssignments[{}].shiftTypeValue assigns night to day-only operator",
                    assignment.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_task_operator(
    work_order_idx: usize,
    field: &str,
    value: Option<usize>,
    range: &[usize],
    operator_ids: &HashSet<usize>,
) -> Result<(), String> {
    let Some(operator_id) = value else {
        return Ok(());
    };
    if !operator_ids.contains(&operator_id) {
        return Err(format!(
            "furnaceAssignments[{work_order_idx}].{field} references unknown operator {operator_id}"
        ));
    }
    if !range.contains(&operator_id) {
        return Err(format!(
            "furnaceAssignments[{work_order_idx}].{field} is outside its value range"
        ));
    }
    Ok(())
}

fn leak_string(value: &str) -> &'static str {
    Box::leak(value.to_string().into_boxed_str())
}

fn leak_slice<T>(values: Vec<T>) -> &'static [T] {
    Box::leak(values.into_boxed_slice())
}

fn parse_process(value: &str) -> Result<HeatTreatmentProcess, String> {
    match value {
        "Quenching" => Ok(HeatTreatmentProcess::Quenching),
        "Tempering" => Ok(HeatTreatmentProcess::Tempering),
        "Annealing" => Ok(HeatTreatmentProcess::Annealing),
        "Carburizing" => Ok(HeatTreatmentProcess::Carburizing),
        "Nitriding" => Ok(HeatTreatmentProcess::Nitriding),
        "Stress Relieving" => Ok(HeatTreatmentProcess::StressRelieving),
        "Normalizing" => Ok(HeatTreatmentProcess::Normalizing),
        "Solution Treatment" => Ok(HeatTreatmentProcess::SolutionTreating),
        "Aging" => Ok(HeatTreatmentProcess::Aging),
        other => Err(format!("unknown heat treatment process {other}")),
    }
}

fn parse_furnace_type(value: &str) -> Result<FurnaceType, String> {
    match value {
        "Chamber" => Ok(FurnaceType::Chamber),
        "Pit" => Ok(FurnaceType::Pit),
        "Car Hearth" => Ok(FurnaceType::CarHearth),
        "Aluminum" => Ok(FurnaceType::Aluminum),
        other => Err(format!("unknown furnace type {other}")),
    }
}

fn parse_operator_role(value: &str) -> Result<OperatorRole, String> {
    match value {
        "Shift Lead" => Ok(OperatorRole::ShiftLead),
        "Furnace Operator" => Ok(OperatorRole::FurnaceOperator),
        "Maintenance Technician" => Ok(OperatorRole::MaintenanceTechnician),
        "Material Handler" => Ok(OperatorRole::MaterialHandler),
        "Quality Control" => Ok(OperatorRole::QualityControl),
        other => Err(format!("unknown operator role {other}")),
    }
}

fn parse_operator_skill(value: &str) -> Result<OperatorSkill, String> {
    match value {
        "FURNACE_PROGRAM" => Ok(OperatorSkill::FurnaceProgram),
        "LOAD_BUILD" => Ok(OperatorSkill::LoadBuild),
        "CRANE_FORKLIFT" => Ok(OperatorSkill::CraneForklift),
        "QUENCH_OPERATE" => Ok(OperatorSkill::QuenchOperate),
        "MONITOR" => Ok(OperatorSkill::Monitor),
        "ELECTRICAL" => Ok(OperatorSkill::Electrical),
        "MECHANICAL" => Ok(OperatorSkill::Mechanical),
        "QC_INSPECT" => Ok(OperatorSkill::QcInspect),
        "RECEIVE_STAGE" => Ok(OperatorSkill::ReceiveStage),
        "SHIP_STORE" => Ok(OperatorSkill::ShipStore),
        other => Err(format!("unknown operator skill {other}")),
    }
}

fn parse_shift_type(value: &str) -> Result<ShiftType, String> {
    match value {
        "Morning" => Ok(ShiftType::Morning),
        "Afternoon" => Ok(ShiftType::Afternoon),
        "Night" => Ok(ShiftType::Night),
        other => Err(format!("unknown shift type {other}")),
    }
}

fn parse_priority(value: &str) -> Result<PriorityBand, String> {
    match value {
        "Standard" => Ok(PriorityBand::Standard),
        "Urgent" => Ok(PriorityBand::Urgent),
        "Express" => Ok(PriorityBand::Express),
        other => Err(format!("unknown priority {other}")),
    }
}
