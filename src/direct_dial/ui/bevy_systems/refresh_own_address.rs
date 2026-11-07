use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::p2p;

pub fn refresh_own_address(
    ready: Option<Res<p2p::ReadyChannel>>,
    mut texts: Query<&mut Text, With<direct_dial::ui::OwnAddressText>>,
) {
    let snapshot: Option<(identity::Peer, Vec<identity::PeerAddr>)> = ready.and_then(|ready| {
        let me = ready.ready_rx.borrow().as_ref().cloned()?;
        let observed: Vec<identity::PeerAddr> = ready
            .observed_rx
            .borrow()
            .as_ref()
            .map(|addrs| addrs.as_slice().to_vec())
            .unwrap_or_default();
        Some((me, observed))
    });
    let Some((me, observed)) = snapshot else {
        for mut text in &mut texts {
            *text = Text::new("Your address: waiting for network...");
        }
        return;
    };
    let dialable = identity::PeerAddr::dialable(me.addrs);
    let label = observed.iter().chain(&dialable).next().map_or_else(
        || format!("Your id: {} (listening...)", me.id.as_str()),
        |addr| format!("Your address: {addr}/p2p/{}", me.id.as_str()),
    );
    for mut text in &mut texts {
        *text = Text::new(label.clone());
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::refresh_own_address;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    fn ready_with(id: &str, addrs: Vec<&str>) -> p2p::ReadyChannel {
        let channel = p2p::ReadyChannel::default();
        channel.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from(id),
            addrs: addrs.into_iter().map(identity::PeerAddr::from).collect(),
        }));
        channel
    }

    fn labels(app: &mut App) -> Vec<String> {
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<direct_dial::ui::OwnAddressText>>();
        texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ready_with("me", vec!["/ip4/127.0.0.1/tcp/4001"]));
        app.world_mut()
            .spawn((direct_dial::ui::OwnAddressText, Text::new("old")));
        app.add_systems(Update, refresh_own_address);
        app.update();
        let shown = labels(&mut app);
        assert_eq!(shown.len(), 1);
        assert!(shown[0].contains("/p2p/me"));
    }

    #[test]
    fn waiting_label_without_a_ready_channel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut()
            .spawn((direct_dial::ui::OwnAddressText, Text::new("old")));
        app.add_systems(Update, refresh_own_address);
        app.update();
        let shown = labels(&mut app);
        assert_eq!(shown.len(), 1);
        assert!(shown[0].contains("waiting"));
    }

    fn preview_build(app: &mut App) {
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
        commands.spawn((direct_dial::ui::OwnAddressText, Text::new("old")));
    }

    const fn preview_noop(_world: &mut World) {}

    fn show_address(world: &mut World) {
        world.insert_resource(ready_with("me", vec!["/ip4/127.0.0.1/tcp/4001"]));
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
                    label: String::from("own address shown"),
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
