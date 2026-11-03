//! DTO tests for transport decoding and runtime telemetry.

use super::*;
use std::time::Duration;

#[test]
fn plan_dto_returns_decode_errors_for_semantically_invalid_payloads() {
    let dto = PlanDto {
        fields: Map::new(),
        score: None,
    };

    assert!(dto.to_domain().is_err());
}

#[test]
fn runtime_telemetry_derives_stock_transport_fields() {
    let telemetry = SolverTelemetry {
        elapsed: Duration::from_millis(2_500),
        step_count: 9,
        moves_generated: 300,
        moves_evaluated: 200,
        moves_accepted: 50,
        score_calculations: 80,
        generation_time: Duration::from_millis(400),
        evaluation_time: Duration::from_millis(900),
        ..SolverTelemetry::default()
    };

    let dto = TelemetryDto::from_runtime(&telemetry);

    assert_eq!(dto.elapsed_ms, 2_500);
    assert_eq!(dto.step_count, 9);
    assert_eq!(dto.moves_generated, 300);
    assert_eq!(dto.moves_evaluated, 200);
    assert_eq!(dto.moves_accepted, 50);
    assert_eq!(dto.score_calculations, 80);
    assert_eq!(dto.generation_ms, 400);
    assert_eq!(dto.evaluation_ms, 900);
    assert_eq!(dto.moves_per_second, 80);
    assert!((dto.acceptance_rate - 0.25).abs() < f64::EPSILON);
}
