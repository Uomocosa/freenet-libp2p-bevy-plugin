use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey, NativeKeyCode};
use bevy::input::mouse::{MouseButton, MouseButtonInput};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::window::{PrimaryWindow, WindowEvent};
use freenet_libp2p_bevy_plugin::{direct_dial, identity, ui};

use ui::direct_dial::{AddFriendButton, FriendAddressInput};

const TYPE_RETRY_FRAMES: u32 = 60;
const CLICK_RETRY_FRAMES: u32 = 60;
const MAX_CLICKS: u32 = 60;

/// One scripted friend add, driven through real Bevy input.
///
/// The script runner arms the intent with one address at a time; the system
/// opens the friends drawer with a real Tab key press, focuses the drawer
/// input, types the address, clicks the real add button, and records the
/// address once the friend book holds the new id. Typing retries clear the
/// field first so a retry never appends twice.
#[derive(Resource, Default)]
pub struct FriendIntent {
    active: Option<FriendTyping>,
    done: Vec<String>,
}

pub struct FriendTyping {
    addr: String,
    step: FriendStep,
    frames: u32,
    clicks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FriendStep {
    Open,
    WaitOpen,
    Type,
    WaitTyped,
    Click,
    Release,
    WaitBooked,
}

impl FriendIntent {
    #[must_use]
    pub fn is_done(&self, addr: &str) -> bool {
        self.done.iter().any(|done| done == addr)
    }

    pub fn arm(&mut self, addr: &str) {
        if self.is_done(addr) {
            return;
        }
        if self
            .active
            .as_ref()
            .is_none_or(|typing| typing.addr != addr)
        {
            self.active = Some(FriendTyping {
                addr: addr.to_string(),
                step: FriendStep::Open,
                frames: 0,
                clicks: 0,
            });
        }
    }

    pub(crate) fn finish_active(&mut self) {
        if let Some(typing) = self.active.take() {
            self.done.push(typing.addr);
        }
    }
}

impl FriendIntent {
    pub(crate) fn press_tab(window: Entity, keyboard: &mut MessageWriter<KeyboardInput>) {
        for press in [ButtonState::Pressed, ButtonState::Released] {
            keyboard.write(KeyboardInput {
                key_code: KeyCode::Tab,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state: press,
                text: None,
                repeat: false,
                window,
            });
        }
    }

    pub(crate) fn type_address(
        field: Entity,
        window: Entity,
        addr: &str,
        focus: &mut InputFocus,
        keyboard: &mut MessageWriter<KeyboardInput>,
    ) {
        for ch in addr.chars() {
            let text = ch.to_string();
            keyboard.write(KeyboardInput {
                key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                logical_key: Key::Character(text.clone().into()),
                state: ButtonState::Pressed,
                text: Some(text.into()),
                repeat: false,
                window,
            });
        }
        focus.set(field, FocusCause::Pressed);
    }

    pub(crate) fn booked_or_retry(
        typing: &mut FriendTyping,
        state: &direct_dial::state_machine::DialState,
    ) -> bool {
        let booked = identity::Peer::try_from(typing.addr.as_str())
            .is_ok_and(|peer| state.is_friend(&peer.id));
        if booked {
            return true;
        }
        typing.frames = typing.frames.saturating_add(1);
        if typing.frames >= CLICK_RETRY_FRAMES && typing.clicks >= MAX_CLICKS {
            return false;
        }
        if typing.frames >= CLICK_RETRY_FRAMES {
            typing.step = FriendStep::Click;
        }
        true
    }
}

pub fn drive_friend_intent(
    mut intent: ResMut<FriendIntent>,
    drawer: Option<Res<ui::direct_dial::FriendsDrawerState>>,
    mut fields: Query<(Entity, &mut EditableText), With<FriendAddressInput>>,
    buttons: Query<&UiGlobalTransform, (With<AddFriendButton>, With<Hovered>)>,
    state: Res<direct_dial::state_machine::DialState>,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut keyboard: MessageWriter<KeyboardInput>,
    mut window_clicks: MessageWriter<WindowEvent>,
    mut focus: ResMut<InputFocus>,
) {
    let state = state.into_inner();
    let drawer_open = drawer.is_none_or(|drawer| **drawer);
    let mut finished = false;
    let mut abandoned = false;
    if let Some(typing) = intent.active.as_mut() {
        match typing.step {
            FriendStep::Open => {
                let Ok((window_entity, _)) = windows.single() else {
                    return;
                };
                FriendIntent::press_tab(window_entity, &mut keyboard);
                typing.step = FriendStep::WaitOpen;
                typing.frames = 0;
            }
            FriendStep::WaitOpen => {
                if drawer_open {
                    typing.step = FriendStep::Type;
                    typing.frames = 0;
                } else {
                    typing.frames = typing.frames.saturating_add(1);
                    if typing.frames >= TYPE_RETRY_FRAMES {
                        typing.step = FriendStep::Open;
                    }
                }
            }
            FriendStep::Type => {
                let Ok((field, mut input)) = fields.single_mut() else {
                    return;
                };
                let Ok((window_entity, _)) = windows.single() else {
                    return;
                };
                *input = EditableText::new(String::new());
                FriendIntent::type_address(
                    field,
                    window_entity,
                    &typing.addr,
                    &mut focus,
                    &mut keyboard,
                );
                typing.step = FriendStep::WaitTyped;
                typing.frames = 0;
            }
            FriendStep::WaitTyped => {
                let Ok((_, input)) = fields.single() else {
                    return;
                };
                if input.value() == typing.addr.as_str() {
                    typing.step = FriendStep::Click;
                    typing.frames = 0;
                } else {
                    typing.frames = typing.frames.saturating_add(1);
                    if typing.frames >= TYPE_RETRY_FRAMES {
                        typing.step = FriendStep::Type;
                    }
                }
            }
            FriendStep::Click => {
                let Ok(transform) = buttons.single() else {
                    return;
                };
                let Ok((window_entity, mut window)) = windows.single_mut() else {
                    return;
                };
                window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
                window_clicks.write(WindowEvent::MouseButtonInput(MouseButtonInput {
                    button: MouseButton::Left,
                    state: ButtonState::Pressed,
                    window: window_entity,
                }));
                typing.step = FriendStep::Release;
            }
            FriendStep::Release => {
                let Ok((window_entity, _)) = windows.single() else {
                    return;
                };
                window_clicks.write(WindowEvent::MouseButtonInput(MouseButtonInput {
                    button: MouseButton::Left,
                    state: ButtonState::Released,
                    window: window_entity,
                }));
                typing.clicks = typing.clicks.saturating_add(1);
                typing.step = FriendStep::WaitBooked;
                typing.frames = 0;
            }
            FriendStep::WaitBooked => {
                if FriendIntent::booked_or_retry(typing, state) {
                    finished = true;
                } else {
                    abandoned = true;
                }
            }
        }
    }
    if finished {
        intent.finish_active();
    } else if abandoned {
        intent.active = None;
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::keyboard::KeyboardInput;
    use bevy::input::mouse::MouseButtonInput;
    use bevy::input_focus::InputFocus;
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::window::{PrimaryWindow, WindowEvent};
    use freenet_libp2p_bevy_plugin::direct_dial;
    use freenet_libp2p_bevy_plugin::ui;

    use super::{FriendIntent, FriendStep, drive_friend_intent};

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::input::InputPlugin);
        app.add_message::<KeyboardInput>();
        app.add_message::<MouseButtonInput>();
        app.add_message::<WindowEvent>();
        app.init_resource::<InputFocus>();
        app.insert_resource(direct_dial::state_machine::DialState::default());
        app.insert_resource(FriendIntent::default());
        app.insert_resource(ui::direct_dial::FriendsDrawerState::default());
        app.insert_resource(ui::discovery::EscMenuState::default());
        app.insert_resource(ui::discovery::FriendsPressBeat::default());
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app.world_mut()
            .spawn((ui::direct_dial::FriendAddressInput, EditableText::new("")));
        app.add_systems(
            Update,
            (
                drive_friend_intent,
                ui::discovery::bevy_systems::handle_menu_keys,
            ),
        );
        app
    }

    #[test]
    fn test_usage() {
        let mut intent = FriendIntent::default();
        assert!(!intent.is_done("addr"));
        intent.arm("addr");
        intent.finish_active();
        assert!(intent.is_done("addr"));
        intent.arm("addr");
        assert!(intent.active.is_none());
    }

    #[test]
    fn typing_opens_the_drawer_first_and_sends_one_key_event_per_char() {
        let mut app = app();
        app.world_mut().resource_mut::<FriendIntent>().arm("ab");
        app.update();
        let intent = app.world().resource::<FriendIntent>();
        assert_eq!(
            intent.active.as_ref().map(|typing| typing.step),
            Some(FriendStep::WaitOpen),
            "the drawer is opened before typing"
        );
        app.update();
        assert!(
            **app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>(),
            "the Tab press opened the drawer"
        );
        app.update();
        let keys = app.world().resource::<Messages<KeyboardInput>>();
        assert_eq!(keys.len(), 4, "one Tab press+release and two characters");
        let intent = app.world().resource::<FriendIntent>();
        assert_eq!(
            intent.active.as_ref().map(|typing| typing.step),
            Some(FriendStep::WaitTyped)
        );
    }
}
