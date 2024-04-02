use serde::{Deserialize, Serialize};
use std::fs;
use std::process::exit;

// Parent struct holding the entire config file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub(crate) agent: AgentConfig,
    pub(crate) categories: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub port: u16,
    pub base_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub relative_path: String,
}

pub fn load_config(filename: &str) -> Data {
    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`", filename);
            exit(1);
        }
    };

    let data: Data = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(a) => {
            eprintln!("Unable to load data from `{}`", filename);
            eprintln!("`{}`", a);
            exit(1);
        }
    };

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_valid_config() {
        let data: Result<Data, toml::de::Error> = toml::from_str(
            r#"
           [agent]
           port = 3000
           name = "Agent Smith"
           base_path = "/path/to/stuff"
        
           [[categories]]
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
}
