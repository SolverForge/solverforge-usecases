use super::super::types::{CustomerType, LocationData};

pub(in crate::data::data_seed) const DEPOTS: &[LocationData] = &[
    LocationData {
        name: "Centro Storico Depot",
        lat: 43.7696,
        lng: 11.2558,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Santa Maria Novella Depot",
        lat: 43.7745,
        lng: 11.2487,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Campo di Marte Depot",
        lat: 43.7820,
        lng: 11.2820,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Rifredi Depot",
        lat: 43.7950,
        lng: 11.2410,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Novoli Depot",
        lat: 43.7880,
        lng: 11.2220,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Gavinana Depot",
        lat: 43.7520,
        lng: 11.2680,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Mercato Centrale Depot",
        lat: 43.7762,
        lng: 11.2540,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Santa Croce Depot",
        lat: 43.7688,
        lng: 11.2620,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Santo Spirito Depot",
        lat: 43.7665,
        lng: 11.2470,
        customer_type: CustomerType::Business,
    },
    LocationData {
        name: "Careggi Depot",
        lat: 43.8020,
        lng: 11.2530,
        customer_type: CustomerType::Business,
    },
];
