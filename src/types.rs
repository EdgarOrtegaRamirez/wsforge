//! Type generation from WebSocket message samples
//!
//! Auto-generate TypeScript and Python type definitions from JSON message samples.

use serde_json::Value;
use std::collections::HashSet;

/// TypeScript type definition
#[derive(Debug, Clone)]
pub struct TypeScriptType {
    pub name: String,
    pub definition: String,
}

/// Python type definition
#[derive(Debug, Clone)]
pub struct PythonType {
    #[allow(dead_code)]
    pub name: String,
    pub definition: String,
    pub imports: Vec<String>,
}

/// Generate type definitions from message samples
pub struct TypeGenerator {
    pub type_name: String,
}

impl TypeGenerator {
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
        }
    }

    /// Generate TypeScript types from JSON samples
    pub fn generate_typescript(&self, samples: &[String]) -> Vec<TypeScriptType> {
        if samples.is_empty() {
            return vec![TypeScriptType {
                name: self.type_name.clone(),
                definition: format!("{}: any;", self.type_name),
            }];
        }

        let types = samples
            .iter()
            .filter_map(|s| {
                let value: Value = serde_json::from_str(s).ok()?;
                Some(self.json_to_ts(&value))
            })
            .collect::<Vec<_>>();

        if types.is_empty() {
            return vec![TypeScriptType {
                name: self.type_name.clone(),
                definition: format!("{}: any;", self.type_name),
            }];
        }

        vec![TypeScriptType {
            name: self.type_name.clone(),
            definition: types.join("\n"),
        }]
    }

    /// Generate Python type hints from JSON samples
    pub fn generate_python(&self, samples: &[String]) -> Vec<PythonType> {
        if samples.is_empty() {
            return vec![PythonType {
                name: self.type_name.clone(),
                definition: format!("{}: Any", self.type_name),
                imports: vec!["from typing import Any".to_string()],
            }];
        }

        let mut import_set: HashSet<String> = HashSet::new();
        let types = samples
            .iter()
            .filter_map(|s| {
                let value: Value = serde_json::from_str(s).ok()?;
                let py_type = self.json_to_python(&value, &mut import_set);
                Some(py_type)
            })
            .collect::<Vec<_>>();

        let mut imports: Vec<String> = import_set.into_iter().collect();
        imports.sort();

        if types.is_empty() {
            return vec![PythonType {
                name: self.type_name.clone(),
                definition: format!("{}: Any", self.type_name),
                imports: vec!["from typing import Any".to_string()],
            }];
        }

        vec![PythonType {
            name: self.type_name.clone(),
            definition: types.join("\n"),
            imports,
        }]
    }

    fn json_to_ts(&self, value: &Value) -> String {
        match value {
            Value::Null => "undefined".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(items) => {
                if items.is_empty() {
                    "unknown[]".to_string()
                } else {
                    let item_types: HashSet<&str> = items.iter().map(|v| json_ts_type(v)).collect();
                    if item_types.len() == 1 {
                        format!("{}[]", item_types.into_iter().next().unwrap())
                    } else {
                        format!(
                            "({})[]",
                            item_types.iter().copied().collect::<Vec<_>>().join(" | ")
                        )
                    }
                }
            }
            Value::Object(map) => {
                let fields: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}?: {}", k, json_ts_type(v)))
                    .collect();
                format!("{{ {} }}", fields.join("; "))
            }
        }
    }

    fn json_to_python(&self, value: &Value, import_set: &mut HashSet<String>) -> String {
        match value {
            Value::Null => {
                import_set.insert("None".to_string());
                "None".to_string()
            }
            Value::Bool(_) => "bool".to_string(),
            Value::Number(n) => {
                if n.is_f64() {
                    "float".to_string()
                } else {
                    "int".to_string()
                }
            }
            Value::String(_) => "str".to_string(),
            Value::Array(items) => {
                import_set.insert("List".to_string());
                if items.is_empty() {
                    "List[object]".to_string()
                } else {
                    let item_types: HashSet<String> = items
                        .iter()
                        .map(|v| self.json_py_type(v, import_set))
                        .collect();
                    if item_types.len() == 1 {
                        format!("List[{}]", item_types.into_iter().next().unwrap())
                    } else {
                        let v: Vec<&String> = item_types.iter().collect();
                        let joined: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
                        format!("List[{}]", joined.join(" | "))
                    }
                }
            }
            Value::Object(map) => {
                let mut class_name = format!(
                    "{}{}",
                    self.type_name,
                    capitalize(map.keys().next().map(|k| k.as_str()).unwrap_or("Data"))
                );
                if map.len() > 1 {
                    // If multiple top-level objects have different names, just use base name
                    class_name = self.type_name.clone();
                }

                let fields: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        let py_type = self.json_py_type(v, import_set);
                        format!("    {}: {}", k, py_type)
                    })
                    .collect();

                format!(
                    "class {}({}):\n{}",
                    class_name,
                    "TypedDict",
                    fields.join("\n")
                )
            }
        }
    }

    fn json_py_type(&self, value: &Value, import_set: &mut HashSet<String>) -> String {
        match value {
            Value::Null => {
                import_set.insert("Optional".to_string());
                "Optional[str]".to_string()
            }
            Value::Bool(_) => "bool".to_string(),
            Value::Number(n) => {
                if n.is_f64() {
                    "float".to_string()
                } else {
                    "int".to_string()
                }
            }
            Value::String(_) => "str".to_string(),
            Value::Array(_) => {
                import_set.insert("List".to_string());
                "List[object]".to_string()
            }
            Value::Object(_) => {
                import_set.insert("Dict".to_string());
                "Dict[str, object]".to_string()
            }
        }
    }
}

fn json_ts_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "unknown[]",
        Value::Object(_) => "{ ... }",
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_string_type() {
        let gen = TypeGenerator::new("Message");
        let samples = vec![r#"{"name": "test"}"#.to_string()];
        let types = gen.generate_typescript(&samples);
        assert_eq!(types.len(), 1);
        assert!(types[0].definition.contains("name"));
        assert!(types[0].definition.contains("string"));
    }

    #[test]
    fn test_typescript_number_type() {
        let gen = TypeGenerator::new("Event");
        let samples = vec![r#"{"count": 42}"#.to_string()];
        let types = gen.generate_typescript(&samples);
        assert!(types[0].definition.contains("count"));
        assert!(types[0].definition.contains("number"));
    }

    #[test]
    fn test_typescript_empty_samples() {
        let gen = TypeGenerator::new("Message");
        let samples: Vec<String> = vec![];
        let types = gen.generate_typescript(&samples);
        assert!(types[0].definition.contains("any"));
    }

    #[test]
    fn test_python_string_type() {
        let gen = TypeGenerator::new("Message");
        let samples = vec![r#"{"name": "test"}"#.to_string()];
        let types = gen.generate_python(&samples);
        assert_eq!(types.len(), 1);
        assert!(types[0].definition.contains("name"));
        assert!(types[0].definition.contains("str"));
    }

    #[test]
    fn test_python_number_type() {
        let gen = TypeGenerator::new("Event");
        let samples = vec![r#"{"count": 42}"#.to_string()];
        let types = gen.generate_python(&samples);
        assert!(types[0].definition.contains("count"));
    }

    #[test]
    fn test_python_empty_samples() {
        let gen = TypeGenerator::new("Message");
        let samples: Vec<String> = vec![];
        let types = gen.generate_python(&samples);
        assert!(types[0].definition.contains("Any"));
    }
}
