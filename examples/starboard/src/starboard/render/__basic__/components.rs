use crate::starboard;

use bevy::prelude::Component;

#[derive(Component)]
pub struct BoardSprite;

#[derive(Component)]
pub struct BoardBorderSprite;

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct CursorOutline;

#[derive(Component)]
pub struct CursorBrush(pub starboard::identity::DailyBrush);

#[derive(Component)]
pub struct CursorLabel;
