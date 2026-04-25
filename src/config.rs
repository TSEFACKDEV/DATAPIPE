use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PipelineConfig {
    pub source: SourceConfig,
    pub destination: DestinationConfig,
    #[serde(default)]
    pub transforms: Vec<TransformConfig>,
    #[serde(default)]
    pub join: Option<JoinConfig>,
    #[serde(default)]
    pub schema: Option<SchemaConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub format: String, // "csv", "json", "delimited"
    pub path: String,
    pub delimiter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DestinationConfig {
    pub format: String, // "csv", "json", "jsonl"
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct TransformConfig {
    pub r#type: String, // "rename", "filter", "cast", "compute", "drop"
    pub from: Option<String>,
    pub to: Option<String>,
    pub column: Option<String>,
    pub value: Option<String>,
    pub target_type: Option<String>,
    pub new_column: Option<String>,
    pub expression: Option<String>,
    pub operator: Option<String>, // Pour filter: "=", "!=", "<", ">"
}

#[derive(Debug, Deserialize)]
pub struct JoinConfig {
    pub right_source: SourceConfig,
    pub left_key: String,
    pub right_key: String,
    pub join_type: String, // "inner", "left"
}

#[derive(Debug, Deserialize)]
pub struct SchemaConfig {
    pub required_columns: Vec<String>,
    pub column_types: Option<std::collections::HashMap<String, String>>,
}

impl PipelineConfig {
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: PipelineConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let toml_str = r#"
[source]
format = "csv"
path = "data/test.csv"
delimiter = ","

[destination]
format = "json"
path = "output/test.json"

[[transforms]]
type = "rename"
from = "old_name"
to = "new_name"
"#;

        let config: PipelineConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.source.format, "csv");
        assert_eq!(config.transforms.len(), 1);
    }
}