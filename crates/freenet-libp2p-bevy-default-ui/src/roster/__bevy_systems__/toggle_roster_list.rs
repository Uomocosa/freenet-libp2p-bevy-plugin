use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::roster;
use roster::{ListState, RosterPill};

pub fn toggle_roster_list(
    presses: Query<(), (Added<Pressed>, With<RosterPill>)>,
    mut list: ResMut<ListState>,
) {
    if presses.is_empty() {
        return;
    }
    list.toggle();
}

#[cfg(test)]
mod tests {
    use super::toggle_roster_list;
    use crate::roster;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<roster::ListState>();
        let pill = app.world_mut().spawn(roster::RosterPill).id();
        app.add_systems(Update, toggle_roster_list);
        app.world_mut().entity_mut(pill).insert(Pressed);
        app.update();
        assert!(**app.world().resource::<roster::ListState>());
    }

    #[test]
    fn a_pill_press_toggles_the_list() {
        let mut app = App::new();
        app.init_resource::<roster::ListState>();
        let pill = app.world_mut().spawn(roster::RosterPill).id();
        app.add_systems(Update, toggle_roster_list);
        app.world_mut().entity_mut(pill).insert(Pressed);
        app.update();
        assert!(**app.world().resource::<roster::ListState>());
        app.world_mut().entity_mut(pill).remove::<Pressed>();
        app.world_mut().entity_mut(pill).insert(Pressed);
        app.update();
        assert!(!**app.world().resource::<roster::ListState>());
    }

    #[test]
    fn without_a_press_the_list_state_stays() {
        let mut app = App::new();
        app.init_resource::<roster::ListState>();
        app.world_mut().spawn(roster::RosterPill);
        app.add_systems(Update, toggle_roster_list);
        app.update();
        app.update();
        assert!(!**app.world().resource::<roster::ListState>());
    }
}
