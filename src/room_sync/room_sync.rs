use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::room_sync;

#[derive(Resource, Deref, DerefMut)]
pub struct RoomSync<S: room_sync::RoomState> {
    pub pending: Vec<S::Op>,
}

impl<S: room_sync::RoomState> Default for RoomSync<S> {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
        }
    }
}

#[rustfmt::skip]
impl<S: room_sync::RoomState> RoomSync<S> {
    pub fn send(&mut self, op: S::Op) { self.push(op); }
    pub fn take_all(&mut self) -> Vec<S::Op> { self.split_off(0) }
    pub fn restore(&mut self, ops: Vec<S::Op>) { self.extend(ops); }
}

// no test_usage necessary
