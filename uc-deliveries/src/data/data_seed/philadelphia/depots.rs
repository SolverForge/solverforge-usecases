use super::super::types::{CustomerType, LocationData};

pub(in crate::data::data_seed) const DEPOTS: &[LocationData] = &[
    LocationData {
        name: "Central Depot - City Hall",
        lat: 39.9526,
        lng: -75.1652,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "South Philly Depot",
        lat: 39.9256,
        lng: -75.1697,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "University City Depot",
        lat: 39.9522,
        lng: -75.1932,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "North Philly Depot",
        lat: 39.9907,
        lng: -75.1556,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Fishtown Depot",
        lat: 39.9712,
        lng: -75.1340,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "West Philly Depot",
        lat: 39.9601,
        lng: -75.2175,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Logan Square Depot",
        lat: 39.9567,
        lng: -75.1720,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Pennsport Depot",
        lat: 39.9320,
        lng: -75.1450,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Kensington Depot",
        lat: 39.9850,
        lng: -75.1280,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Spruce Hill Depot",
        lat: 39.9530,
        lng: -75.2100,
        customer_type: CustomerType::Business,
    },
];
