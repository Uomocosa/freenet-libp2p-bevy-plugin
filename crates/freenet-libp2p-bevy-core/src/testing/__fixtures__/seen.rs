use bevy::prelude::Resource;

use crate::testing;

#[derive(Resource, Default)]
pub struct Seen {
    pub pointer: Vec<testing::PointerStep>,
    pub keys: Vec<testing::RecordedKeyStep>,
    pub window_keys: Vec<testing::RecordedKeyStep>,
}
