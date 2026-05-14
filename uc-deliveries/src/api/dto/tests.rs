use super::*;

#[test]
fn plan_dto_serializes_hard_soft_score_as_display_string() {
    let mut plan = Plan::new("score check", Vec::new(), Vec::new());
    plan.score = Some(HardSoftScore::of(0, -335));

    let dto = PlanDto::from_plan(&plan);
    assert_eq!(dto.score.as_deref(), Some("0hard/-335soft"));

    let value = serde_json::to_value(&dto).expect("dto should serialize");
    assert_eq!(value["score"], Value::String("0hard/-335soft".to_string()));
    assert!(
        !value["score"].is_object(),
        "score must not serialize as a JSON object"
    );
}

#[test]
fn plan_dto_ignores_inbound_score() {
    let mut fields = Map::new();
    fields.insert("name".to_string(), Value::String("spoofed".to_string()));
    fields.insert(
        "routingMode".to_string(),
        Value::String("straight_line".to_string()),
    );
    fields.insert("viewState".to_string(), Value::Object(Map::new()));
    fields.insert("deliveries".to_string(), Value::Array(Vec::new()));
    fields.insert("vehicles".to_string(), Value::Array(Vec::new()));

    let dto = PlanDto {
        fields,
        score: Some("0hard/-335soft".to_string()),
    };
    let plan = dto.to_domain().expect("dto should deserialize");

    assert_eq!(plan.score, None);
}
