use bevy::prelude::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAlign {
    Start,
    Centre,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAnchor {
    Parent,
    Entity(Entity),
}
