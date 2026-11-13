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

    fn share_with(full: Option<&str>) -> core_direct_dial::OwnShareAddress {
        core_direct_dial::OwnShareAddress(Some(core_direct_dial::OwnAddress {
            id: identity::PeerId::from("me"),
            short: String::from("12D3Koo...99RC"),
            full: full.map(String::from),
        }))
    }

    fn labels(app: &mut App) -> Vec<String> {
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<ui::direct_dial::OwnAddressText>>();
        texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(share_with(Some("/ip4/127.0.0.1/tcp/4001/p2p/me")));
        app.world_mut()
            .spawn((ui::direct_dial::OwnAddressText, Text::new("old")));
        app.add_systems(Update, refresh_own_address);
        app.update();
        let shown = labels(&mut app);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0], "12D3Koo...99RC");
    }

    #[test]
    fn waiting_label_without_the_resource() {
        let mut app = App::new();
        app.world_mut()
            .spawn((ui::direct_dial::OwnAddressText, Text::new("old")));
        app.add_systems(Update, refresh_own_address);
        app.update();
        let shown = labels(&mut app);
        assert_eq!(shown.len(), 1);
        assert!(shown[0].contains("waiting"));
    }

    #[test]
    fn waiting_label_while_the_address_is_unknown() {
        let mut app = App::new();
        app.insert_resource(share_with(None));
        app.world_mut()
            .spawn((ui::direct_dial::OwnAddressText, Text::new("old")));
        app.add_systems(Update, refresh_own_address);
        app.update();
        assert_eq!(labels(&mut app)[0], "waiting for network...");
    }

    #[test]
    fn the_chip_shows_the_short_form_never_the_full_address() {
        let mut app = App::new();
        app.insert_resource(share_with(Some("/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW")));
        app.world_mut()
            .spawn((ui::direct_dial::OwnAddressText, Text::new("old")));
        app.add_systems(Update, refresh_own_address);
        app.update();
        let shown = labels(&mut app);
        assert!(!shown.iter().any(|text| text.contains("/ip4/")));
        assert_eq!(shown[0], "12D3Koo...99RC");
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<core_direct_dial::OwnShareAddress>();
        app.add_systems(
            Startup,
            (
                spawn_label,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
        app.add_systems(Update, refresh_own_address);
    }

    fn spawn_label(mut commands: Commands) {
        commands.spawn((ui::direct_dial::OwnAddressText, Text::new("old")));
    }

    const fn preview_noop(_world: &mut World) {}

    fn show_address(world: &mut World) {
        **world.resource_mut::<core_direct_dial::OwnShareAddress>() =
            Some(core_direct_dial::OwnAddress {
                id: identity::PeerId::from("me"),
                short: String::from("12D3Koo...99RC"),
                full: Some(String::from("/ip4/127.0.0.1/tcp/4001/p2p/me")),
            });
    }

    #[test]
    fn refresh_own_address_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_own_address"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("waiting for network"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("short form on the chip"),
                    apply: show_address,
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
