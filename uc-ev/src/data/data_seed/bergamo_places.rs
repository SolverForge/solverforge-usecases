//! Real places around Bergamo, Italy.
//!
//! Coordinates and names come from OpenStreetMap (© OpenStreetMap
//! contributors, ODbL 1.0): pharmacies tagged `amenity=pharmacy` and car
//! charging stations tagged `amenity=charging_station`, picked so they spread
//! across the city, the Brembana and Seriana valleys, and the lake shore.

/// Pharmacies served from the hub: `(name, lat, lng)` from OpenStreetMap.
pub(super) const PHARMACIES: &[(&str, f64, f64)] = &[
    ("Farmacia Sant'Alessandro", 45.909491, 10.037705),
    ("Farmacia Dr.ssa Di Vaira Antonella", 45.61619, 9.463131),
    ("Farmacia di Barzio", 45.94561, 9.468295),
    ("farmacia", 45.609887, 9.892357),
    (
        "Farmacia Dott. Castelli Marco R., Selvino",
        45.783746,
        9.756784,
    ),
    ("Farmacia Rivoltella sas", 45.77015, 9.454412),
    ("Farmacia Gualteri S.N.C.", 45.640149, 9.684605),
    (
        "Farmacia Dott. Glauco Guatteri, Tavernola Bergamasca",
        45.708596,
        10.046541,
    ),
    ("Farmacia di Piazza Brembana", 45.94863, 9.673895),
    ("Farmacia Ponte Nossa S.R.L.", 45.863196, 9.882554),
    ("Farmacia Zizza, Almè", 45.742081, 9.60964),
    ("Farmacia Pharma Salute", 45.705605, 9.874394),
    ("Farmacia Di Sovere, Sovere", 45.819767, 10.024168),
    ("Farmacia Dott.ssa Teresa Artillo", 45.887198, 9.559303),
    ("Farmacia San Pellegrino", 45.838364, 9.664686),
    ("Farmacia Lucini", 45.666543, 9.558509),
    (
        "Farmacia Comunale MRC Albà, Albano Sant'Alessandro",
        45.686619,
        9.768225,
    ),
    ("Farmacia Cuccia", 45.63462, 10.044469),
    ("Farmacia San Marco", 45.600839, 9.594577),
    ("Farmacia Perani", 45.662952, 9.956641),
    ("Farmacia Dottore Casati", 45.68904, 9.467978),
    ("Farmacia San Martino, Bedulita", 45.79141, 9.551153),
    (
        "Farmacia Borgo Santa Caterina, Bergamo",
        45.705476,
        9.684812,
    ),
    ("Farmacia Agostini", 45.797508, 9.838674),
    ("Farmacia Comunale, Bolgare", 45.634642, 9.807422),
    ("Farmacia comunale", 45.713919, 9.958963),
    ("Farmacia Dott. Pedenovi", 45.888399, 9.943398),
    ("Farmacia Dottori Volta", 45.715663, 9.548505),
    ("Farmacia Alzano Sopra, Alzano Lombardo", 45.737443, 9.73475),
    (
        "Farmacia Dott.ssa Paris Daniela, Riva di Solto",
        45.774091,
        10.039799,
    ),
    ("Farmacia degli Spezieri, Lallio", 45.667133, 9.630297),
    ("Farmacia di Concesa, Trezzo sull'Adda", 45.602246, 9.52642),
    ("Farmacia Dott.ssa Sacchi", 45.641318, 9.509695),
    ("Farmacia Losa", 45.732222, 9.498364),
    ("Farmacia Regina Pacis", 45.691363, 9.825473),
    ("Farmacia Centrale, Albino", 45.760376, 9.797803),
    (
        "Farmacia Salem Ghassan, Costa Valle Imagna",
        45.803137,
        9.503736,
    ),
    ("Farmacia Piazzini, Ponte San Pietro", 45.697089, 9.591158),
    ("Farmacia Danesi", 45.658452, 10.004976),
    ("Farmacia Ferrari, Ponteranica", 45.729767, 9.653177),
    ("Farmacia Gaio", 45.651655, 9.847319),
    ("Farmacia Dottor Fenaroli, Capriolo", 45.633855, 9.935716),
    ("Farmacia Ornati, Mariano al Brembo", 45.641505, 9.586358),
    ("Farmacia Dr Max, Seriate", 45.689107, 9.723862),
    ("Farmacia Tribulina, Scanzorosciate", 45.715094, 9.773391),
    ("Farmacia di Carenno SNC", 45.802665, 9.463115),
    ("Farmacia Villa", 45.69376, 9.649507),
    (
        "Farmacia Dottoressa Raffaella Del Ponte, Olmo al Brembo",
        45.969296,
        9.64907,
    ),
];

/// Public car-charging stations: `(operator, lat, lng)` from OpenStreetMap.
pub(super) const CHARGERS: &[(&str, f64, f64)] = &[
    ("a2a - Via dell'industria", 45.667935, 9.66483),
    ("Enel X", 45.970277, 9.959622),
    ("Enel X", 45.657243, 10.048539),
    ("Ressolar", 45.901025, 9.654348),
    ("Charge Point - Pradella Sistemi", 45.810823, 9.868149),
    ("EVBILITY", 45.70109, 9.480299),
    ("Pradella Sistemi srl", 45.866886, 9.996144),
    ("Pradella Sistemi srl", 45.7067, 9.8715),
    ("Enel X", 45.6042, 9.511847),
    ("Repower", 45.748325, 9.75323),
    ("Be Charge", 45.887159, 9.797848),
    ("Enel X", 45.737267, 9.5847),
    ("Enel X", 45.788528, 9.992958),
    ("Enel X Way", 45.661921, 9.78108),
];
