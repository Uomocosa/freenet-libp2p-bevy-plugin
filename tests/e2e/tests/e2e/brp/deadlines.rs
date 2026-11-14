use std::time::Duration;

use freenet_libp2p_bevy_plugin::testing::Network;

const FAKE_ACTION: Duration = Duration::from_secs(60);
const FAKE_LOBBY: Duration = Duration::from_secs(120);
/// Real networks wait 3.5 times longer: the lobby budget becomes 420 s, the orchestrator's
/// room-discovery budget for `Mainnet` and `LocalGateways` (`orchestrator.rs`, `default_wait_deadline`).
const REAL_NETWORK_MULTIPLIER: f64 = 3.5;

/// The waits one run uses. `E2E_TIME_SCALE` overrides the per-network multiplier, as it does
/// for the old orchestrator (`orchestrator.rs`, `time_scale`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deadlines {
    multiplier: f64,
}

impl Deadlines {
    #[must_use]
    pub fn for_network(network: Network) -> Self {
        let default = match network {
            Network::Fake => 1.0,
            Network::Mainnet | Network::LocalGateways | Network::DirectDial => {
                REAL_NETWORK_MULTIPLIER
            }
        };
        Self {
            multiplier: scale_from_env().unwrap_or(default),
        }
    }

    /// `base` is the fake-network budget; real networks stretch it by the multiplier.
    #[must_use]
    pub fn scaled(self, base: Duration) -> Duration {
        base.mul_f64(self.multiplier)
    }

    #[must_use]
    pub fn action(self) -> Duration {
        self.scaled(FAKE_ACTION)
    }

    #[must_use]
    pub fn lobby(self) -> Duration {
        self.scaled(FAKE_LOBBY)
    }
}

fn scale_from_env() -> Option<f64> {
    std::env::var("E2E_TIME_SCALE")
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|scale| scale.is_finite() && *scale > 0.0)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use freenet_libp2p_bevy_plugin::testing::Network;

    use super::Deadlines;

    #[test]
    fn test_usage() {
        let fake = Deadlines { multiplier: 1.0 };
        assert_eq!(fake.lobby(), Duration::from_secs(120));
        assert_eq!(fake.action(), Duration::from_secs(60));
        let real = Deadlines::for_network(Network::Mainnet);
        let scaled = real.lobby();
        assert!(scaled >= Duration::from_secs(400));
        assert!(Deadlines::for_network(Network::Fake).action() <= Duration::from_secs(60) * 2);
    }
}
