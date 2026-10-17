use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultUiPlugin;

#[atomic_delegates]
impl DefaultUiPlugin {
    pub fn build_plugin(&self, app: &mut App) {}
    pub fn finish_plugin(&self, app: &mut App) {}
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for DefaultUiPlugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
    fn finish(&self, app: &mut App) {
        self.finish_plugin(app);
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use bevy::prelude::*;

    use crate::discovery;

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(discovery::ui::DefaultUiPlugin);
        app.add_systems(
            Update,
            pin_hover.before(discovery::ui::bevy_systems::animate_hover),
        );
    }

    fn preview_snapshot() -> discovery::Snapshot {
        let mut snapshot = discovery::Snapshot {
            freenet_peers: (1..=3_u16)
                .map(|port| identity::FreenetPeer {
                    id: identity::FreenetPeerId::from(format!("peer-{port}").as_str()),
                    addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
                })
                .collect(),
            ..Default::default()
        };
        for (name, room_size) in [("alpha", 8), ("beta", 4)] {
            snapshot.lobby.insert(
                identity::RoomName::from(name),
                discovery::Room {
                    room_size: discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        snapshot
    }

    const fn preview_noop(_world: &mut World) {}

    #[derive(Component)]
    struct PinHover;

    fn pin_hover(mut pinned: Query<&mut Interaction, With<PinHover>>) {
        for mut interaction in &mut pinned {
            *interaction = Interaction::Hovered;
        }
    }

    fn hover_all(world: &mut World) {
        let mut inputs = world
            .query_filtered::<&mut bevy::text::EditableText, With<discovery::ui::RoomNameInput>>();
        if let Ok(mut input) = inputs.single_mut(world) {
            input.queue_edit(bevy::text::TextEdit::Insert("gamma".into()));
        }
        let mut glows = world.query_filtered::<Entity, With<discovery::ui::HoverGlow>>();
        let hovered: Vec<Entity> = glows.iter(world).collect();
        for entity in hovered {
            world.entity_mut(entity).insert((
                Interaction::Hovered,
                PinHover,
                discovery::ui::HoverGlow(1.0),
            ));
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_message::<discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(discovery::ui::DefaultUiPlugin);
        app.finish();
        for _ in 0..3 {
            app.update();
        }
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::CreateRoomButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }

    #[test]
    fn default_ui_plugin_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("default_ui_plugin"),
                kind: lele_bevy_preview::scene::Kind::Plugin,
                build: preview_build,
                states: vec![
                    lele_bevy_preview::scene::State {
                        label: String::from("lobby with two rooms"),
                        apply: preview_noop,
                    },
                    lele_bevy_preview::scene::State {
                        label: String::from("all hovered"),
                        apply: hover_all,
                    },
                ],
                timeline: None,
            },
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
