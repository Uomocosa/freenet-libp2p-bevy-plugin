use crate::starboard;

use bevy::prelude::Component;
use derive_more::Deref;

#[derive(Component)]
pub struct BoardSprite;

#[derive(Component)]
pub struct BoardBorderSprite;

#[derive(Component)]
pub struct Cursor;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorPlacement {
    InRoom,
    Preview,
    Stream,
}

#[derive(Component)]
pub struct CursorOutline;

#[derive(Component, Deref)]
pub struct CursorBrush(pub starboard::identity::DailyBrush);

#[derive(Component)]
pub struct CursorLabel;
