use bevy::prelude::*;

use crate::starboard;

pub type CursorSprites<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static starboard::render::CursorPlacement,
        &'static starboard::render::Owner,
        &'static mut starboard::render::CursorBrush,
        &'static mut Sprite,
        &'static mut Transform,
    ),
    With<starboard::render::Cursor>,
>;

// no test_usage necessary
