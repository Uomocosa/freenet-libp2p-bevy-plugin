use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::{InteractionDisabled, Pressed};

use crate::discovery;
use discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR,
};
use discovery::{CreateRoomButton, RoomNameInput};

type CreateButtonParts<'a> = (
    Entity,
    Option<&'a Hovered>,
    Has<InteractionDisabled>,
    &'a mut BackgroundColor,
    Option<&'a Children>,
);

pub fn gate_create(
    input: Single<&EditableText, With<RoomNameInput>>,
    button: Single<CreateButtonParts, With<CreateRoomButton>>,
    mut labels: Query<&mut TextColor>,
    mut commands: Commands,
) {
    let typed = input.into_inner().value().to_string();
    let enabled = !typed.trim().is_empty();
    let (entity, hovered, disabled, mut background, children) = button.into_inner();
    let BackgroundColor(existing) = &mut *background;
    let (color, label_color) = if enabled {
        (BUTTON_COLOR, BUTTON_LABEL_COLOR)
    } else {
        (DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR)
    };
    if *existing != color {
        *existing = color;
    }
    let mut label_iter = labels
        .iter_many_mut(children.into_iter().flatten())
        .matched();
    while let Some(mut label) = label_iter.fetch_next() {
        let TextColor(current) = &mut *label;
        if *current != label_color {
            *current = label_color;
        }
    }
    match (enabled, hovered.is_some()) {
        (true, false) => {
            commands.entity(entity).insert(Hovered(false));
        }
        (false, true) => {
            commands
                .entity(entity)
                .remove::<Hovered>()
                .remove::<Pressed>();
        }
        _ => {}
    }
    match (enabled, disabled) {
        (true, true) => {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
        (false, false) => {
            commands.entity(entity).insert(InteractionDisabled);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::picking::backend::HitData;
    use bevy::picking::events::{Pointer, PointerPress};
    use bevy::picking::hover::Hovered;
    use bevy::picking::pointer::{Location, PointerButton, PointerId};
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui::{InteractionDisabled, Pressed};
    use bevy::ui_widgets::{Button, ButtonPlugin, TextInput};
    use bevy::window::{Window, WindowRef};

    use super::gate_create;

    fn app_with(typed: &str) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut()
            .spawn((discovery::RoomNameInput, EditableText::new(typed)));
        app.world_mut().spawn((
            discovery::CreateRoomButton,
            Hovered(false),
            BackgroundColor(Color::BLACK),
        ));
        app.add_systems(Update, gate_create);
        app
    }

    fn clickable(app: &mut App) -> usize {
        let mut query = app
            .world_mut()
            .query_filtered::<(), (With<discovery::CreateRoomButton>, With<Hovered>)>();
        query.iter(app.world()).count()
    }

    fn disabled(app: &mut App) -> usize {
        let mut query = app
            .world_mut()
            .query_filtered::<(), (With<discovery::CreateRoomButton>, With<InteractionDisabled>)>();
        query.iter(app.world()).count()
    }

    #[test]
    fn test_usage() {
        let mut named = app_with("alpha");
        named.update();
        assert_eq!(clickable(&mut named), 1);

        let mut blank = app_with("   ");
        blank.update();
        assert_eq!(clickable(&mut blank), 0);
    }

    #[test]
    fn disabled_button_carries_interaction_disabled() {
        let mut named = app_with("alpha");
        named.update();
        assert_eq!(disabled(&mut named), 0);

        let mut blank = app_with("   ");
        blank.update();
        assert_eq!(disabled(&mut blank), 1);
        assert_eq!(clickable(&mut blank), 0);
    }

    #[test]
    fn disabled_button_ignores_pointer_press() {
        let mut app = App::new();
        app.add_plugins(ButtonPlugin);
        let window = app.world_mut().spawn(Window::default()).id();
        let location = Location {
            target: NormalizedRenderTarget::Window(
                WindowRef::Entity(window)
                    .normalize(Some(window))
                    .expect("test window is a valid target"),
            ),
            position: Vec2::ZERO,
        };
        let disabled = app
            .world_mut()
            .spawn((
                discovery::CreateRoomButton,
                Button,
                InteractionDisabled,
                Hovered(false),
            ))
            .id();
        let enabled = app
            .world_mut()
            .spawn((discovery::CreateRoomButton, Button, Hovered(false)))
            .id();
        app.update();
        for target in [disabled, enabled] {
            app.world_mut().trigger(PointerPress {
                entity: target,
                pointer: Pointer::new(PointerId::Mouse, location.clone()),
                button: PointerButton::Primary,
                hit: HitData::new(window, 0.0, None, None),
                count: 1,
            });
        }
        app.update();
        assert!(
            !app.world().entity(disabled).contains::<Pressed>(),
            "disabled button must not gain Pressed"
        );
        assert!(
            app.world().entity(enabled).contains::<Pressed>(),
            "enabled button must gain Pressed"
        );
    }

    fn spawn_button(mut commands: Commands) {
        commands.spawn((discovery::RoomNameInput, EditableText::new(""), TextInput));
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            discovery::CreateRoomButton,
            Button,
            Hovered::default(),
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![Text::new("Create New Room")],
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_button);
        app.add_systems(Update, gate_create);
    }

    const fn preview_noop(_world: &mut World) {}

    fn type_name(world: &mut World) {
        let mut inputs =
            world.query_filtered::<&mut EditableText, With<discovery::RoomNameInput>>();
        if let Ok(mut input) = inputs.single_mut(world) {
            *input = EditableText::new("alpha");
        }
    }

    #[test]
    fn gate_create_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("gate_create"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("blank name"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("typed name"),
                    apply: type_name,
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
