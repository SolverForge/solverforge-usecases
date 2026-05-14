//! Deterministic Bergamo demo-data builder and routing preparation.
//!
//! The public app starts from ordinary domain facts: locations, service visits,
//! technician routes, and travel legs. Road-network preparation enriches those
//! facts before solving, but the solver still receives a normal
//! `FieldServicePlan`.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use solverforge_maps::{
    BoundingBox, Coord, NetworkConfig, NetworkRef, RoadNetwork, RoutingError, UNREACHABLE,
};

use super::bergamo_locations::{DEPOTS, SERVICE_LOCATIONS};
use super::bergamo_profiles::VISIT_PROFILES;
use super::bergamo_technicians::TECHNICIANS;
use crate::domain::{
    FieldServicePlan, Location, ServiceVisit, ServiceVisitInit, TechnicianRoute,
    TechnicianRouteInit, TravelLeg, TravelLegInit,
};

const BERGAMO_BBOX: BoundingBox = BoundingBox {
    min_lat: 45.64,
    min_lng: 9.58,
    max_lat: 45.75,
    max_lng: 9.78,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoData {
    Standard,
}

#[derive(Debug)]
pub enum DemoDataError {
    Routing(RoutingError),
}

impl fmt::Display for DemoDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Routing(error) => write!(f, "Bergamo OSM routing data is unavailable: {error}"),
        }
    }
}

impl std::error::Error for DemoDataError {}

impl From<RoutingError> for DemoDataError {
    fn from(error: RoutingError) -> Self {
        Self::Routing(error)
    }
}

const AVAILABLE_DEMO_DATA: &[DemoData] = &[DemoData::Standard];
const DEFAULT_DEMO_DATA: DemoData = DemoData::Standard;

pub fn default_demo_data() -> DemoData {
    DEFAULT_DEMO_DATA
}

/// Returns the complete list of public demo ids exposed through `/demo-data`.
pub fn available_demo_data() -> &'static [DemoData] {
    AVAILABLE_DEMO_DATA
}

impl DemoData {
    pub fn id(self) -> &'static str {
        match self {
            DemoData::Standard => "STANDARD",
        }
    }

    pub fn default_demo_data() -> Self {
        default_demo_data()
    }

    pub fn available_demo_data() -> &'static [Self] {
        available_demo_data()
    }

    fn technician_count(self) -> usize {
        match self {
            Self::Standard => 6,
        }
    }

    fn visit_count(self) -> usize {
        match self {
            Self::Standard => SERVICE_LOCATIONS.len() * 2,
        }
    }
}

impl FromStr for DemoData {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "STANDARD" => Ok(DemoData::Standard),
            _ => Err(()),
        }
    }
}

/// Builds the requested demo plan and prepares road-network travel facts.
pub async fn generate(demo: DemoData) -> Result<FieldServicePlan, DemoDataError> {
    // The initial demo response must be fast and deterministic, so it ships only
    // seed self-legs. Full road-network legs are prepared when a solve starts.
    let locations = build_locations(demo);
    let travel_legs = build_seed_travel_legs(locations.len());
    let service_visits = build_service_visits(demo);
    let technician_routes = build_technician_routes(demo);

    Ok(FieldServicePlan::new(
        locations,
        service_visits,
        travel_legs,
        technician_routes,
    ))
}

/// Replaces seed travel legs with road-network durations and distances.
pub async fn prepare_routing(plan: &mut FieldServicePlan) -> Result<(), DemoDataError> {
    // This is the expensive OSM-backed step. It runs once per submitted plan so
    // every candidate route move is scored against a stable travel matrix.
    let coords = plan
        .locations
        .iter()
        .map(|location| Coord::new(location.lat(), location.lng()))
        .collect::<Vec<_>>();
    let network = load_network().await?;
    let matrix = network.compute_matrix(&coords, None).await;
    plan.travel_legs = build_travel_legs(&matrix, coords.len());
    Ok(())
}

pub async fn load_network() -> Result<NetworkRef, DemoDataError> {
    RoadNetwork::load_or_fetch(&BERGAMO_BBOX, &network_config(), None)
        .await
        .map_err(DemoDataError::from)
}

pub fn network_config() -> NetworkConfig {
    NetworkConfig::default()
        .cache_dir(PathBuf::from(".osm_cache/field-service-routing/bergamo"))
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .overpass_max_retries(1)
        .overpass_retry_backoff(Duration::from_secs(2))
}

fn build_locations(demo: DemoData) -> Vec<Location> {
    let service_location_count = demo.visit_count().min(SERVICE_LOCATIONS.len());

    DEPOTS
        .iter()
        .map(|seed| seed.to_location("depot"))
        .chain(
            SERVICE_LOCATIONS
                .iter()
                .take(service_location_count)
                .map(|seed| seed.to_location("customer")),
        )
        .collect()
}

fn build_service_visits(demo: DemoData) -> Vec<ServiceVisit> {
    (0..demo.visit_count())
        .map(|idx| {
            let seed = &SERVICE_LOCATIONS[idx % SERVICE_LOCATIONS.len()];
            let profile = VISIT_PROFILES[idx % VISIT_PROFILES.len()];
            ServiceVisit::new(ServiceVisitInit {
                id: format!("visit-{idx:02}"),
                name: profile.name.to_string(),
                customer: seed.label.to_string(),
                location_idx: DEPOTS.len() + (idx % SERVICE_LOCATIONS.len()),
                duration_minutes: profile.duration_minutes,
                earliest_minute: profile.earliest_minute,
                latest_minute: profile.latest_minute,
                required_skill_mask: profile.required_skill_mask,
                required_parts_mask: profile.required_parts_mask,
                priority: profile.priority,
                territory: seed.territory.to_string(),
            })
        })
        .collect()
}

fn build_technician_routes(demo: DemoData) -> Vec<TechnicianRoute> {
    TECHNICIANS
        .iter()
        .take(demo.technician_count())
        .enumerate()
        .map(|(idx, seed)| {
            TechnicianRoute::new(TechnicianRouteInit {
                id: format!("route-{idx:02}"),
                technician_id: seed.id.to_string(),
                technician_name: seed.name.to_string(),
                color: seed.color.to_string(),
                start_location_idx: seed.start_location_idx,
                end_location_idx: seed.end_location_idx,
                shift_start_minute: 8 * 60,
                shift_end_minute: 18 * 60,
                max_route_minutes: 10 * 60,
                skill_mask: seed.skill_mask,
                inventory_mask: seed.inventory_mask,
                territory: seed.territory.to_string(),
            })
        })
        .collect()
}

fn build_seed_travel_legs(width: usize) -> Vec<TravelLeg> {
    (0..width)
        .map(|idx| {
            TravelLeg::new(TravelLegInit {
                id: format!("leg-{idx:02}-{idx:02}"),
                name: format!("leg-{idx:02}-{idx:02}"),
                from_location_idx: idx,
                to_location_idx: idx,
                duration_seconds: 0,
                distance_meters: 0,
                reachable: true,
            })
        })
        .collect()
}

fn build_travel_legs(matrix: &solverforge_maps::TravelTimeMatrix, width: usize) -> Vec<TravelLeg> {
    let mut legs = Vec::with_capacity(width * width);

    for from in 0..width {
        for to in 0..width {
            let (duration_seconds, distance_meters, reachable) = if from == to {
                (0, 0, true)
            } else {
                let matrix_duration = matrix.get(from, to).unwrap_or(UNREACHABLE);
                let matrix_distance = matrix.distance_meters(from, to).unwrap_or(UNREACHABLE);
                if matrix_duration == UNREACHABLE || matrix_distance == UNREACHABLE {
                    (0, 0, false)
                } else {
                    (matrix_duration, matrix_distance, true)
                }
            };

            legs.push(TravelLeg::new(TravelLegInit {
                id: format!("leg-{from:02}-{to:02}"),
                name: format!("leg-{from:02}-{to:02}"),
                from_location_idx: from,
                to_location_idx: to,
                duration_seconds,
                distance_meters,
                reachable,
            }));
        }
    }

    legs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_technician_routes_start_without_assigned_visits() {
        for demo in DemoData::available_demo_data() {
            let routes = build_technician_routes(*demo);

            assert!(!routes.is_empty());
            assert!(routes.iter().all(|route| route.visits.is_empty()));
        }
    }

    #[tokio::test]
    async fn generated_seed_plan_has_only_identity_travel_legs() {
        let plan = generate(DemoData::Standard).await.unwrap();

        assert_eq!(plan.travel_legs.len(), plan.locations.len());
        assert!(plan.travel_legs.iter().enumerate().all(|(idx, leg)| {
            leg.from_location_idx == idx
                && leg.to_location_idx == idx
                && leg.duration_seconds == 0
                && leg.distance_meters == 0
                && leg.reachable
        }));
    }
}
