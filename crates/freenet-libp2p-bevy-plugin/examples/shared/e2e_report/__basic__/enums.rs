use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckLayer {
    Scenario,
    Freenet,
    Libp2p,
}
