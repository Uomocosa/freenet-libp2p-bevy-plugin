use bevy::prelude::Entity;
use bevy::prelude::{Has, Query, With};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use crate::time;

pub fn zone_inputs_from_entities<Op>(
    state: &mut replica_sync::bevy_systems::SyncState<Op>,
    game: &identity::GameId,
    me_zones: &Query<&peers::InZone, With<peers::Me>>,
    zone_details: &Query<replica_sync::ZoneDetails<'_>>,
    member_peers: &Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
) -> (
    Vec<p2p::Libp2pCommand>,
    Vec<replica_sync::state_machine::Input<Op>>,
) {
    let current_zone_entity = me_zones.iter().next().map(|in_zone| **in_zone);
    let current = current_zone_entity.and_then(|entity| {
        let (zone_name, prefix, liveness, members) = zone_details.get(entity).ok()?;
        let zone_name = zone_name.clone();
        let prefix = prefix
            .cloned()
            .unwrap_or_else(|| p2p::HistoryKeyPrefix::new(game, &zone_name, time::UnixTime::EPOCH));
        let liveness = liveness
            .copied()
            .unwrap_or(replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: false,
            });
        let mut member_ids: Vec<identity::PeerId> = members.map_or_else(Vec::new, |members| {
            let members: &[Entity] = members;
            members
                .iter()
                .filter_map(|member| member_peers.get(*member).ok())
                .filter(|(_, is_connected, is_me)| *is_connected && !*is_me)
                .map(|(peer, _, _)| (**peer).clone())
                .collect()
        });
        member_ids.sort();
        Some((zone_name, prefix, member_ids, liveness))
    });
    let mut commands: Vec<p2p::Libp2pCommand> = Vec::new();
    let mut inputs: Vec<replica_sync::state_machine::Input<Op>> = Vec::new();
    match (state.zone.clone(), current) {
        (None, None) => {}
        (Some(previous), None) => {
            commands.push(p2p::Libp2pCommand::Unsubscribe {
                topic: identity::Topic::from_replica_sync(game, &previous),
            });
            state.reset_after_leaving();
        }
        (None, Some((zone_name, prefix, members, liveness))) => {
            commands.push(p2p::Libp2pCommand::Subscribe {
                topic: identity::Topic::from_replica_sync(game, &zone_name),
            });
            inputs.push(replica_sync::state_machine::Input::Joined {
                zone: zone_name,
                prefix,
                members,
                liveness,
            });
        }
        (Some(previous), Some((zone_name, prefix, members, liveness))) if previous != zone_name => {
            commands.push(p2p::Libp2pCommand::Unsubscribe {
                topic: identity::Topic::from_replica_sync(game, &previous),
            });
            commands.push(p2p::Libp2pCommand::Subscribe {
                topic: identity::Topic::from_replica_sync(game, &zone_name),
            });
            inputs.push(replica_sync::state_machine::Input::Joined {
                zone: zone_name,
                prefix,
                members,
                liveness,
            });
        }
        (Some(_), Some((_, prefix, members, liveness))) => {
            let members_changed = state.member_snapshot.members != members;
            let prefix_changed = state.kademlia_history.prefix.as_ref() != Some(&prefix);
            let liveness_changed = state.kademlia_history.liveness != liveness;
            if members_changed || prefix_changed || liveness_changed {
                inputs.push(replica_sync::state_machine::Input::Members {
                    members,
                    prefix,
                    liveness,
                });
            }
        }
    }
    (commands, inputs)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::replica_sync;
    use crate::time;

    #[derive(Resource, Default)]
    struct Captured {
        commands: Vec<p2p::Libp2pCommand>,
        inputs: Vec<replica_sync::state_machine::Input<u8>>,
    }

    fn capture(
        mut state: ResMut<replica_sync::bevy_systems::SyncState<u8>>,
        game: Res<replica_sync::SyncGame>,
        me_zones: Query<&peers::InZone, With<peers::Me>>,
        zone_details: Query<replica_sync::ZoneDetails<'_>>,
        member_peers: Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
        mut captured: ResMut<Captured>,
    ) {
        let game = game.into_inner();
        let (commands, inputs) =
            replica_sync::bevy_systems::SyncState::<u8>::zone_inputs_from_entities(
                &mut state,
                game,
                &me_zones,
                &zone_details,
                &member_peers,
            );
        captured.commands = commands;
        captured.inputs = inputs;
    }

    fn app_with_capture(game: identity::GameId) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(replica_sync::SyncGame(game));
        app.init_resource::<replica_sync::bevy_systems::SyncState<u8>>();
        app.init_resource::<Captured>();
        app.add_systems(Update, capture);
        app
    }

    fn join(
        app: &mut App,
        zone: &identity::ZoneName,
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        liveness: replica_sync::state_machine::ZoneLiveness,
    ) -> Entity {
        let zone_entity = app.world_mut().spawn((zone.clone(), prefix, liveness)).id();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("me")),
            peers::Me,
            peers::InZone(zone_entity),
        ));
        for member in members {
            app.world_mut().spawn((
                peers::Peer::from(member),
                peers::InZone(zone_entity),
                peers::Connected,
            ));
        }
        zone_entity
    }

    fn captured(
        app: &App,
    ) -> (
        Vec<p2p::Libp2pCommand>,
        Vec<replica_sync::state_machine::Input<u8>>,
    ) {
        let captured = app.world().resource::<Captured>();
        (captured.commands.clone(), captured.inputs.clone())
    }

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let zone = identity::ZoneName::from("zone");
        let prefix = p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::from_secs(7));
        let liveness = replica_sync::state_machine::ZoneLiveness {
            is_zone_still_listed: true,
            has_other_history_holders: true,
        };
        let mut app = app_with_capture(game);
        app.update();
        assert_eq!(captured(&app), (Vec::new(), Vec::new()));
        let zone_entity = join(
            &mut app,
            &zone,
            vec![identity::PeerId::from("alice")],
            prefix,
            liveness,
        );
        app.update();
        let (commands, inputs) = captured(&app);
        assert_eq!(commands.len(), 1);
        assert!(matches!(
            inputs.as_slice(),
            [replica_sync::state_machine::Input::Joined { .. }]
        ));
        let joined = inputs.into_iter().next().expect("joined");
        app.world_mut()
            .resource_mut::<replica_sync::bevy_systems::SyncState<u8>>()
            .update(joined, replica_sync::ElapsedSinceAppStart::default());
        app.update();
        assert_eq!(captured(&app), (Vec::new(), Vec::new()));
        app.world_mut()
            .entity_mut(zone_entity)
            .insert(replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: false,
                has_other_history_holders: false,
            });
        app.update();
        let (_, inputs) = captured(&app);
        assert!(matches!(
            inputs.as_slice(),
            [replica_sync::state_machine::Input::Members { .. }]
        ));
        let me_entity = app
            .world_mut()
            .query_filtered::<Entity, With<peers::Me>>()
            .iter(app.world())
            .next()
            .expect("me");
        app.world_mut()
            .entity_mut(me_entity)
            .remove::<peers::InZone>();
        app.update();
        let (commands, inputs) = captured(&app);
        assert_eq!(commands.len(), 1);
        assert_eq!(inputs, Vec::new());
        assert_eq!(
            app.world()
                .resource::<replica_sync::bevy_systems::SyncState<u8>>()
                .zone,
            None
        );
    }
}
