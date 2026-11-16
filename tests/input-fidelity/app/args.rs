use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};
use clap::Parser;
use freenet_libp2p_bevy_core::testing::E2EDriverPlugin;

use crate::probe_e2e_methods::ProbeE2EMethods;

pub const PROBE_WINDOW_SIZE: (u32, u32) = (960, 540);
pub const PROBE_WINDOW_ORIGIN: IVec2 = IVec2::new(0, 0);
pub const PROBE_WINDOW_TITLE: &str = "input-probe";

#[derive(Debug, Parser)]
#[command(name = "input_probe")]
pub struct ProbeArgs {
    #[arg(long)]
    pub e2e_port: u16,
}

impl ProbeArgs {
    pub fn init_tracing() {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .init();
    }

    #[must_use]
    pub fn run(&self) -> AppExit {
        let window = Window {
            title: PROBE_WINDOW_TITLE.to_string(),
            resolution: PROBE_WINDOW_SIZE.into(),
            position: WindowPosition::At(PROBE_WINDOW_ORIGIN),
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        };
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_plugins(E2EDriverPlugin::new(self.e2e_port).with(ProbeE2EMethods));
        app.run()
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::ProbeArgs;

    #[test]
    fn test_usage() {
        let args =
            ProbeArgs::try_parse_from(["input_probe", "--e2e-port", "15799"]).expect("args parse");
        assert_eq!(args.e2e_port, 15799);
    }
}
