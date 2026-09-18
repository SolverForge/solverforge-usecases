use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
};
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    Stream, StreamExt,
};

use super::dto::JobLifecycleEventDto;
use super::routes::{status_from_solver_error, AppState};

#[cfg(test)]
const SSE_KEEP_ALIVE_INTERVAL: Duration = Duration::from_millis(25);
#[cfg(not(test))]
const SSE_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

pub async fn events(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let receiver = state
        .solver
        .subscribe(&id)
        .map_err(status_from_solver_error)?;
    let bootstrap = state
        .solver
        .bootstrap_event(&id)
        .map_err(status_from_solver_error)?;
    let bootstrap_event_sequence = Some(bootstrap.event_sequence);

    Ok(sse_response(job_event_stream(
        bootstrap,
        receiver,
        bootstrap_event_sequence,
    )))
}

fn job_event_stream(
    bootstrap: JobLifecycleEventDto,
    receiver: broadcast::Receiver<JobLifecycleEventDto>,
    bootstrap_event_sequence: Option<u64>,
) -> Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> {
    let bootstrap_stream =
        tokio_stream::iter(std::iter::once(Ok::<_, Infallible>(sse_event(bootstrap))));
    let live_stream = BroadcastStream::new(receiver).filter_map(move |message| match message {
        Ok(event) => {
            if event_is_not_newer(&event, bootstrap_event_sequence) {
                return None;
            }
            Some(Ok::<_, Infallible>(sse_event(event)))
        }
        Err(BroadcastStreamRecvError::Lagged(_)) => None,
    });
    Box::pin(bootstrap_stream.chain(live_stream))
}

fn sse_event(event: JobLifecycleEventDto) -> Event {
    let event_id = event.event_sequence.to_string();
    Event::default()
        .id(event_id)
        .json_data(event)
        .expect("event dto should serialize")
}

fn event_is_not_newer(event: &JobLifecycleEventDto, bootstrap_event_sequence: Option<u64>) -> bool {
    bootstrap_event_sequence.is_some_and(|sequence| event.event_sequence <= sequence)
}

fn sse_response(stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>) -> Response {
    let mut response = Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(SSE_KEEP_ALIVE_INTERVAL))
        .into_response();
    response
        .headers_mut()
        .insert("X-Accel-Buffering", HeaderValue::from_static("no"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::dto::TelemetryDto;
    use axum::body::Bytes;

    fn sample_event(event_sequence: u64) -> JobLifecycleEventDto {
        JobLifecycleEventDto {
            event_type: "progress",
            job_id: "7".to_string(),
            event_sequence,
            lifecycle_state: "SOLVING",
            terminal_reason: None,
            snapshot_revision: Some(3),
            current_score: Some("-4hard/-8soft".to_string()),
            best_score: Some("-4hard/-8soft".to_string()),
            telemetry: TelemetryDto {
                elapsed_ms: 20,
                step_count: 4,
                moves_generated: 12,
                moves_evaluated: 10,
                moves_accepted: 2,
                moves_applied: 1,
                moves_not_doable: 0,
                moves_acceptor_rejected: 0,
                moves_forager_ignored: 0,
                moves_hard_improving: 0,
                moves_hard_neutral: 0,
                moves_hard_worse: 0,
                conflict_repair_provider_generated: 0,
                conflict_repair_duplicate_filtered: 0,
                conflict_repair_illegal_filtered: 0,
                conflict_repair_not_doable_filtered: 0,
                conflict_repair_hard_improving: 0,
                conflict_repair_exposed: 0,
                score_calculations: 9,
                construction_slots_assigned: 0,
                construction_slots_kept: 0,
                construction_slots_no_doable: 0,
                generation_time_ms: 7,
                evaluation_time_ms: 8,
                moves_per_second: 500,
                acceptance_rate: 0.2,
                selectors: Vec::new(),
            },
            solution: None,
            error: None,
        }
    }

    #[tokio::test]
    async fn serialized_bootstrap_events_include_sse_id_and_payload() {
        let (_tx, rx) = broadcast::channel(8);
        let response = sse_response(job_event_stream(sample_event(2), rx, Some(2)));
        let mut body = response.into_body().into_data_stream();
        let chunk = body
            .next()
            .await
            .expect("bootstrap event should be emitted")
            .expect("sse chunk should be valid");
        let text = std::str::from_utf8(&chunk).expect("sse chunk should be utf8");
        assert!(text.contains("id: 2"));
        assert!(text.contains(r#"data: {"eventType":"progress""#));
    }

    #[test]
    fn live_filter_skips_events_already_covered_by_bootstrap() {
        assert!(event_is_not_newer(&sample_event(2), Some(2)));
        assert!(!event_is_not_newer(&sample_event(2), Some(1)));
        assert!(!event_is_not_newer(&sample_event(2), None));
    }

    #[tokio::test]
    async fn keep_alive_frames_arrive_for_idle_streams() {
        let (_tx, rx) = broadcast::channel(8);
        let response = sse_response(job_event_stream(sample_event(2), rx, Some(2)));
        let mut body = response.into_body().into_data_stream();
        body.next()
            .await
            .expect("bootstrap event should be emitted")
            .expect("bootstrap event should be valid");

        assert!(tokio::time::timeout(Duration::from_millis(5), body.next())
            .await
            .is_err());

        let chunk = tokio::time::timeout(
            SSE_KEEP_ALIVE_INTERVAL + Duration::from_millis(50),
            body.next(),
        )
        .await
        .expect("keep-alive should arrive after the configured interval")
        .expect("keep-alive frame should exist")
        .expect("keep-alive frame should be valid");
        assert_eq!(chunk, Bytes::from_static(b":\n\n"));
    }
}
