//! Deterministic flight network used by the browser and acceptance tests.

use std::str::FromStr;

use crate::domain::{Airport, CrewAssignment, Employee, Flight, Plan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoData {
    Standard,
}

pub fn default_demo_data() -> DemoData {
    DemoData::default_demo_data()
}

pub fn available_demo_data() -> &'static [DemoData] {
    DemoData::available_demo_data()
}

impl DemoData {
    pub fn id(self) -> &'static str {
        "STANDARD"
    }
    pub fn default_demo_data() -> Self {
        Self::Standard
    }
    pub fn available_demo_data() -> &'static [Self] {
        &[Self::Standard]
    }
}

impl FromStr for DemoData {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .eq_ignore_ascii_case("STANDARD")
            .then_some(Self::Standard)
            .ok_or(())
    }
}

pub fn generate(_demo: DemoData) -> Plan {
    let specs = [
        ("LHR", "London Heathrow", 514_775, -4_614),
        ("JFK", "New York JFK", 406_397, -737_789),
        ("CNF", "Belo Horizonte", -196_244, -439_719),
        ("BRU", "Brussels", 509_014, 44_844),
        ("ATL", "Atlanta", 336_367, -844_281),
        ("BNE", "Brisbane", -273_833, 1_531_183),
    ];
    let airports = specs
        .iter()
        .enumerate()
        .map(|(index, (code, name, lat, lon))| {
            let taxi_minutes = (0..specs.len())
                .map(|other| if index == other { 30 } else { 180 })
                .collect();
            Airport::new(*code, *name, *lat, *lon, taxi_minutes)
        })
        .collect::<Vec<_>>();

    let names = [
        "Avery Morgan",
        "Jordan Lee",
        "Riley Patel",
        "Casey Novak",
        "Morgan Diaz",
        "Taylor Kim",
        "Quinn Adams",
        "Cameron Silva",
        "Parker Evans",
        "Reese Brown",
        "Alex Martin",
        "Drew Wilson",
        "Sage Clark",
        "Blake Turner",
        "Rowan Scott",
        "Hayden Green",
        "Skyler Baker",
        "Emerson Hall",
        "Finley Ward",
        "Dakota Reed",
        "Kendall Price",
        "Robin Young",
        "Jamie Bell",
        "Charlie King",
    ];
    let employees = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let skill = if index % 4 < 2 {
                "Pilot"
            } else {
                "Flight attendant"
            };
            let unavailable = if index % 9 == 0 {
                vec![(index % 5) as i64]
            } else {
                vec![]
            };
            Employee::new(
                format!("CREW-{:02}", index + 1),
                *name,
                index % airports.len(),
                vec![skill.into()],
                unavailable,
            )
        })
        .collect::<Vec<_>>();

    let legs = [
        ("SF101", 0, 1, 60, 540),
        ("SF102", 1, 0, 1_400, 1_880),
        ("SF201", 4, 3, 2_880, 3_420),
        ("SF202", 3, 4, 4_320, 4_860),
        ("SF301", 0, 2, 5_760, 6_480),
        ("SF302", 2, 0, 9_360, 10_080),
        ("SF401", 4, 5, 11_520, 12_600),
        ("SF402", 5, 4, 15_500, 16_580),
        ("SF501", 0, 3, 17_280, 17_400),
        ("SF502", 3, 0, 18_300, 18_420),
    ];
    let flights = legs
        .iter()
        .map(|(number, from, to, departure, arrival)| {
            Flight::new(*number, *number, *from, *to, *departure, *arrival)
        })
        .collect::<Vec<_>>();

    let mut assignments = Vec::new();
    for (flight_idx, flight) in flights.iter().enumerate() {
        let attendant_count =
            if flight.departure_airport_idx == 2 || flight.arrival_airport_idx == 2 {
                3
            } else {
                2
            };
        for seat in 0..(2 + attendant_count) {
            let skill = if seat < 2 {
                "Pilot"
            } else {
                "Flight attendant"
            };
            assignments.push(CrewAssignment::new(
                format!("{}-{}-{}", flight.id, skill.replace(' ', "-"), seat + 1),
                flight_idx,
                (seat + 1) as u32,
                skill,
                flight.departure_airport_idx,
                flight.arrival_airport_idx,
                flight.departure_minute,
                flight.arrival_minute,
            ));
        }
    }

    Plan::new(airports, employees, flights, assignments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_demo_has_complete_crew_demand() {
        let plan = generate(DemoData::Standard);
        assert_eq!(plan.airports.len(), 6);
        assert_eq!(plan.flights.len(), 10);
        assert_eq!(plan.crew_assignments.len(), 42);
        assert!(plan
            .crew_assignments
            .iter()
            .all(|assignment| assignment.employee_idx.is_none()));
    }
}
