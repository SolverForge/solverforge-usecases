use super::builders::{vessel, work_package, VesselSpec, WorkPackageSpec};
use crate::domain::{Delivery, Dock, Part, ReadinessPolicy, TechnicianPool, Vessel, WorkPackage};

const SMALL: &[&str] = &["small", "medium", "large"];
const MEDIUM: &[&str] = &["medium", "large"];

macro_rules! wp {
    (
        $id:expr, $vessel_id:expr, $package_type:expr, $duration:expr, $window:expr,
        $dock_class:expr, $demand:expr, $primary_part:expr, $secondary_part:expr,
        $priority:expr, $defer_allowed:expr, $baseline:expr
    ) => {
        WorkPackageSpec {
            id: $id,
            vessel_id: $vessel_id,
            package_type: $package_type,
            duration: $duration,
            window: $window,
            dock_class: $dock_class,
            demand: $demand,
            primary_part: $primary_part,
            secondary_part: $secondary_part,
            priority: $priority,
            defer_allowed: $defer_allowed,
            baseline: $baseline,
        }
    };
}

#[rustfmt::skip]
const VESSELS: &[VesselSpec] = &[
    VesselSpec { id: "PC-01", name: "Alder", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-02", name: "Brine", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-03", name: "Cinder", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-04", name: "Drift", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-05", name: "Eider", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-06", name: "Firth", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-07", name: "Gannet", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-08", name: "Hawser", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-09", name: "Inlet", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "PC-10", name: "Jasper", class: "patrol_cutter", compatible: SMALL },
    VesselSpec { id: "FG-01", name: "Northlight", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-02", name: "Rook", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-03", name: "Tern", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-04", name: "Vale", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-05", name: "Windward", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-06", name: "Yarrow", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-07", name: "Zephyr", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "FG-08", name: "Beacon", class: "frigate_like", compatible: MEDIUM },
    VesselSpec { id: "SV-01", name: "Harbor", class: "support", compatible: MEDIUM },
    VesselSpec { id: "SV-02", name: "Lantern", class: "support", compatible: MEDIUM },
    VesselSpec { id: "SV-03", name: "Morrow", class: "support", compatible: MEDIUM },
    VesselSpec { id: "SV-04", name: "Oriole", class: "support", compatible: MEDIUM },
    VesselSpec { id: "SV-05", name: "Quay", class: "support", compatible: MEDIUM },
    VesselSpec { id: "SV-06", name: "Sable", class: "support", compatible: MEDIUM },
];

#[rustfmt::skip]
const WORK_PACKAGES: &[WorkPackageSpec] = &[
    wp!("WP-PC-01","PC-01","minor_maintenance",4,(1,14),"small",(1,1,1,0),("FILTER-PACK",1),("",0),80,false,(1,"D3")),
    wp!("WP-PC-02","PC-02","inspection_prep_hull",5,(1,18),"medium",(0,1,2,0),("HULL-COMPOSITE-KIT",1),("",0),75,false,(1,"D2")),
    wp!("WP-PC-03","PC-03","corrective_electrical",3,(8,24),"small",(0,2,0,0),("CABLING-BUNDLE",1),("",0),70,false,(8,"D3")),
    wp!("WP-PC-04","PC-04","routine_service",4,(7,24),"medium",(1,0,1,0),("FILTER-PACK",1),("",0),65,false,(8,"D2")),
    wp!("WP-PC-05","PC-05","post_patrol_hull_repair",4,(1,16),"medium",(1,0,1,0),("HULL-COMPOSITE-KIT",1),("",0),72,false,(2,"D4")),
    wp!("WP-PC-06","PC-06","sensor_electrical_repair",5,(7,26),"medium",(0,1,2,0),("RADAR-COUPLER",1),("",0),74,false,(8,"D4")),
    wp!("WP-PC-07","PC-07","short_cycle_service",3,(14,30),"small",(1,1,0,0),("FILTER-PACK",1),("",0),63,false,(15,"D3")),
    wp!("WP-PC-08","PC-08","stern_ramp_service",4,(20,36),"medium",(1,1,1,0),("CABLING-BUNDLE",1),("",0),69,false,(22,"D2")),
    wp!("WP-PC-09","PC-09","corrective_hull_patch",5,(20,38),"medium",(0,1,2,0),("HULL-COMPOSITE-KIT",1),("",0),68,false,(22,"D4")),
    wp!("WP-PC-10","PC-10","post_patrol_service",4,(37,50),"medium",(1,0,1,0),("FILTER-PACK",1),("",0),62,true,(39,"D2")),
    wp!("WP-FG-01","FG-01","major_maintenance",8,(1,28),"large",(2,1,2,0),("PROP-SEAL-KIT",1),("POWER-MODULE",1),90,false,(8,"D1")),
    wp!("WP-FG-02","FG-02","propulsion_package",7,(1,24),"large",(2,1,1,0),("PUMP-ASSEMBLY",1),("",0),86,false,(1,"D5")),
    wp!("WP-FG-03","FG-03","electrical_inspection_package",6,(12,32),"medium",(0,2,1,1),("RADAR-COUPLER",1),("CABLING-BUNDLE",1),82,false,(14,"D2")),
    wp!("WP-FG-04","FG-04","shaft_seal_replacement",8,(21,42),"large",(2,1,2,0),("PROP-SEAL-KIT",1),("POWER-MODULE",1),88,false,(23,"D1")),
    wp!("WP-FG-05","FG-05","combat_system_power_refresh",8,(16,38),"large",(2,1,2,0),("POWER-MODULE",1),("NAV-IMU",1),87,false,(18,"D5")),
    wp!("WP-FG-06","FG-06","generator_overhaul",8,(26,48),"medium",(2,1,2,0),("GENERATOR-AVR",1),("FILTER-PACK",1),84,false,(28,"D2")),
    wp!("WP-FG-07","FG-07","midlife_hull_availability",9,(31,54),"large",(2,1,2,0),("HULL-COMPOSITE-KIT",1),("VALVE-SET",1),83,false,(33,"D1")),
    wp!("WP-FG-08","FG-08","navigation_power_availability",9,(32,56),"large",(2,2,2,0),("NAV-IMU",1),("GENERATOR-AVR",1),81,false,(34,"D5")),
    wp!("WP-SV-01","SV-01","hull_mechanical",6,(1,21),"large",(0,0,3,0),("VALVE-SET",1),("",0),60,false,(1,"D1")),
    wp!("WP-SV-02","SV-02","routine_service",5,(16,36),"large",(1,0,1,0),("FILTER-PACK",1),("",0),52,false,(17,"D1")),
    wp!("WP-SV-03","SV-03","propulsion_auxiliary_service",7,(8,30),"large",(2,1,1,0),("PUMP-ASSEMBLY",1),("",0),58,false,(9,"D5")),
    wp!("WP-SV-04","SV-04","light_service",4,(26,44),"large",(0,1,1,0),("HVAC-CONTROLLER",1),("",0),35,true,(28,"D5")),
    wp!("WP-SV-05","SV-05","tender_crane_hydraulic_service",6,(42,56),"large",(0,1,3,0),("CRANE-HYDRAULIC-KIT",1),("VALVE-SET",1),55,false,(44,"D1")),
    wp!("WP-SV-06","SV-06","support_generator_service",6,(43,56),"large",(1,1,2,0),("GENERATOR-AVR",1),("FILTER-PACK",1),54,false,(45,"D5")),
];

pub fn vessels() -> Vec<Vessel> {
    VESSELS.iter().map(vessel).collect()
}

pub fn docks() -> Vec<Dock> {
    vec![
        Dock::new("D1", "North Heavy Dock", "large", 1),
        Dock::new("D2", "Central Medium Dock", "medium", 1),
        Dock::new("D3", "South Patrol Dock", "small", 1),
        Dock::new("D4", "East Repair Dock", "medium", 1),
        Dock::new("D5", "West Support Dock", "large", 1),
    ]
}

pub fn technician_pools() -> Vec<TechnicianPool> {
    vec![
        TechnicianPool::new("PROP", "Propulsion Techs", "propulsion", 8, 2),
        TechnicianPool::new("ELEC", "Electrical Techs", "electrical", 7, 2),
        TechnicianPool::new("HULL", "Hull Techs", "hull", 9, 2),
        TechnicianPool::new("QA", "Inspection Team", "inspection", 4, 0),
        TechnicianPool::new("TRNG", "Training Capacity", "training", 4, 0),
    ]
}

pub fn parts() -> Vec<Part> {
    [
        ("PROP-SEAL-KIT", "Propulsion Shaft Seal Kit", 1),
        ("POWER-MODULE", "Ship Service Power Module", 2),
        ("RADAR-COUPLER", "Radar Waveguide Coupler", 2),
        ("PUMP-ASSEMBLY", "Auxiliary Pump Assembly", 2),
        ("VALVE-SET", "Seawater Valve Set", 4),
        ("CABLING-BUNDLE", "Marine Cabling Bundle", 4),
        ("FILTER-PACK", "Fuel And Lube Filter Pack", 10),
        ("HULL-COMPOSITE-KIT", "Hull Repair Composite Kit", 4),
        ("NAV-IMU", "Navigation IMU Module", 1),
        ("HVAC-CONTROLLER", "HVAC Controller", 2),
        ("GENERATOR-AVR", "Generator AVR", 2),
        ("CRANE-HYDRAULIC-KIT", "Tender Crane Hydraulic Kit", 1),
    ]
    .into_iter()
    .map(|(id, name, stock)| Part::new(id, name, stock))
    .collect()
}

#[rustfmt::skip]
pub fn deliveries() -> Vec<Delivery> {
    [
        ("DELIV-01", "Propulsion seal replenishment", "PROP-SEAL-KIT", 4, 19),
        ("DELIV-02", "Power module replenishment", "POWER-MODULE", 3, 15),
        ("DELIV-03", "Radar coupler replenishment", "RADAR-COUPLER", 3, 10),
        ("DELIV-04", "Pump assembly replenishment", "PUMP-ASSEMBLY", 3, 24),
        ("DELIV-05", "Navigation electronics replenishment", "NAV-IMU", 2, 18),
        ("DELIV-06", "Generator control replenishment", "GENERATOR-AVR", 3, 27),
        ("DELIV-07", "Tender hydraulic replenishment", "CRANE-HYDRAULIC-KIT", 2, 31),
        ("DELIV-08", "Hull composite replenishment", "HULL-COMPOSITE-KIT", 5, 12),
    ]
    .into_iter()
    .map(|(id, name, part, quantity, day)| Delivery::new(id, name, part, quantity, day))
    .collect()
}

#[rustfmt::skip]
pub fn readiness_policies() -> Vec<ReadinessPolicy> {
    vec![ReadinessPolicy::new("POLICY-BASELINE", "Baseline readiness", 14, 4, 4)]
}

pub fn work_packages() -> Vec<WorkPackage> {
    WORK_PACKAGES.iter().map(work_package).collect()
}
