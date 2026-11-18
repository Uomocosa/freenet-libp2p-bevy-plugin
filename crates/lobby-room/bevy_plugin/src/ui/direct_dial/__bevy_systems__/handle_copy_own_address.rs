use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
use ui::direct_dial::{
    COPY_TOAST_BG_COLOR, COPY_TOAST_LINGER_SECS, COPY_TOAST_TEXT_COLOR, CopyToast, OwnAddressChip,
};

pub fn handle_copy_own_address(
    presses: Query<(), (Added<Pressed>, With<OwnAddressChip>)>,
    share: Option<Res<core_direct_dial::OwnShareAddress>>,
    clipboard: Option<ResMut<bevy::clipboard::Clipboard>>,
    mut commands: Commands,
) {
    if presses.is_empty() {
        return;
    }
    let Some(address) = share.map(|share| (**share).clone()) else {
        return;
    };
    let Some(full) = address.and_then(|address| address.full) else {
        return;
    };
    if let Some(mut clipboard) = clipboard
        && clipboard.set_text(full).is_err()
    {
        tracing::warn!(target: "direct_dial", "own address copy to the clipboard failed");
    }
    commands.spawn((
        CopyToast(COPY_TOAST_LINGER_SECS),
        Text::new("Copied to clipboard"),
        TextColor(COPY_TOAST_TEXT_COLOR),
        BackgroundColor(COPY_TOAST_BG_COLOR),
        UiTransform {
            translation: Val2 {
                x: Val::Percent(-50.0),
                y: Val::ZERO,
            },
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(28.0),
            left: Val::Percent(50.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
    ));
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_copy_own_address;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn handle_copy_own_address_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("handle_copy_own_address"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_direct_dial::OwnShareAddress(Some(
                    core_direct_dial::OwnAddress {
                        id: identity::PeerId::from("me"),
                        short: String::from("me"),
                        full: Some(String::from("/ip4/127.0.0.1/tcp/4001/p2p/me")),
                    },
                )));
                app.init_resource::<bevy::clipboard::Clipboard>();
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.spawn((
                        ChildOf(root),
                        ui::direct_dial::OwnAddressChip,
                        bevy::ui_widgets::Button,
                        bevy::picking::hover::Hovered::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::BLACK),
                        children![Text::new("12D3Koo...99RC   copy")],
                    ));
                });
                app.add_systems(Update, handle_copy_own_address);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("before the click"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("after the click"),
                    apply: |world: &mut World| {
                        let mut chips =
                            world.query_filtered::<Entity, With<ui::direct_dial::OwnAddressChip>>();
                        if let Ok(chip) = chips.single(world) {
                            world.entity_mut(chip).insert(Pressed);
                        }
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
