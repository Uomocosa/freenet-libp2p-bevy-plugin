use bevy::prelude::{ButtonInput, MouseButton};
use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct HeldButton(pub MouseButton);

impl HeldButton {
    pub(crate) fn is_held(self, buttons: &ButtonInput<MouseButton>) -> bool {
        buttons.pressed(*self) || buttons.just_pressed(*self)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{ButtonInput, MouseButton};

    use super::HeldButton;

    #[test]
    fn test_usage() {
        let mut buttons = ButtonInput::<MouseButton>::default();
        assert!(!HeldButton(MouseButton::Left).is_held(&buttons));
        buttons.press(MouseButton::Left);
        assert!(HeldButton(MouseButton::Left).is_held(&buttons));
    }
}
