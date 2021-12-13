use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum GeneratorType {
    Flat,
    Water,
    Default,
}
