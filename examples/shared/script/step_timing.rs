use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepTiming {
    pub raw: String,
    pub secs: f64,
    pub failure: Option<String>,
}
