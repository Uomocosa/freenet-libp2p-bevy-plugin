use crate::ui;
use bevy::prelude::*;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

pub fn refresh_own_address(
    share: Option<Res<core_direct_dial::OwnShareAddress>>,
    mut texts: Query<&mut Text, With<ui::direct_dial::OwnAddressText>>,
) {
    let label = match share.map(|share| (**share).clone()) {
        None | Some(None) => String::from("waiting for network..."),
        Some(Some(address)) => match address.full {
            Some(_) => address.short,
            None => String::from("waiting for network..."),
        },
    };
    for mut text in &mut texts {
        *text = Text::new(label.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;

    use super::refresh_own_address;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn refresh_own_address_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_own_address"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.init_resource::<core_direct_dial::OwnShareAddress>();
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
                            commands.spawn((ui::direct_dial::OwnAddressText, Text::new("old")));
                        },
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
                app.add_systems(Update, refresh_own_address);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("waiting for network"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("short form on the chip"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<core_direct_dial::OwnShareAddress>() =
                            Some(core_direct_dial::OwnAddress {
                                id: identity::PeerId::from("me"),
                                short: String::from("12D3Koo...99RC"),
                                full: Some(String::from("/ip4/127.0.0.1/tcp/4001/p2p/me")),
                            });
                    },
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
