use bevy::prelude::{Component, Entity};
use derive_more::{Deref, DerefMut};

use crate::ui;

#[derive(Component, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct RollMachine(pub ui::roll::State);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct RollNumber {
    pub pill: Entity,
    pub slot: usize,
    pub value: u32,
    pub row_height: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollNumberColumn;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollNumberRow {
    Outgoing,
    Incoming,
}
