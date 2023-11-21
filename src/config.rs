use serde::{Serialize,Deserialize};
use std::fs;
use std::process::exit;

#[derive(Serialize,Deserialize,Clone)]
pub struct Data {
    pub(crate) agent: AgentConfig,
    category: toml::Table,
}

// Config struct holds to data from the `[config]` section.
#[derive(Serialize,Deserialize,Clone)]
pub struct AgentConfig {
    pub name: String,
    pub port: u16,
    pub base_path: String,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct Categories {
    pub name: String,
    pub config: CategoryConfig,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct CategoryConfig {
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
    #[test]
    fn placeholder() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}