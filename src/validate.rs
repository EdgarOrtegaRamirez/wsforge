//! JSON Schema validation for WebSocket messages
//!
//! Validates messages against a simplified JSON schema format.
//! Supports: object (with required/optional fields), array, string, number, boolean, enum.

use serde_json::{Map, Value};

/// A simplified JSON schema for WebSocket message validation
#[derive(Debug, Clone)]
pub struct Schema {
    pub r#type: SchemaType,
    pub properties: Option<Vec<Property>>,
    pub required: Vec<String>,
    pub items: Option<Box<Schema>>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub schema: Schema,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaType {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Enum(Vec<String>),
}

/// Validation result for a single message
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validate a message against a schema
pub fn validate(message: &str, schema: &Schema) -> ValidationResult {
    let mut result = ValidationResult::new();

    let parsed: Value = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(e) => {
            result.add_error(format!("JSON parse error: {}", e));
            return result;
        }
    };

    validate_value(&parsed, schema, &mut result, "root");
    result
}

fn validate_value(value: &Value, schema: &Schema, result: &mut ValidationResult, path: &str) {
    match (&schema.r#type, value) {
        (SchemaType::Object, Value::Object(map)) => {
            validate_object(map, schema, result, path);
        }
        (SchemaType::Object, _) => {
            result.add_error(format!(
                "{}: expected object, got {}",
                path,
                value_type_name(value)
            ));
        }
        (SchemaType::Array, Value::Array(items)) => {
            if let Some(item_schema) = &schema.items {
                for (i, item) in items.iter().enumerate() {
                    validate_value(item, item_schema, result, &format!("{}[{}]", path, i));
                }
            }
        }
        (SchemaType::Array, _) => {
            result.add_error(format!(
                "{}: expected array, got {}",
                path,
                value_type_name(value)
            ));
        }
        (SchemaType::String, Value::String(_)) => {}
        (SchemaType::String, _) => {
            result.add_error(format!(
                "{}: expected string, got {}",
                path,
                value_type_name(value)
            ));
        }
        (SchemaType::Number, Value::Number(_)) => {}
        (SchemaType::Number, _) => {
            result.add_error(format!(
                "{}: expected number, got {}",
                path,
                value_type_name(value)
            ));
        }
        (SchemaType::Boolean, Value::Bool(_)) => {}
        (SchemaType::Boolean, _) => {
            result.add_error(format!(
                "{}: expected boolean, got {}",
                path,
                value_type_name(value)
            ));
        }
        (SchemaType::Enum(values), v) => {
            let s = match v {
                Value::String(s) => s.as_str(),
                _ => {
                    result.add_error(format!(
                        "{}: expected string for enum, got {}",
                        path,
                        value_type_name(v)
                    ));
                    return;
                }
            };
            if !values.contains(&s.to_string()) {
                result.add_error(format!("{}: value '{}' not in enum {:?}", path, s, values));
            }
        }
    }
}

fn validate_object(
    map: &Map<String, Value>,
    schema: &Schema,
    result: &mut ValidationResult,
    path: &str,
) {
    // Check required fields
    for req in &schema.required {
        if !map.contains_key(req.as_str()) {
            result.add_error(format!("{}: missing required field '{}'", path, req));
        }
    }

    // Validate known properties (only check non-required fields, since required check above handles those)
    if let Some(properties) = &schema.properties {
        let required_set: std::collections::HashSet<&str> =
            schema.required.iter().map(|s| s.as_str()).collect();
        for prop in properties {
            if let Some(value) = map.get(prop.name.as_str()) {
                validate_value(
                    value,
                    &prop.schema,
                    result,
                    &format!("{}.{}", path, prop.name),
                );
            } else if !prop.optional && !required_set.contains(prop.name.as_str()) {
                // Only report if not already caught by required check
                result.add_error(format!("{}: missing required field '{}'", path, prop.name));
            }
        }
    }

    // Warn about unknown fields
    if let Some(properties) = &schema.properties {
        let known: Vec<&str> = properties.iter().map(|p| p.name.as_str()).collect();
        for key in map.keys() {
            if !known.contains(&key.as_str()) {
                result.add_warning(format!("{}: unknown field '{}'", path, key));
            }
        }
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Parse a simple schema from JSON
pub fn parse_schema(json: &str) -> Result<Schema, String> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| format!("Invalid schema JSON: {}", e))?;

    let obj = value
        .as_object()
        .ok_or_else(|| "Schema must be a JSON object".to_string())?;

    let r#type = match obj.get("type").and_then(|v| v.as_str()) {
        Some("object") => SchemaType::Object,
        Some("array") => SchemaType::Array,
        Some("string") => SchemaType::String,
        Some("number") => SchemaType::Number,
        Some("boolean") => SchemaType::Boolean,
        Some("enum") => {
            let items = obj
                .get("enum")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "Enum schema missing 'enum' array".to_string())?;
            SchemaType::Enum(
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
            )
        }
        Some(t) => return Err(format!("Unsupported schema type: {}", t)),
        None => return Err("Schema missing 'type' field".to_string()),
    };

    let mut required = Vec::new();
    let mut properties = Vec::new();

    if r#type == SchemaType::Object {
        if let Some(req) = obj.get("required") {
            required = req
                .as_array()
                .ok_or_else(|| "'required' must be an array".to_string())?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }

        if let Some(props) = obj.get("properties") {
            for (name, prop_schema) in props.as_object().ok_or("'properties' must be an object")? {
                let prop_schema =
                    parse_schema(&serde_json::to_string(prop_schema).map_err(|e| e.to_string())?)?;
                let optional = !required.contains(name);
                properties.push(Property {
                    name: name.clone(),
                    schema: prop_schema,
                    optional,
                });
            }
        }
    }

    Ok(Schema {
        r#type,
        properties: if properties.is_empty() {
            None
        } else {
            Some(properties)
        },
        required,
        items: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_object() {
        let schema = Schema {
            r#type: SchemaType::Object,
            properties: Some(vec![Property {
                name: "name".to_string(),
                schema: Schema {
                    r#type: SchemaType::String,
                    properties: None,
                    required: Vec::new(),
                    items: None,
                },
                optional: false,
            }]),
            required: vec!["name".to_string()],
            items: None,
        };

        let result = validate(r#"{"name": "test"}"#, &schema);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_required_field() {
        let schema = Schema {
            r#type: SchemaType::Object,
            properties: Some(vec![Property {
                name: "name".to_string(),
                schema: Schema {
                    r#type: SchemaType::String,
                    properties: None,
                    required: Vec::new(),
                    items: None,
                },
                optional: false,
            }]),
            required: vec!["name".to_string()],
            items: None,
        };

        let result = validate(r#"{}"#, &schema);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_invalid_type() {
        let schema = Schema {
            r#type: SchemaType::String,
            properties: None,
            required: Vec::new(),
            items: None,
        };

        let result = validate(r#"123"#, &schema);
        assert!(!result.valid);
    }

    #[test]
    fn test_parse_schema() {
        let json = r#"{
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        }"#;
        let schema = parse_schema(json).unwrap();
        assert_eq!(schema.required.len(), 2);
        assert!(schema.required.contains(&"name".to_string()));
        assert!(schema.required.contains(&"age".to_string()));
    }

    #[test]
    fn test_enum_validation() {
        let schema = Schema {
            r#type: SchemaType::Enum(vec![
                "red".to_string(),
                "green".to_string(),
                "blue".to_string(),
            ]),
            properties: None,
            required: Vec::new(),
            items: None,
        };

        let result = validate(r#""red""#, &schema);
        assert!(result.valid);

        let result = validate(r#""yellow""#, &schema);
        assert!(!result.valid);
    }
}
