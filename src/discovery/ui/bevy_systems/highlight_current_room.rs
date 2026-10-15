use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{CURRENT_ROOM_COLOR, ROOM_COLOR};
use discovery::ui::{IsCurrentRoom, RoomButton};

pub fn highlight_current_room(
    mut commands: Commands,
    snapshot: Res<discovery::Snapshot>,
    mut rows: Query<(
        Entity,
        &RoomButton,
        &mut BackgroundColor,
        Option<&IsCurrentRoom>,
    )>,
) {
    let snapshot = snapshot.into_inner();
    let current = snapshot.room.as_ref().map(|room| &room.name);
    for (entity, button, mut background, marker) in &mut rows {
        let is_current = current.is_some_and(|room| *room == **button);
        let color = if is_current {
            CURRENT_ROOM_COLOR
        } else {
            ROOM_COLOR
        };
        let BackgroundColor(existing) = &mut *background;
        if *existing != color {
            *existing = color;
        }
        match (is_current, marker.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(IsCurrentRoom);
            }
            (false, true) => {
                commands.entity(entity).remove::<IsCurrentRoom>();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::discovery;
    use crate::net_id;

    use super::highlight_current_room;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let room = net_id::RoomName::from("alpha");
        let snapshot = discovery::Snapshot {
            room: Some(discovery::Room {
                name: room.clone(),
                members: discovery::Members::new(),
            }),
            ..Default::default()
        };
        app.insert_resource(snapshot);
        let entity = app
            .world_mut()
            .spawn((
                discovery::ui::RoomButton(room),
                BackgroundColor(Color::BLACK),
            ))
            .id();
        app.add_systems(Update, highlight_current_room);
        app.update();
        assert!(
            app.world()
                .entity(entity)
                .contains::<discovery::ui::IsCurrentRoom>()
        );
    }
}
