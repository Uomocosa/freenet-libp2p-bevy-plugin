#![allow(clippy::needless_pass_by_value)]
use std::ops::Deref;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKeyCode};
use bevy::input::mouse::MouseButtonInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::{PrimaryWindow, WindowEvent};
use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::testing::{Action, PendingAction};

/// One scripted interaction, driven through **real** Bevy input.
///
/// `create:<room>` focuses the room-name field, types the name with
/// [`KeyboardInput`] events, and waits for the Create button to become
/// clickable; `join:<room>` waits for that room's button to appear. Either way
/// the cursor is moved onto that button and pressed with a real pointer click
/// (window cursor + a window mouse-button event) only once picking really
/// hovers it — a stale hover would press whatever row the pointer used to be
/// over — and the click is retried, after a cooldown, until the
/// room is actually joined — the cooldown avoids toggling back out, since a
/// second press on the joined room is a `Leave`.
#[derive(Resource)]
pub struct Intent {
    action: Option<Action>,
    step: Step,
    retries: u32,
    clicks: u32,
    cooldown: u32,
}

impl Intent {
    #[must_use]
    pub const fn new(action: Option<Action>) -> Self {
        Self {
            action,
            step: Step::Type,
            retries: 0,
            clicks: 0,
            cooldown: 0,
        }
    }

    /// Arms the intent with a new action and restarts its step machine, so the
    /// scripted session can create a room, join it, and join it again later.
    pub fn arm_with_action(&mut self, action: Action) {
        self.action = Some(action);
        self.step = Step::Type;
        self.retries = 0;
        self.clicks = 0;
        self.cooldown = 0;
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Step {
    /// Type the room name (create only), then hand off to `Enable`.
    #[default]
    Type,
    /// Wait until the Create button accepts input (non-empty name).
    Enable,
    /// Move the cursor onto the button and press it.
    Click,
    /// Release the mouse button.
    Release,
    /// Wait until the room is joined; retry the click if it was dropped.
    Verify,
    Done,
}

const TYPE_RETRY_FRAMES: u32 = 60;
const CLICK_COOLDOWN: u32 = 60;
const MAX_CLICKS: u32 = 60;

pub fn script_input<F, C, J>(
    mut intent: ResMut<Intent>,
    mut pending: Option<ResMut<PendingAction>>,
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    fields: Query<Entity, With<F>>,
    create: Query<(&UiGlobalTransform, &Hovered), With<C>>,
    join: Query<(&UiGlobalTransform, &Hovered, &J)>,
    mut keyboard: MessageWriter<KeyboardInput>,
    mut window_clicks: MessageWriter<WindowEvent>,
    mut focus: ResMut<InputFocus>,
) where
    F: Component,
    C: Component,
    J: Component + Deref<Target = identity::RoomName>,
{
    if let Some(pending) = pending.as_mut() {
        arm_with_pending(&mut intent, pending);
    }
    let Some(action) = intent.action.clone() else {
        return;
    };
    let Ok((window_entity, mut window)) = windows.single_mut() else {
        return;
    };
    match intent.step {
        Step::Type => {
            let Action::Create(name) = &action else {
                intent.step = Step::Click;
                return;
            };
            let Ok(field) = fields.single() else {
                return;
            };
            focus.set(field, FocusCause::Pressed);
            for ch in name.as_str().chars() {
                let text = ch.to_string();
                keyboard.write(KeyboardInput {
                    key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                    logical_key: Key::Character(text.clone().into()),
                    state: ButtonState::Pressed,
                    text: Some(text.into()),
                    repeat: false,
                    window: window_entity,
                });
            }
            tracing::info!("lobby intent typing room={}", name.as_str());
            intent.step = Step::Enable;
        }
        Step::Enable => {
            let Action::Create(_) = &action else {
                intent.step = Step::Click;
                return;
            };
            if create.iter().next().is_some() {
                intent.step = Step::Click;
            } else {
                intent.retries = intent.retries.saturating_add(1);
                if intent.retries >= TYPE_RETRY_FRAMES {
                    intent.retries = 0;
                    intent.step = Step::Type;
                }
            }
        }
        Step::Click => {
            let Some((transform, hovered)) = click_aim(&action, &create, &join) else {
                return;
            };
            window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
            if !hovered.get() {
                return;
            }
            if intent.clicks == 0 {
                tracing::info!("lobby intent pressed {action:?}");
            }
            intent.cooldown = 0;
            // Picking converts only per-window mouse-button events into
            // pointer presses; a bare MouseButtonInput message never reaches
            // a Bevy 0.20 button.
            window_clicks.write(WindowEvent::MouseButtonInput(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: window_entity,
            }));
            intent.step = Step::Release;
        }
        Step::Release => {
            window_clicks.write(WindowEvent::MouseButtonInput(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
                window: window_entity,
            }));
            intent.clicks = intent.clicks.saturating_add(1);
            intent.step = Step::Verify;
        }
        Step::Verify => {
            let target = match &action {
                Action::Create(room) | Action::Join(room) => room,
            };
            let joined = state
                .as_deref()
                .and_then(|state| state.room.as_ref())
                .is_some_and(|room| room.name == *target);
            if joined || intent.clicks >= MAX_CLICKS {
                intent.step = Step::Done;
            } else {
                intent.cooldown = intent.cooldown.saturating_add(1);
                if intent.cooldown >= CLICK_COOLDOWN {
                    intent.cooldown = 0;
                    intent.step = Step::Click;
                }
            }
        }
        Step::Done => {}
    }
}

// needed helper: finds the button one scripted click aims at — the create button,
// or the join row whose room name matches the intent
fn click_aim<'a, C, J>(
    action: &Action,
    create: &'a Query<(&UiGlobalTransform, &Hovered), With<C>>,
    join: &'a Query<(&UiGlobalTransform, &Hovered, &J)>,
) -> Option<(&'a UiGlobalTransform, &'a Hovered)>
where
    C: Component,
    J: Component + Deref<Target = identity::RoomName>,
{
    match action {
        Action::Create(_) => create.iter().next(),
        Action::Join(room) => join
            .iter()
            .find(|(_, _, button)| ***button == *room)
            .map(|(transform, hovered, _)| (transform, hovered)),
    }
}

// needed helper: arms the intent with the pending action, consuming it so a repeated
// Join re-arms the intent and clicks again
fn arm_with_pending(intent: &mut Intent, pending: &mut PendingAction) {
    if let Some(action) = pending.0.take() {
        intent.arm_with_action(action);
    }
}

pub fn log_joined(intent: Res<Intent>, mut events: MessageReader<discovery::Event>) {
    for event in events.read() {
        if let discovery::Event::Joined(room) = event {
            match &intent.action {
                Some(Action::Create(_)) => {
                    tracing::info!("lobby created room={}", room.as_str());
                }
                Some(Action::Join(_)) => {
                    tracing::info!("lobby joined room={}", room.as_str());
                }
                None => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::KeyboardInput;
    use bevy::input::mouse::{MouseButton, MouseButtonInput};
    use bevy::input_focus::InputFocus;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::ui::UiGlobalTransform;
    use bevy::window::{PrimaryWindow, Window, WindowEvent};
    use derive_more::Deref;

    use freenet_libp2p_bevy_plugin::identity;

    use super::{Intent, arm_with_pending, script_input};
    use freenet_libp2p_bevy_plugin::testing::{Action, PendingAction};

    fn join(room: &str) -> Action {
        Action::Join(identity::RoomName::from(room))
    }

    fn create(room: &str) -> Action {
        Action::Create(identity::RoomName::from(room))
    }

    #[derive(Component)]
    struct TestField;

    #[derive(Component)]
    struct TestCreateButton;

    #[derive(Component, Deref)]
    struct TestJoinButton(identity::RoomName);

    fn window_clicks(app: &App) -> Vec<WindowEvent> {
        let messages = app.world().resource::<Messages<WindowEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    /// Drives one [`Step::Click`] through [`Step::Release`]: the synthetic
    /// click must reach picking as a window mouse-button event, or no Bevy
    /// 0.20 button (which listens for `Pressed` from `PointerPress`) fires.
    #[test]
    fn a_scripted_click_sends_window_mouse_button_press_and_release() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<KeyboardInput>();
        app.add_message::<MouseButtonInput>();
        app.add_message::<WindowEvent>();

        app.init_resource::<InputFocus>();
        app.insert_resource(Intent::new(Some(create("room-1"))));
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.world_mut().spawn((
            TestCreateButton,
            Hovered(true),
            UiGlobalTransform::default(),
        ));
        app.world_mut().resource_mut::<Intent>().step = super::Step::Click;
        app.add_systems(
            Update,
            script_input::<TestField, TestCreateButton, TestJoinButton>,
        );
        app.update();
        let pressed = window_clicks(&app);
        assert!(
            pressed.iter().any(|event| matches!(
                event,
                WindowEvent::MouseButtonInput(MouseButtonInput {
                    button: MouseButton::Left,
                    state: ButtonState::Pressed,
                    window: event_window,
                }) if *event_window == window
            )),
            "a scripted click must press the window mouse button, sent {pressed:?}"
        );
        app.update();
        let released = window_clicks(&app);
        assert!(
            released.iter().any(|event| matches!(
                event,
                WindowEvent::MouseButtonInput(MouseButtonInput {
                    button: MouseButton::Left,
                    state: ButtonState::Released,
                    window: event_window,
                }) if *event_window == window
            )),
            "releasing a scripted click must release the window mouse button, sent {released:?}"
        );
    }

    /// A join click must wait until picking really hovers the named room: the
    /// pointer may rest on another row while the script aims at its room, and
    /// pressing then joins a stranger's room.
    #[test]
    fn a_join_click_waits_for_the_target_room_to_be_really_hovered() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<KeyboardInput>();
        app.add_message::<MouseButtonInput>();
        app.add_message::<WindowEvent>();

        app.init_resource::<InputFocus>();
        let alpha = identity::RoomName::from("alpha");
        let beta = identity::RoomName::from("beta");
        app.insert_resource(Intent::new(Some(join(beta.as_str()))));
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app.world_mut().spawn((
            TestJoinButton(alpha),
            Hovered(true),
            UiGlobalTransform::default(),
        ));
        app.world_mut().spawn((
            TestJoinButton(beta.clone()),
            Hovered(false),
            UiGlobalTransform::default(),
        ));
        app.add_systems(
            Update,
            script_input::<TestField, TestCreateButton, TestJoinButton>,
        );
        app.update();
        app.update();
        assert!(
            window_clicks(&app).is_empty(),
            "a join click must not press while another row is really hovered"
        );
        let beta_row = app
            .world_mut()
            .query_filtered::<Entity, (With<TestJoinButton>,)>()
            .iter(app.world())
            .find(|row| {
                app.world()
                    .get::<TestJoinButton>(*row)
                    .is_some_and(|button| button.0 == beta)
            });
        let Some(beta_row) = beta_row else {
            panic!("beta row exists");
        };
        app.world_mut().entity_mut(beta_row).insert(Hovered(true));
        app.update();
        assert!(
            window_clicks(&app).iter().any(|event| matches!(
                event,
                WindowEvent::MouseButtonInput(MouseButtonInput {
                    button: MouseButton::Left,
                    state: ButtonState::Pressed,
                    ..
                })
            )),
            "once the target room is really hovered the join click presses"
        );
    }

    #[test]
    fn test_usage() {
        let mut intent = Intent::new(None);
        let mut pending = PendingAction(Some(join("room-1")));
        arm_with_pending(&mut intent, &mut pending);
        assert!(pending.0.is_none());
    }

    #[test]
    fn a_repeated_join_re_arms_the_intent_and_empties_pending() {
        let mut intent = Intent::new(None);
        let mut pending = PendingAction(Some(join("room-1")));
        arm_with_pending(&mut intent, &mut pending);
        assert!(pending.0.is_none());
        intent.step = super::Step::Done;
        intent.clicks = 3;
        pending.0 = Some(join("room-1"));
        arm_with_pending(&mut intent, &mut pending);
        assert!(pending.0.is_none());
        assert_eq!(intent.step, super::Step::Type);
        assert_eq!(intent.clicks, 0);
    }
}
