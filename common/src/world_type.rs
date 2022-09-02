use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GeneratorType {
    Flat,
    Water,
    Alien,
    Default,
}
