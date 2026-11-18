use crate::ui;
use bevy::prelude::*;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

pub fn show_friend_address_error(
    error: Res<core_direct_dial::FriendAddressError>,
    mut texts: Query<&mut Text, With<ui::direct_dial::FriendErrorText>>,
) {
    let error = error.into_inner();
    let label: String = (**error).clone().unwrap_or_default();
    for mut text in &mut texts {
        *text = Text::new(label.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;

    use super::show_friend_address_error;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

    #[test]
    fn show_friend_address_error_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_friend_address_error"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_direct_dial::FriendAddressError::default());
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
                            commands.spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(ui::discovery::PANEL_COLOR),
                                children![(ui::direct_dial::FriendErrorText, Text::new(""),)],
                            ));
                        },
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
                app.add_systems(Update, show_friend_address_error);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("bad address"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<core_direct_dial::FriendAddressError>() = Some(
                            String::from("bad address: use /ip4/<host>/tcp/<port>/p2p/<id>"),
                        );
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("already a friend"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<core_direct_dial::FriendAddressError>() =
                            Some(String::from("already a friend"));
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
