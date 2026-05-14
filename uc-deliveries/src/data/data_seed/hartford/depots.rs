use super::super::types::{CustomerType, LocationData};

pub(in crate::data::data_seed) const DEPOTS: &[LocationData] = &[
    LocationData {
        name: "Downtown Hartford Depot",
        lat: 41.7658,
        lng: -72.6734,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Asylum Hill Depot",
        lat: 41.7700,
        lng: -72.6900,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "South End Depot",
        lat: 41.7400,
        lng: -72.6750,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "West End Depot",
        lat: 41.7680,
        lng: -72.7100,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Barry Square Depot",
        lat: 41.7450,
        lng: -72.6800,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Clay Arsenal Depot",
        lat: 41.7750,
        lng: -72.6850,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Science Center Depot",
        lat: 41.7650,
        lng: -72.6695,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Frog Hollow Depot",
        lat: 41.7580,
        lng: -72.6900,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Blue Hills Depot",
        lat: 41.7850,
        lng: -72.7050,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Charter Oak Depot",
        lat: 41.7495,
        lng: -72.6650,
        customer_type: CustomerType::Business,
    },
];
