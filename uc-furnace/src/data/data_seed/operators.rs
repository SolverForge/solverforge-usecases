use crate::domain::{base_role_skill_mask, Operator, OperatorRole, OperatorSkill};

use super::catalog::DemoDataProfile;

pub(super) fn build_operators(profile: DemoDataProfile) -> Vec<Operator> {
    let mut operators = base_operators();
    append_reserve_operators(&mut operators, profile);
    operators
}

fn base_operators() -> Vec<Operator> {
    vec![
        operator(0, "Marco Bellini", OperatorRole::ShiftLead, false, &[]),
        operator(1, "Fabio Rinaldi", OperatorRole::ShiftLead, false, &[]),
        operator(
            2,
            "Davide Greco",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(3, "Luca Mariani", OperatorRole::FurnaceOperator, false, &[]),
        operator(
            4,
            "Stefano Moretti",
            OperatorRole::FurnaceOperator,
            false,
            &[],
        ),
        operator(
            5,
            "Paolo Conti",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            6,
            "Andrea Bruno",
            OperatorRole::MaintenanceTechnician,
            false,
            &[],
        ),
        operator(
            7,
            "Giorgio Esposito",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::LoadBuild],
        ),
        operator(
            8,
            "Matteo Ferri",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(9, "Simone Neri", OperatorRole::MaterialHandler, true, &[]),
        operator(
            10,
            "Riccardo Sala",
            OperatorRole::MaterialHandler,
            true,
            &[OperatorSkill::LoadBuild],
        ),
        operator(
            11,
            "Elena Villa",
            OperatorRole::QualityControl,
            true,
            &[OperatorSkill::ShipStore],
        ),
        operator(
            12,
            "Michele Riva",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(
            13,
            "Giulio De Luca",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            14,
            "Roberto Leone",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::LoadBuild, OperatorSkill::QuenchOperate],
        ),
        operator(15, "Alessio Guerra", OperatorRole::ShiftLead, false, &[]),
        operator(
            16,
            "Tommaso Villa",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(
            17,
            "Sergio Bassi",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(18, "Ivan Serra", OperatorRole::FurnaceOperator, false, &[]),
        operator(
            19,
            "Enrico Nardi",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::LoadBuild],
        ),
        operator(
            20,
            "Dario Bellotti",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(
            21,
            "Gabriele Rossi",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(22, "Nicola Santini", OperatorRole::ShiftLead, false, &[]),
        operator(
            23,
            "Massimo Orsi",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            24,
            "Fabiano Grechi",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::LoadBuild],
        ),
        operator(
            25,
            "Leonardo Piva",
            OperatorRole::MaintenanceTechnician,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(
            26,
            "Federico Galli",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(
            27,
            "Samuele Vannini",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            28,
            "Matteo Alberti",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            29,
            "Claudio Martini",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ),
        operator(30, "Ettore Romano", OperatorRole::ShiftLead, false, &[]),
        operator(
            31,
            "Francesco Serra",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            32,
            "Alberto Costa",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
        operator(
            33,
            "Mauro Benedetti",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ),
    ]
}

fn append_reserve_operators(operators: &mut Vec<Operator>, profile: DemoDataProfile) {
    let next_id = operators.len();
    if profile.shift_lead_reserves >= 1 {
        operators.push(operator(
            next_id,
            "Valerio Fontana",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ));
    }
    if profile.furnace_operator_reserves >= 1 {
        operators.push(operator(
            operators.len(),
            "Marta Fabbri",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ));
    }
    if profile.furnace_operator_reserves >= 2 {
        operators.push(operator(
            operators.len(),
            "Pietro Ricci",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ));
    }
    if profile.shift_lead_reserves >= 2 {
        operators.push(operator(
            operators.len(),
            "Andrea Bianchi",
            OperatorRole::ShiftLead,
            false,
            &[OperatorSkill::CraneForklift],
        ));
    }
    if profile.furnace_operator_reserves >= 3 {
        operators.push(operator(
            operators.len(),
            "Silvia Rota",
            OperatorRole::FurnaceOperator,
            false,
            &[OperatorSkill::FurnaceProgram],
        ));
    }
}

fn operator(
    id: usize,
    name: &'static str,
    role: OperatorRole,
    day_only: bool,
    overrides: &'static [OperatorSkill],
) -> Operator {
    let mut mask = base_role_skill_mask(role);
    for skill in overrides {
        mask = mask.insert(*skill);
    }

    let skills = OperatorSkill::ALL
        .into_iter()
        .filter(|skill| mask.contains(*skill))
        .collect::<Vec<_>>();

    Operator {
        id,
        name,
        role,
        day_only,
        skills: Box::leak(skills.into_boxed_slice()),
        skill_mask: mask,
    }
}
