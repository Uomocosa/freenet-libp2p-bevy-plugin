use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{DOUBLE_CLICK_SECS, RoomButton};

pub fn enter_on_double_click(
    buttons: Query<&RoomButton, Added<Pressed>>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    time: Res<Time>,
    mut last_press: Local<Option<(core_discovery::RoomName, f32)>>,
    mut commands: MessageWriter<core_discovery::Command>,
) {
    let rows = rows.map(Res::into_inner);
    let now = time.into_inner().elapsed_secs();
    for button in &buttons {
        let room: core_discovery::RoomName = (**button).clone();
        let is_joinable = rows.is_some_and(|rows| {
            rows.iter()
                .find(|row| row.name == room)
                .is_some_and(|row| row.occupancy != core_discovery::RoomOccupancy::Full)
        });
        if !is_joinable {
            *last_press = None;
            continue;
        }
        let is_double = last_press
            .as_ref()
            .is_some_and(|(previous, at)| *previous == room && now - *at <= DOUBLE_CLICK_SECS);
        if is_double {
            commands.write(core_discovery::Command::Join(room));
            *last_press = None;
        } else {
            *last_press = Some((room, now));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::Pressed;
    use std::time::Duration;

    use super::enter_on_double_click;

    fn row(name: &str, occupancy: core_discovery::RoomOccupancy) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 0,
            room_size: core_discovery::RoomSize::DEFAULT,
            occupancy,
            friends: 0,
        }
    }

    fn rows_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha", core_discovery::RoomOccupancy::HasSpace),
            row("beta", core_discovery::RoomOccupancy::HasSpace),
            row("gamma", core_discovery::RoomOccupancy::Full),
        ]));
        app.add_systems(Update, enter_on_double_click);
        app
    }

    fn press(app: &mut App, room: &str) {
        app.world_mut().spawn((
            ui::discovery::RoomButton(core_discovery::RoomName::from(room)),
            Pressed,
        ));
    }

    fn sent(app: &App) -> Vec<core_discovery::Command> {
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    #[test]
    fn test_usage() {
        let mut app = rows_app();
        press(&mut app, "alpha");
        app.update();
        assert_eq!(sent(&app), Vec::new());
    }

    #[test]
    fn a_double_press_within_the_window_joins_once() {
        let mut app = rows_app();
        press(&mut app, "alpha");
        app.update();
        press(&mut app, "alpha");
        app.update();
        assert_eq!(
            sent(&app),
            vec![core_discovery::Command::Join(
                core_discovery::RoomName::from("alpha")
            )]
        );
    }

    #[test]
    fn two_presses_half_a_second_apart_do_not_join() {
        let mut app = rows_app();
        press(&mut app, "alpha");
        app.update();
        for _ in 0..10 {
            app.update();
        }
        press(&mut app, "alpha");
        app.update();
        assert_eq!(sent(&app), Vec::new());
    }

    #[test]
    fn presses_on_two_different_rows_do_not_join() {
        let mut app = rows_app();
        press(&mut app, "alpha");
        app.update();
        press(&mut app, "beta");
        app.update();
        assert_eq!(sent(&app), Vec::new());
    }

    #[test]
    fn a_double_press_on_a_full_room_does_not_join() {
        let mut app = rows_app();
        press(&mut app, "gamma");
        app.update();
        press(&mut app, "gamma");
        app.update();
        assert_eq!(sent(&app), Vec::new());
    }
}
