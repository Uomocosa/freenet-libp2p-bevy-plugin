use bevy::input::ButtonState;
use bevy::math::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub enum PointerStep {
    Moved(Vec2),
    Button(ButtonState),
}
