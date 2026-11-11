use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::direct_dial;
use direct_dial::{
    COPY_TOAST_BG_COLOR, COPY_TOAST_LINGER_SECS, COPY_TOAST_TEXT_COLOR, CopyToast, OwnAddressChip,
};
use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

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
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_copy_own_address;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    fn share_with_full(full: Option<&str>) -> core_direct_dial::OwnShareAddress {
        core_direct_dial::OwnShareAddress(Some(core_direct_dial::OwnAddress {
            id: identity::PeerId::from("me"),
            short: String::from("me"),
            full: full.map(String::from),
        }))
    }

    fn app_with(full: Option<&str>, pressed: bool) -> App {
        let mut app = App::new();
        app.insert_resource(share_with_full(full));
        app.init_resource::<bevy::clipboard::Clipboard>();
        let mut chip = app.world_mut().spawn(direct_dial::OwnAddressChip);
        if pressed {
            chip.insert(Pressed);
        }
        app.add_systems(Update, handle_copy_own_address);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(Some("/ip4/127.0.0.1/tcp/4001/p2p/me"), true);
        app.update();
        let mut toasts = app
            .world_mut()
            .query_filtered::<&direct_dial::CopyToast, ()>();
        assert_eq!(toasts.iter(app.world()).count(), 1);
    }

    #[test]
    fn a_press_with_a_full_address_spawns_the_toast() {
        let mut app = app_with(Some("/ip4/127.0.0.1/tcp/4001/p2p/me"), true);
        app.update();
        let mut toasts = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::CopyToast>>();
        assert_eq!(toasts.iter(app.world()).count(), 1);
    }

    #[test]
    fn an_unpressed_chip_spawns_nothing() {
        let mut app = app_with(Some("/ip4/127.0.0.1/tcp/4001/p2p/me"), false);
        app.update();
        let mut toasts = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::CopyToast>>();
        assert_eq!(toasts.iter(app.world()).count(), 0);
    }

    #[test]
    fn without_a_full_address_nothing_is_copied() {
        let mut app = app_with(None, true);
        app.update();
        let mut toasts = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::CopyToast>>();
        assert_eq!(toasts.iter(app.world()).count(), 0);
    }

    #[test]
    fn without_the_resource_nothing_is_copied() {
        let mut app = App::new();
        app.init_resource::<bevy::clipboard::Clipboard>();
        app.world_mut()
            .spawn((direct_dial::OwnAddressChip, Pressed));
        app.add_systems(Update, handle_copy_own_address);
        app.update();
        let mut toasts = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::CopyToast>>();
        assert_eq!(toasts.iter(app.world()).count(), 0);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(share_with_full(Some("/ip4/127.0.0.1/tcp/4001/p2p/me")));
        app.init_resource::<bevy::clipboard::Clipboard>();
        app.add_systems(Startup, spawn_chip);
        app.add_systems(Update, handle_copy_own_address);
    }

    fn spawn_chip(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            direct_dial::OwnAddressChip,
            bevy::ui_widgets::Button,
            bevy::picking::hover::Hovered::default(),
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![Text::new("12D3Koo...99RC   copy")],
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn press_chip(world: &mut World) {
        let mut chips = world.query_filtered::<Entity, With<direct_dial::OwnAddressChip>>();
        let chip = chips.single(world).expect("chip");
        world.entity_mut(chip).insert(Pressed);
    }

    #[test]
    fn handle_copy_own_address_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("handle_copy_own_address"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("before the click"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("after the click"),
                    apply: press_chip,
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
