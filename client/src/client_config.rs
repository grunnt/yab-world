use log::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub render_range_chunks: u16,
    pub camera_sensitivity_x: f32,
    pub camera_sensitivity_y: f32,
}

impl ClientConfig {
    pub fn defaults() -> Self {
        ClientConfig {
            render_range_chunks: 16,
            camera_sensitivity_x: 1.0,
            camera_sensitivity_y: 1.0,
        }
    }

    pub fn load() -> Self {
        match fs::read_to_string(ClientConfig::get_config_path()) {
            Ok(config_string) => match serde_json::from_str(&config_string) {
                Ok(config) => {
                    return validated(config);
                }
                Err(e) => {
                    warn!("Error parsing configuration file: {}", e);
                }
            },
            Err(e) => {
                warn!("Error loading configuration file: {}", e);
            }
        }
        ClientConfig::defaults()
    }

    pub fn save(&mut self) {
        let config_string = serde_json::to_string_pretty(&self).unwrap();
        fs::write(ClientConfig::get_config_path(), config_string).unwrap();
        info!(
            "Configuration file {:?} updated: {:?}",
            ClientConfig::get_config_path(),
            self
        );
    }

    fn get_config_path() -> PathBuf {
        PathBuf::from("config.json")
    }
}

fn validated(config: ClientConfig) -> ClientConfig {
    let mut validated = config.clone();
    let defaults = ClientConfig::defaults();
    if validated.render_range_chunks <= 4 || validated.render_range_chunks > 256 {
        warn!(
            "Incorrect render_range_chunks {}, resetting to default",
            validated.render_range_chunks
        );
        validated.render_range_chunks = defaults.render_range_chunks;
    }
    validated
}
