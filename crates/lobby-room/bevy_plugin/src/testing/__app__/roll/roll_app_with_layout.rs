use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::ui;

pub fn roll_app_with_layout(style: ui::roll::NumberStyle) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::window::WindowPlugin>()
            .disable::<bevy::a11y::AccessibilityPlugin>(),
    );
    app.add_message::<bevy::window::WindowEvent>()
        .add_message::<bevy::window::WindowResized>()
        .add_message::<bevy::window::WindowCreated>()
        .add_message::<bevy::window::WindowClosing>()
        .add_message::<bevy::window::WindowClosed>()
        .add_message::<bevy::window::WindowCloseRequested>()
        .add_message::<bevy::window::WindowDestroyed>()
        .add_message::<bevy::window::WindowFocused>()
        .add_message::<bevy::window::WindowOccluded>()
        .add_message::<bevy::window::WindowScaleFactorChanged>()
        .add_message::<bevy::window::WindowBackendScaleFactorChanged>()
        .add_message::<bevy::window::WindowMoved>()
        .add_message::<bevy::window::WindowThemeChanged>()
        .add_message::<bevy::window::Ime>();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_nanos(
        33_333_333,
    )));
    app.add_systems(Startup, move |mut commands: Commands| {
        commands.spawn((
            Camera2d,
            Camera {
                computed: bevy::camera::ComputedCameraValues {
                    target_info: Some(bevy::camera::RenderTargetInfo {
                        physical_size: UVec2::new(800, 600),
                        scale_factor: 1.0,
                    }),
                    ..default()
                },
                ..default()
            },
            bevy::ui::IsDefaultUiCamera,
        ));
        let root = commands
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .id();
        commands.spawn((
            ChildOf(root),
            Text::new("8"),
            style.text_font(),
            style.line_height(),
            TextColor(Color::WHITE),
            TextLayout::no_wrap(),
        ));
        let pill = commands
            .spawn((
                ChildOf(root),
                ui::roll::RollMachine(ui::roll::State::new(vec![9], None)),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .id();
        commands.entity(pill).with_children(|parent| {
            ui::roll::bevy_systems::spawn_roll_number(&mut parent.reborrow(), pill, 0, 9, style);
        });
    });
    app.add_plugins(ui::roll::UiPlugin);
    app.finish();
    app.cleanup();
    app.update();
    app
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::ui;

    use super::roll_app_with_layout;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 13.0,
            text_color: Color::WHITE,
        };
        let mut app = roll_app_with_layout(style);
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        assert_eq!(numbers.iter(app.world()).count(), 1);
    }
}
