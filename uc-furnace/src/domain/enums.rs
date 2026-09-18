#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeatTreatmentProcess {
    Quenching,
    Tempering,
    Annealing,
    Carburizing,
    Nitriding,
    StressRelieving,
    Normalizing,
    SolutionTreating,
    Aging,
}

impl std::fmt::Display for HeatTreatmentProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Quenching => "Quenching",
            Self::Tempering => "Tempering",
            Self::Annealing => "Annealing",
            Self::Carburizing => "Carburizing",
            Self::Nitriding => "Nitriding",
            Self::StressRelieving => "Stress Relieving",
            Self::Normalizing => "Normalizing",
            Self::SolutionTreating => "Solution Treatment",
            Self::Aging => "Aging",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FurnaceType {
    Chamber,
    Pit,
    CarHearth,
    Aluminum,
}

impl std::fmt::Display for FurnaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Chamber => "Chamber",
            Self::Pit => "Pit",
            Self::CarHearth => "Car Hearth",
            Self::Aluminum => "Aluminum",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorRole {
    ShiftLead,
    FurnaceOperator,
    MaintenanceTechnician,
    MaterialHandler,
    QualityControl,
}

impl OperatorRole {
    pub const ALL: [OperatorRole; 5] = [
        OperatorRole::ShiftLead,
        OperatorRole::FurnaceOperator,
        OperatorRole::MaintenanceTechnician,
        OperatorRole::MaterialHandler,
        OperatorRole::QualityControl,
    ];
}

impl std::fmt::Display for OperatorRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::ShiftLead => "Shift Lead",
            Self::FurnaceOperator => "Furnace Operator",
            Self::MaintenanceTechnician => "Maintenance Technician",
            Self::MaterialHandler => "Material Handler",
            Self::QualityControl => "Quality Control",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorSkill {
    FurnaceProgram,
    LoadBuild,
    CraneForklift,
    QuenchOperate,
    Monitor,
    Electrical,
    Mechanical,
    QcInspect,
    ReceiveStage,
    ShipStore,
}

impl OperatorSkill {
    pub const ALL: [OperatorSkill; 10] = [
        OperatorSkill::FurnaceProgram,
        OperatorSkill::LoadBuild,
        OperatorSkill::CraneForklift,
        OperatorSkill::QuenchOperate,
        OperatorSkill::Monitor,
        OperatorSkill::Electrical,
        OperatorSkill::Mechanical,
        OperatorSkill::QcInspect,
        OperatorSkill::ReceiveStage,
        OperatorSkill::ShipStore,
    ];

    pub const fn bit(self) -> u16 {
        match self {
            Self::FurnaceProgram => 1 << 0,
            Self::LoadBuild => 1 << 1,
            Self::CraneForklift => 1 << 2,
            Self::QuenchOperate => 1 << 3,
            Self::Monitor => 1 << 4,
            Self::Electrical => 1 << 5,
            Self::Mechanical => 1 << 6,
            Self::QcInspect => 1 << 7,
            Self::ReceiveStage => 1 << 8,
            Self::ShipStore => 1 << 9,
        }
    }
}

impl std::fmt::Display for OperatorSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::FurnaceProgram => "FURNACE_PROGRAM",
            Self::LoadBuild => "LOAD_BUILD",
            Self::CraneForklift => "CRANE_FORKLIFT",
            Self::QuenchOperate => "QUENCH_OPERATE",
            Self::Monitor => "MONITOR",
            Self::Electrical => "ELECTRICAL",
            Self::Mechanical => "MECHANICAL",
            Self::QcInspect => "QC_INSPECT",
            Self::ReceiveStage => "RECEIVE_STAGE",
            Self::ShipStore => "SHIP_STORE",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShiftType {
    Morning,
    Afternoon,
    Night,
}

impl ShiftType {
    pub const fn index(self) -> usize {
        match self {
            Self::Morning => 0,
            Self::Afternoon => 1,
            Self::Night => 2,
        }
    }
}

impl std::fmt::Display for ShiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Morning => "Morning",
            Self::Afternoon => "Afternoon",
            Self::Night => "Night",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriorityBand {
    Standard,
    Urgent,
    Express,
}

impl PriorityBand {
    pub const fn weight(self) -> i64 {
        match self {
            Self::Standard => 1,
            Self::Urgent => 4,
            Self::Express => 9,
        }
    }

    pub const fn as_u32(self) -> u32 {
        match self {
            Self::Standard => 1,
            Self::Urgent => 2,
            Self::Express => 3,
        }
    }

    pub const fn from_u32(value: u32) -> Self {
        match value {
            3 => Self::Express,
            2 => Self::Urgent,
            _ => Self::Standard,
        }
    }
}

impl std::fmt::Display for PriorityBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Standard => "Standard",
            Self::Urgent => "Urgent",
            Self::Express => "Express",
        };
        write!(f, "{label}")
    }
}
