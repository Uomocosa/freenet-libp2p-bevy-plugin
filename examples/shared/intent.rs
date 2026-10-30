#![allow(clippy::needless_pass_by_value)]
use std::ops::Deref;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKeyCode};
use bevy::input::mouse::MouseButtonInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;
use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::identity;

use crate::shared::action::Action;

/// One scripted interaction, driven through **real** Bevy input.
///
/// `create:<room>` focuses the room-name field, types the name with
/// [`KeyboardInput`] events, and waits for the Create button to become
/// clickable; `join:<room>` waits for that room's button to appear. Either way
/// the button is then pressed with a real pointer click (window cursor +
/// [`MouseButtonInput`]) and the click is retried, after a cooldown, until the
/// room is actually joined — the cooldown avoids toggling back out, since a
/// second press on the joined room is a `Leave`.
/// The action the intent should drive next, one scripted step at a time.
/// A scripted session writes it when it wants to create a room, join one, or
/// join one again after a leave. `script_input` takes it (`take()`) and always
/// re-arms the intent, even for a repeated `Join`, so a rejoin after a leave
/// clicks again.
#[derive(Resource, Default, Debug, Clone)]
pub struct PendingAction(pub Option<Action>);

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
    status: Res<discovery::Status>,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    fields: Query<Entity, With<F>>,
    create: Query<&UiGlobalTransform, (With<C>, With<Interaction>)>,
    join: Query<(&UiGlobalTransform, &J), With<Interaction>>,
    mut keyboard: MessageWriter<KeyboardInput>,
    mut mouse: MessageWriter<MouseButtonInput>,
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
            let transform = match &action {
                Action::Create(_) => create.iter().next(),
                Action::Join(room) => join
                    .iter()
                    .find(|(_, button)| ***button == *room)
                    .map(|(transform, _)| transform),
            };
            let Some(transform) = transform else {
                return;
            };
            if intent.clicks == 0 {
                tracing::info!("lobby intent pressed {action:?}");
            }
            intent.cooldown = 0;
            window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
            mouse.write(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: window_entity,
            });
            intent.step = Step::Release;
        }
        Step::Release => {
            mouse.write(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
                window: window_entity,
            });
            intent.clicks = intent.clicks.saturating_add(1);
            intent.step = Step::Verify;
        }
        Step::Verify => {
            let target = match &action {
                Action::Create(room) | Action::Join(room) => room,
            };
            let joined = status
                .room
                .as_ref()
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
    use freenet_libp2p_bevy_plugin::identity;

    use super::{Intent, PendingAction, arm_with_pending};
    use crate::shared::action::Action;

    fn join(room: &str) -> Action {
        Action::Join(identity::RoomName::from(room))
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
