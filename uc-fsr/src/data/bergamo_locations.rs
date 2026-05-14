//! Static Bergamo depots and customer sites used by the `STANDARD` dataset.
//!
//! These are input facts, not solver decisions. SolverForge later changes only
//! the visit order inside technician routes.

use super::bergamo_catalog::LocationSeed;

pub(super) const DEPOTS: &[LocationSeed] = &[
    LocationSeed {
        id: "depot-ops",
        label: "Bergamo Operations Hub",
        lat: 45.6954,
        lng: 9.6703,
        territory: "center",
    },
    LocationSeed {
        id: "depot-east",
        label: "Seriate Parts Locker",
        lat: 45.6835,
        lng: 9.7210,
        territory: "east",
    },
];

pub(super) const SERVICE_LOCATIONS: &[LocationSeed] = &[
    LocationSeed {
        id: "loc-citta-alta",
        label: "Citta Alta heating fault",
        lat: 45.7036,
        lng: 9.6627,
        territory: "north",
    },
    LocationSeed {
        id: "loc-borgo-palazzo",
        label: "Borgo Palazzo refrigeration",
        lat: 45.6903,
        lng: 9.6909,
        territory: "east",
    },
    LocationSeed {
        id: "loc-stazione",
        label: "Station kiosk power",
        lat: 45.6900,
        lng: 9.6750,
        territory: "center",
    },
    LocationSeed {
        id: "loc-longuelo",
        label: "Longuelo pump service",
        lat: 45.6982,
        lng: 9.6377,
        territory: "west",
    },
    LocationSeed {
        id: "loc-redona",
        label: "Redona lift inspection",
        lat: 45.7107,
        lng: 9.6999,
        territory: "north",
    },
    LocationSeed {
        id: "loc-celadina",
        label: "Celadina controls alarm",
        lat: 45.6815,
        lng: 9.7056,
        territory: "east",
    },
    LocationSeed {
        id: "loc-valtesse",
        label: "Valtesse boiler reset",
        lat: 45.7202,
        lng: 9.6736,
        territory: "north",
    },
    LocationSeed {
        id: "loc-colognola",
        label: "Colognola valve leak",
        lat: 45.6767,
        lng: 9.6469,
        territory: "south",
    },
    LocationSeed {
        id: "loc-malpensata",
        label: "Malpensata sensor swap",
        lat: 45.6840,
        lng: 9.6687,
        territory: "south",
    },
    LocationSeed {
        id: "loc-seriate",
        label: "Seriate medical cooler",
        lat: 45.6856,
        lng: 9.7242,
        territory: "east",
    },
    LocationSeed {
        id: "loc-gorle",
        label: "Gorle access control",
        lat: 45.7014,
        lng: 9.7138,
        territory: "east",
    },
    LocationSeed {
        id: "loc-treviglio-road",
        label: "Azzano workshop air unit",
        lat: 45.6579,
        lng: 9.6734,
        territory: "south",
    },
    LocationSeed {
        id: "loc-monterosso",
        label: "Monterosso lift callout",
        lat: 45.7161,
        lng: 9.6905,
        territory: "north",
    },
    LocationSeed {
        id: "loc-loreto",
        label: "Loreto electrical board",
        lat: 45.6995,
        lng: 9.6517,
        territory: "west",
    },
    LocationSeed {
        id: "loc-stezzano",
        label: "Stezzano retail HVAC",
        lat: 45.6508,
        lng: 9.6534,
        territory: "south",
    },
    LocationSeed {
        id: "loc-grumello",
        label: "Grumello pressure issue",
        lat: 45.6888,
        lng: 9.6275,
        territory: "west",
    },
    LocationSeed {
        id: "loc-orio",
        label: "Orio terminal chiller",
        lat: 45.6689,
        lng: 9.7044,
        territory: "south",
    },
    LocationSeed {
        id: "loc-ranica",
        label: "Ranica municipal lift",
        lat: 45.7241,
        lng: 9.7133,
        territory: "north",
    },
    LocationSeed {
        id: "loc-torre-boldone",
        label: "Torre Boldone boiler",
        lat: 45.7178,
        lng: 9.7075,
        territory: "north",
    },
    LocationSeed {
        id: "loc-villaggio-sposi",
        label: "Villaggio Sposi pump",
        lat: 45.6901,
        lng: 9.6365,
        territory: "west",
    },
    LocationSeed {
        id: "loc-dalmine",
        label: "Dalmine line sensor",
        lat: 45.6482,
        lng: 9.6061,
        territory: "west",
    },
    LocationSeed {
        id: "loc-alzano",
        label: "Alzano Lombardo relay",
        lat: 45.7362,
        lng: 9.7271,
        territory: "north",
    },
    LocationSeed {
        id: "loc-ponte-san-pietro",
        label: "Ponte San Pietro valve",
        lat: 45.7001,
        lng: 9.5908,
        territory: "west",
    },
    LocationSeed {
        id: "loc-scanzo",
        label: "Scanzorosciate cooler",
        lat: 45.7105,
        lng: 9.7354,
        territory: "east",
    },
];
