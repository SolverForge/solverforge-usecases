use super::bergamo_catalog::{
    TechnicianSeed, PART_FILTERS, PART_RELAYS, PART_SENSORS, PART_VALVES, SKILL_ELECTRICAL,
    SKILL_ELEVATOR, SKILL_HVAC, SKILL_PLUMBING,
};

pub(super) const TECHNICIANS: &[TechnicianSeed] = &[
    TechnicianSeed {
        id: "tech-ada",
        name: "Ada Romano",
        color: "#2563eb",
        start_location_idx: 0,
        end_location_idx: 0,
        skill_mask: ALL_SKILLS,
        inventory_mask: ALL_PARTS,
        territory: "center",
    },
    TechnicianSeed {
        id: "tech-marco",
        name: "Marco Bianchi",
        color: "#059669",
        start_location_idx: 1,
        end_location_idx: 1,
        skill_mask: ALL_SKILLS,
        inventory_mask: ALL_PARTS,
        territory: "east",
    },
    TechnicianSeed {
        id: "tech-elena",
        name: "Elena Conti",
        color: "#d97706",
        start_location_idx: 0,
        end_location_idx: 0,
        skill_mask: ALL_SKILLS,
        inventory_mask: ALL_PARTS,
        territory: "north",
    },
    TechnicianSeed {
        id: "tech-paolo",
        name: "Paolo Gatti",
        color: "#be123c",
        start_location_idx: 0,
        end_location_idx: 0,
        skill_mask: ALL_SKILLS,
        inventory_mask: ALL_PARTS,
        territory: "west",
    },
    TechnicianSeed {
        id: "tech-sara",
        name: "Sara Ferri",
        color: "#7c3aed",
        start_location_idx: 1,
        end_location_idx: 1,
        skill_mask: ALL_SKILLS,
        inventory_mask: ALL_PARTS,
        territory: "south",
    },
    TechnicianSeed {
        id: "tech-luca",
        name: "Luca Moretti",
        color: "#0f766e",
        start_location_idx: 0,
        end_location_idx: 0,
        skill_mask: ALL_SKILLS,
        inventory_mask: ALL_PARTS,
        territory: "east",
    },
];

const ALL_SKILLS: i64 = SKILL_ELECTRICAL | SKILL_ELEVATOR | SKILL_HVAC | SKILL_PLUMBING;
const ALL_PARTS: i64 = PART_FILTERS | PART_RELAYS | PART_SENSORS | PART_VALVES;
