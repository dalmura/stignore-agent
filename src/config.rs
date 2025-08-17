use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug)]
pub enum ConfigError {
    FileRead {
        filename: String,
        source: std::io::Error,
    },
    Parse {
        filename: String,
        source: toml::de::Error,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead { filename, source } => {
                write!(f, "Could not read config file '{}': {}", filename, source)
            }
            ConfigError::Parse { filename, source } => {
                write!(f, "Unable to parse config file '{}': {}", filename, source)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::FileRead { source, .. } => Some(source),
            ConfigError::Parse { source, .. } => Some(source),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub relative_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub port: u16,
    pub base_path: String,
    pub api_key: String,
}

// Parent struct holding the entire config file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub(crate) agent: AgentConfig,
    pub(crate) categories: Vec<Category>,
}

pub fn load_config(filename: &str) -> Result<Data, ConfigError> {
    let contents = fs::read_to_string(filename).map_err(|source| ConfigError::FileRead {
        filename: filename.to_string(),
        source,
    })?;

    let data: Data = toml::from_str(&contents).map_err(|source| ConfigError::Parse {
        filename: filename.to_string(),
        source,
    })?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn serde_valid_config() {
        let data: Result<Data, toml::de::Error> = toml::from_str(
            r#"
           [agent]
           port = 3000
           name = "Agent Smith"
           base_path = "/path/to/stuff"
           api_key = "550e8400-e29b-41d4-a716-446655440000"
        
           [[categories]]
           id = "category_a"
           name = "Category A"
           relative_path = "a/"
        "#,
        );
        assert!(data.is_ok());
    }

    #[test]
    fn serde_invalid_config() {
        let data: Result<Data, toml::de::Error> = toml::from_str(
            r#"
           [agent]
           port = 3000
           fake_field = 0
           name = "Agent Smith"
        "#,
        );
        assert!(data.is_err());
    }

    #[test]
    fn load_config_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[agent]
port = 3001
name = "Test Agent"
base_path = "/tmp/test"
api_key = "550e8400-e29b-41d4-a716-446655440000"

[[categories]]
id = "test_category"
name = "Test Category"
relative_path = "test/"
        "#;

        temp_file.write_all(config_content.as_bytes()).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let result = load_config(file_path);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.agent.port, 3001);
        assert_eq!(data.agent.name, "Test Agent");
        assert_eq!(data.categories.len(), 1);
        assert_eq!(data.categories[0].id, "test_category");
    }

    #[test]
    fn load_config_file_not_found() {
        let result = load_config("nonexistent_file.toml");
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::FileRead { filename, .. } => {
                assert_eq!(filename, "nonexistent_file.toml");
            }
            _ => panic!("Expected FileRead error"),
        }
    }

    #[test]
    fn load_config_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_config = "this is not valid toml content [unclosed bracket";

        temp_file.write_all(invalid_config.as_bytes()).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let result = load_config(file_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::Parse { filename, .. } => {
                assert_eq!(filename, file_path);
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn load_config_missing_required_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let incomplete_config = r#"
[agent]
port = 3001
# missing name, base_path, api_key
        "#;

        temp_file.write_all(incomplete_config.as_bytes()).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let result = load_config(file_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::Parse { .. } => {
                // Expected - missing required fields should cause parse error
            }
            _ => panic!("Expected Parse error for missing required fields"),
        }
    }
}
