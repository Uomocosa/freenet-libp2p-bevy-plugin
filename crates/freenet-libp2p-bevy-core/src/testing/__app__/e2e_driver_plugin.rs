use std::net::Ipv4Addr;

use bevy::input::InputSystems;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::window::{CursorMoved, WindowEvent};

use crate::testing;

pub struct E2EDriverPlugin {
    port: u16,
    extensions: Vec<Box<dyn testing::traits::E2EMethods>>,
}

impl E2EDriverPlugin {
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            port,
            extensions: Vec::new(),
        }
    }
}

impl Plugin for E2EDriverPlugin {
    fn build(&self, app: &mut App) {
        let generic = RemotePlugin::default()
            .with_method_main("e2e/move_cursor", testing::bevy_systems::e2e_move_cursor)
            .with_method_main("e2e/click_at", testing::bevy_systems::e2e_click_at)
            .with_method_main("e2e/follow", testing::bevy_systems::e2e_follow)
            .with_method_main("e2e/type_text", testing::bevy_systems::e2e_type_text)
            .with_method_main("e2e/press_key", testing::bevy_systems::e2e_press_key)
            .with_method_main("e2e/input_idle", testing::bevy_systems::e2e_input_idle)
            .with_method_main("e2e/keys", testing::bevy_systems::e2e_keys)
            .with_method_main("e2e/exit", testing::bevy_systems::e2e_exit);
        let remote = self
            .extensions
            .iter()
            .fold(generic, |remote, extension| extension.register(remote));
        self.extensions
            .iter()
            .for_each(|extension| extension.build(app));
        app.add_plugins(remote)
            .add_plugins(
                RemoteHttpPlugin::default()
                    .with_address(Ipv4Addr::LOCALHOST)
                    .with_port(self.port),
            )
            .init_resource::<testing::SyntheticInputQueue>()
            .add_message::<CursorMoved>()
            .add_message::<WindowEvent>()
            .add_message::<MouseButtonInput>()
            .add_message::<KeyboardInput>()
            .add_systems(
                PreUpdate,
                testing::bevy_systems::drain_synthetic_input.before(InputSystems),
            );
    }
}

impl E2EDriverPlugin {
    #[must_use]
    pub fn with(mut self, extension: impl testing::traits::E2EMethods) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::input::ButtonState;
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::NativeKeyCode;
    use bevy::input::keyboard::{Key, KeyCode};
    use bevy::input_focus::{InputDispatchPlugin, InputFocus};
    use bevy::picking::events::{PointerRelease, PointerState};
    use bevy::prelude::*;
    use bevy::text::{EditableText, TextPlugin};
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::UiScale;
    use bevy::ui_widgets::{TextInput, TextInputPlugin};
    use bevy::window::{Ime, PrimaryWindow};
    use rstest::rstest;

    use super::E2EDriverPlugin;
    use crate::testing;

    #[test]
    fn test_usage() {
        let plugin = E2EDriverPlugin::new(15702)
            .with(testing::Probe)
            .with(testing::Probe);
        assert_eq!(plugin.port, 15702);
        assert_eq!(plugin.extensions.len(), 2);
    }

    #[rstest]
    #[case::typed_characters_carry_their_us_physical_key(
        Some("a1-"),
        &[],
        8,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::KeyA, logical_key: Key::Character("a".into()), state: ButtonState::Pressed, text: Some(String::from("a")) },
            testing::RecordedKeyStep { key_code: KeyCode::KeyA, logical_key: Key::Character("a".into()), state: ButtonState::Released, text: Some(String::from("a")) },
            testing::RecordedKeyStep { key_code: KeyCode::Digit1, logical_key: Key::Character("1".into()), state: ButtonState::Pressed, text: Some(String::from("1")) },
            testing::RecordedKeyStep { key_code: KeyCode::Digit1, logical_key: Key::Character("1".into()), state: ButtonState::Released, text: Some(String::from("1")) },
            testing::RecordedKeyStep { key_code: KeyCode::Minus, logical_key: Key::Character("-".into()), state: ButtonState::Pressed, text: Some(String::from("-")) },
            testing::RecordedKeyStep { key_code: KeyCode::Minus, logical_key: Key::Character("-".into()), state: ButtonState::Released, text: Some(String::from("-")) },
        ]
    )]
    #[case::uppercase_rides_in_a_shift_press_and_release(
        Some("B"),
        &[],
        6,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::ShiftLeft, logical_key: Key::Shift, state: ButtonState::Pressed, text: None },
            testing::RecordedKeyStep { key_code: KeyCode::KeyB, logical_key: Key::Character("B".into()), state: ButtonState::Pressed, text: Some(String::from("B")) },
            testing::RecordedKeyStep { key_code: KeyCode::KeyB, logical_key: Key::Character("B".into()), state: ButtonState::Released, text: Some(String::from("B")) },
            testing::RecordedKeyStep { key_code: KeyCode::ShiftLeft, logical_key: Key::Shift, state: ButtonState::Released, text: None },
        ]
    )]
    #[case::space_reports_the_space_logical_key(
        Some(" "),
        &[],
        4,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::Space, logical_key: Key::Space, state: ButtonState::Pressed, text: Some(String::from(" ")) },
            testing::RecordedKeyStep { key_code: KeyCode::Space, logical_key: Key::Space, state: ButtonState::Released, text: Some(String::from(" ")) },
        ]
    )]
    #[case::unmapped_character_keeps_unidentified_key(
        Some("é"),
        &[],
        4,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified), logical_key: Key::Character("é".into()), state: ButtonState::Pressed, text: Some(String::from("é")) },
            testing::RecordedKeyStep { key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified), logical_key: Key::Character("é".into()), state: ButtonState::Released, text: Some(String::from("é")) },
        ]
    )]
    #[case::key_press_is_visible_before_its_release(
        Some("a"),
        &[],
        1,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::KeyA, logical_key: Key::Character("a".into()), state: ButtonState::Pressed, text: Some(String::from("a")) },
        ]
    )]
    #[case::key_release_follows_press_with_the_press_text(
        Some("a"),
        &[],
        2,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::KeyA, logical_key: Key::Character("a".into()), state: ButtonState::Pressed, text: Some(String::from("a")) },
            testing::RecordedKeyStep { key_code: KeyCode::KeyA, logical_key: Key::Character("a".into()), state: ButtonState::Released, text: Some(String::from("a")) },
        ]
    )]
    #[case::named_keys_drain_as_real_key_events(
        None,
        &[testing::Key::Backspace, testing::Key::Enter, testing::Key::Tab, testing::Key::Escape],
        10,
        vec![
            testing::RecordedKeyStep { key_code: KeyCode::Backspace, logical_key: Key::Backspace, state: ButtonState::Pressed, text: Some(String::from("\u{8}")) },
            testing::RecordedKeyStep { key_code: KeyCode::Backspace, logical_key: Key::Backspace, state: ButtonState::Released, text: Some(String::from("\u{8}")) },
            testing::RecordedKeyStep { key_code: KeyCode::Enter, logical_key: Key::Enter, state: ButtonState::Pressed, text: Some(String::from("\r")) },
            testing::RecordedKeyStep { key_code: KeyCode::Enter, logical_key: Key::Enter, state: ButtonState::Released, text: Some(String::from("\r")) },
            testing::RecordedKeyStep { key_code: KeyCode::Tab, logical_key: Key::Tab, state: ButtonState::Pressed, text: Some(String::from("\t")) },
            testing::RecordedKeyStep { key_code: KeyCode::Tab, logical_key: Key::Tab, state: ButtonState::Released, text: Some(String::from("\t")) },
            testing::RecordedKeyStep { key_code: KeyCode::Escape, logical_key: Key::Escape, state: ButtonState::Pressed, text: Some(String::from("\u{1b}")) },
            testing::RecordedKeyStep { key_code: KeyCode::Escape, logical_key: Key::Escape, state: ButtonState::Released, text: Some(String::from("\u{1b}")) },
        ]
    )]
    fn synthetic_input_drains_as_real_key_events(
        #[case] text: Option<&str>,
        #[case] taps: &[testing::Key],
        #[case] frames: u32,
        #[case] expected: Vec<testing::RecordedKeyStep>,
    ) {
        let mut app = testing::e2e_driver_app();
        {
            let mut queue = app
                .world_mut()
                .resource_mut::<testing::SyntheticInputQueue>();
            if let Some(text) = text {
                queue.push_text(text);
            }
            for key in taps {
                queue.push_tap(*key);
            }
        }
        app.tick(frames);
        let seen = app.world().resource::<testing::Seen>();
        assert_eq!(seen.keys, expected);
        assert_eq!(seen.window_keys, expected);
    }

    #[test]
    fn follow_presses_before_moves_and_releases_last() {
        let mut app = testing::e2e_driver_app();
        app.world_mut()
            .resource_mut::<testing::SyntheticInputQueue>()
            .follow(&testing::PointerPath {
                button: bevy::input::mouse::MouseButton::Left,
                points: vec![Vec2::ZERO, Vec2::new(50.0, 0.0)],
                speed: testing::PointerSpeed(100.0),
            });
        app.tick(20);
        let pointer = &app.world().resource::<testing::Seen>().pointer;
        let pressed = pointer
            .iter()
            .position(|step| *step == testing::PointerStep::Button(ButtonState::Pressed));
        let released = pointer
            .iter()
            .position(|step| *step == testing::PointerStep::Button(ButtonState::Released));
        assert!(pressed.is_some_and(|index| index > 0));
        assert!(released.is_some_and(|index| Some(index) == pointer.len().checked_sub(1)));
    }

    #[test]
    fn named_escape_is_seen_once_by_button_input() {
        let mut app = testing::e2e_driver_app();
        app.add_plugins(InputPlugin);
        app.init_resource::<testing::EscapePresses>();
        app.add_systems(Update, testing::bevy_systems::count_escape_presses);
        app.world_mut()
            .resource_mut::<testing::SyntheticInputQueue>()
            .push_tap(testing::Key::Escape);
        app.tick(4);
        assert_eq!(**app.world().resource::<testing::EscapePresses>(), 1);
    }

    #[test]
    fn typed_text_inserts_into_focused_editable_text() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            InputPlugin,
            InputDispatchPlugin,
            TextPlugin,
            TextInputPlugin,
            E2EDriverPlugin::new(0),
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_message::<Ime>()
        .add_message::<PointerRelease>()
        .init_resource::<PointerState>()
        .init_resource::<UiScale>();
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        let editable = app
            .world_mut()
            .spawn((EditableText::new(""), TextInput))
            .id();
        app.insert_resource(InputFocus::from_entity(editable));
        app.world_mut()
            .resource_mut::<testing::SyntheticInputQueue>()
            .push_text("ab");
        for _ in 0..10 {
            app.update();
        }
        let typed = app
            .world()
            .entity(editable)
            .get::<EditableText>()
            .map(|text| text.value().to_string());
        assert_eq!(typed.as_deref(), Some("ab"));
    }
}
