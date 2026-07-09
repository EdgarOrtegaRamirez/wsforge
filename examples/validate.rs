//! Example: Validate WebSocket messages against a schema
use wsforge::validate::{parse_schema, validate};

fn main() {
    let schema_json = r#"{
        "type": "object",
        "required": ["type", "data"],
        "properties": {
            "type": {"type": "string"},
            "data": {"type": "string"}
        }
    }"#;

    let schema = parse_schema(schema_json).expect("Invalid schema");

    let valid_msg = r#"{"type": "chat", "data": "Hello!"}"#;
    let result = validate(valid_msg, &schema);
    println!("Valid: {} (errors: {})", result.valid, result.errors.len());

    let invalid_msg = r#"{"type": 123}"#;
    let result = validate(invalid_msg, &schema);
    println!("Valid: {} (errors: {})", result.valid, result.errors.len());
}
