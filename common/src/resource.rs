use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type Resource = u16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceDef {
    pub resource: Resource,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceRegistry {
    resources: HashMap<Resource, ResourceDef>,
}

impl ResourceRegistry {
    pub fn empty() -> ResourceRegistry {
        ResourceRegistry {
            resources: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        let mut registry = ResourceRegistry {
            resources: HashMap::new(),
        };
        registry.add(0, "Common");
        registry.add(1, "Iron");
        registry.add(2, "Gold");
        registry
    }

    fn add(&mut self, resource: Resource, name: &str) {
        self.resources.insert(
            resource,
            ResourceDef {
                resource,
                name: name.to_string(),
            },
        );
    }

    pub fn new(folder_path: &Path) -> ResourceRegistry {
        let path = folder_path.join("resources.json");
        match fs::read_to_string(&path) {
            Ok(string) => match serde_json::from_str(&string) {
                Ok(value) => {
                    return value;
                }
                Err(e) => {
                    warn!("Error loading file: {}", e);
                }
            },
            Err(e) => {
                warn!("Error loading file: {}", e);
            }
        }
        let mut defaults = ResourceRegistry::default();
        defaults.save(&path);
        defaults
    }

    fn save(&mut self, path: &Path) {
        let string = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, string).unwrap();
        info!("File {:?} created", path);
    }

    pub fn get_def(&self, resource: Resource) -> &ResourceDef {
        self.resources.get(&resource).unwrap()
    }

    pub fn resources(&self) -> &HashMap<Resource, ResourceDef> {
        &self.resources
    }
}
