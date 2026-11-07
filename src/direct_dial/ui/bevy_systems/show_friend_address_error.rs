use bevy::prelude::*;

use crate::direct_dial;

pub fn show_friend_address_error(
    error: Res<direct_dial::FriendAddressError>,
    mut texts: Query<&mut Text, With<direct_dial::ui::FriendErrorText>>,
) {
    let error = error.into_inner();
    let label: String = (**error).clone().unwrap_or_default();
    for mut text in &mut texts {
        *text = Text::new(label.clone());
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::show_friend_address_error;
    use crate::direct_dial;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(direct_dial::FriendAddressError(Some(String::from(
            "bad address",
        ))));
        app.world_mut()
            .spawn((direct_dial::ui::FriendErrorText, Text::new("")));
        app.add_systems(Update, show_friend_address_error);
        app.update();
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<direct_dial::ui::FriendErrorText>>();
        let shown: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert_eq!(shown, vec![String::from("bad address")]);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(direct_dial::FriendAddressError::default());
        app.add_systems(
            Startup,
            (
                spawn_error_text,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
        app.add_systems(Update, show_friend_address_error);
    }

    fn spawn_error_text(mut commands: Commands) {
        commands.spawn((
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(discovery::ui::basic::constants::PANEL_COLOR),
            children![(direct_dial::ui::FriendErrorText, Text::new(""),)],
        ));
    }

    fn show_bad_address(world: &mut World) {
        **world.resource_mut::<direct_dial::FriendAddressError>() = Some(String::from(
            "bad address: use /ip4/<host>/tcp/<port>/p2p/<id>",
        ));
    }

    fn show_duplicate(world: &mut World) {
        **world.resource_mut::<direct_dial::FriendAddressError>() =
            Some(String::from("already a friend"));
    }

    #[test]
    fn show_friend_address_error_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_friend_address_error"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("bad address"),
                    apply: show_bad_address,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("already a friend"),
                    apply: show_duplicate,
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
