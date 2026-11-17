use crate::discovery as core_discovery;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::discovery::{PREVIEW_HOVER_DELAY_SECS, RoomButton, SelectedRoom};

pub fn track_room_preview(
    time: Res<Time>,
    buttons: Query<(&RoomButton, &Hovered)>,
    mut selection: ResMut<SelectedRoom>,
    mut hovered_room: Local<Option<core_discovery::RoomName>>,
    mut hovered_secs: Local<f32>,
) {
    let delta_secs = time.into_inner().delta_secs();
    let hovered_now = buttons
        .iter()
        .find(|(_, hovered)| hovered.get())
        .map(|(button, _)| (**button).clone());
    if *hovered_room != hovered_now {
        *hovered_room = hovered_now;
        *hovered_secs = 0.0;
    } else if hovered_room.is_some() {
        *hovered_secs += delta_secs;
    }
    let Some(room) = hovered_room.as_ref() else {
        **selection = None;
        return;
    };
    if *hovered_secs >= PREVIEW_HOVER_DELAY_SECS {
        **selection = Some(room.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    use super::track_room_preview;

    fn app_with(hovered: Option<&str>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.init_resource::<ui::discovery::SelectedRoom>();
        if let Some(room) = hovered {
            app.world_mut().spawn((
                ui::discovery::RoomButton(core_discovery::RoomName::from(room)),
                Hovered(true),
            ));
        }
        app.add_systems(Update, track_room_preview);
        app
    }

    fn selected(app: &App) -> Option<core_discovery::RoomName> {
        (**app.world().resource::<ui::discovery::SelectedRoom>()).clone()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(Some("alpha"));
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(selected(&app), None);
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            selected(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }

    #[test]
    fn hovering_for_three_hundred_milliseconds_selects_the_room() {
        let mut app = app_with(Some("alpha"));
        for _ in 0..7 {
            app.update();
        }
        assert_eq!(
            selected(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }

    #[test]
    fn hovering_for_less_than_three_hundred_milliseconds_does_not_select() {
        let mut app = app_with(Some("alpha"));
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(selected(&app), None);
    }

    #[test]
    fn hovering_out_clears_the_preview() {
        let mut app = app_with(None);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        app.update();
        assert_eq!(selected(&app), None);
    }

    #[test]
    fn hovering_a_room_replaces_the_previous_hover() {
        let mut app = app_with(Some("alpha"));
        {
            **app
                .world_mut()
                .resource_mut::<ui::discovery::SelectedRoom>() =
                Some(core_discovery::RoomName::from("beta"));
        }
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(
            selected(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }
}
