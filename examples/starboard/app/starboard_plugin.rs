use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, room_sync};

use crate::starboard;

pub struct StarboardPlugin;

impl Plugin for StarboardPlugin {
    fn build(&self, app: &mut App) {
        use starboard::{input, render, room, ui};
        let in_room = |status: Res<discovery::Status>| room::is_in_room(&status);
        app.init_resource::<starboard::canvas::Canvas>()
            .init_resource::<render::CanvasView>()
            .init_resource::<input::OwnBrush>()
            .init_resource::<input::LatestPaintedCell>()
            .init_resource::<input::CursorCell>()
            .init_resource::<room_sync::LamportClock>()
            .init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>()
            .init_resource::<room::RoomBrushes>()
            .init_resource::<room::RemoteCursors>()
            .insert_resource(room::BrushRefreshTimer(Timer::from_seconds(
                room::BRUSH_REFRESH_SECONDS,
                TimerMode::Repeating,
            )))
            .add_message::<input::PendingLocalDraw>()
            .add_message::<discovery::Event>()
            .add_systems(
                Startup,
                (
                    render::bevy_systems::spawn_board,
                    ui::bevy_systems::spawn_member_panel,
                ),
            )
            .add_systems(
                Update,
                (
                    room::bevy_systems::reset_canvas,
                    room::bevy_systems::refresh_room_brushes,
                    ui::bevy_systems::rebuild_member_list,
                    input::bevy_systems::track_cursor,
                    render::bevy_systems::sync_board_visibility,
                    render::bevy_systems::fit_board,
                    input::bevy_systems::scroll_brush_size.run_if(in_room),
                    input::bevy_systems::track_pointer.run_if(in_room),
                    input::bevy_systems::send_local_draws.run_if(in_room),
                    room::bevy_systems::publish_cursor.run_if(in_room),
                    room::bevy_systems::collect_remote_cursors,
                    render::bevy_systems::upload_dirty_tiles,
                    render::bevy_systems::follow_brush,
                    render::bevy_systems::draw_remote_cursors,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::AccumulatedMouseScroll;
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::room_sync::{OpMeta, RoomState};
    use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, plugin, room_sync};

    use super::StarboardPlugin;
    use crate::starboard;

    fn ready_status() -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ready_status());
        app.add_plugins(StarboardPlugin);
    }

    const fn preview_noop(_world: &mut World) {}

    fn paint_strokes(world: &mut World) {
        let brush = *world.resource::<starboard::input::OwnBrush>();
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        for index in 0_u16..28 {
            let x = 96_u16.saturating_add(index.saturating_mul(24));
            let y = 300_u16.saturating_add(index.saturating_mul(7));
            let op = starboard::input::draw_op(
                brush,
                starboard::canvas::DrawAction::Paint(brush.color),
                starboard::canvas::CellPos { x, y },
                starboard::canvas::CellPos {
                    x: x.saturating_add(40),
                    y: y.saturating_add(60),
                },
            );
            canvas.apply(
                OpMeta {
                    lamport: u32::from(index).saturating_add(1),
                    author: 1,
                },
                &op,
            );
        }
    }

    fn show_cursor(world: &mut World) {
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let Ok(mut window) = windows.single_mut(world) else {
            return;
        };
        window.set_cursor_position(Some(Vec2::new(560.0, 300.0)));
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_board"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("empty board"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("painted board"),
                    apply: paint_strokes,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("brush over the board"),
                    apply: show_cursor,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn board_ui_png_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn board_ui_scene_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    const SETTLE: std::time::Duration = std::time::Duration::from_millis(20);
    const MAX_ROUNDS: usize = 400;
    const CONVERGE_BUDGET: std::time::Duration = std::time::Duration::from_secs(10);

    fn node_with(game: &identity::GameId) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<starboard::room::CursorUpdate>::new(
                p2p::TransportMode::Both,
                p2p::MdnsMode::Disabled,
            )
            .with_game(game.clone()),
        ));
        app.add_plugins(room_sync::SyncPlugin::<starboard::canvas::Canvas>::new(
            game.clone(),
        ));
        app.insert_resource(ready_status());
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<AccumulatedMouseScroll>();
        app.add_message::<discovery::Event>();
        app.add_plugins(StarboardPlugin);
        app.update();
        app
    }

    fn dialable(peer: &identity::Peer) -> identity::Peer {
        identity::Peer {
            id: peer.id.clone(),
            addrs: peer
                .addrs
                .iter()
                .map(|addr| identity::PeerAddr(addr.replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/")))
                .collect(),
        }
    }

    fn ready(app: &App) -> Option<identity::Peer> {
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_rx
            .borrow()
            .clone()
    }

    fn wait_ready(app: &mut App) -> identity::Peer {
        for _ in 0..MAX_ROUNDS {
            app.update();
            if let Some(peer) = ready(app) {
                return peer;
            }
            std::thread::sleep(SETTLE);
        }
        panic!("node never became ready");
    }

    fn wait_connected(
        app: &mut App,
        events: &mut tokio::sync::broadcast::Receiver<p2p::NetEvent>,
        peer: &identity::PeerId,
    ) {
        for _ in 0..MAX_ROUNDS {
            app.update();
            while let Ok(event) = events.try_recv() {
                if let p2p::NetEvent::PeerConnected(id) = event
                    && &id == peer
                {
                    return;
                }
            }
            std::thread::sleep(SETTLE);
        }
        panic!("peer never connected");
    }

    fn join(app: &mut App, room: &identity::RoomName, members: Vec<identity::PeerId>) {
        let mut room_members = discovery::Members::new();
        for peer in members {
            room_members.insert(
                peer,
                discovery::Member {
                    heartbeat: discovery::Heartbeat {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: None,
                    },
                    status: discovery::LinkStatus::Connected,
                },
            );
        }
        app.insert_resource(discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at: discovery::UnixTime::from_secs(1),
                phase: discovery::RoomPhase::Ready,
                members: room_members,
            }),
            ..discovery::Status::default()
        });
        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));
    }

    fn scripted_ops() -> Vec<starboard::canvas::DrawOp> {
        let brush = starboard::input::OwnBrush::default();
        (0_u16..6)
            .map(|index| {
                let x = 200_u16.saturating_add(index.saturating_mul(60));
                let y = 300_u16.saturating_add(index.saturating_mul(30));
                starboard::input::draw_op(
                    brush,
                    starboard::canvas::DrawAction::Paint(brush.color),
                    starboard::canvas::CellPos { x, y },
                    starboard::canvas::CellPos {
                        x: x.saturating_add(80),
                        y: y.saturating_add(40),
                    },
                )
            })
            .collect()
    }

    fn canvas_hash(app: &App) -> u64 {
        starboard::canvas::hash_canvas(app.world().resource::<starboard::canvas::Canvas>())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn two_apps_converge_on_the_painted_canvas() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-ae");
        let mut apps = vec![node_with(&game), node_with(&game)];
        let peers: Vec<identity::Peer> = apps.iter_mut().map(wait_ready).collect();

        let mut events = apps[1]
            .world()
            .resource::<p2p::NetEventChannel>()
            .subscribe();
        apps[1]
            .world()
            .resource::<p2p::NetCommandSender>()
            .send(p2p::NetCommand::Dial(dialable(&peers[0])))
            .expect("dial");
        wait_connected(&mut apps[1], &mut events, &peers[0].id);

        for (index, app) in apps.iter_mut().enumerate() {
            let others = peers
                .iter()
                .enumerate()
                .filter(|(other, _)| *other != index)
                .map(|(_, peer)| peer.id.clone())
                .collect();
            join(app, &room, others);
        }
        for _ in 0..50 {
            for app in &mut apps {
                app.update();
            }
            tokio::time::sleep(SETTLE).await;
        }

        for op in scripted_ops() {
            apps[0]
                .world_mut()
                .write_message(starboard::input::PendingLocalDraw(op));
            apps[0].update();
        }

        let started = std::time::Instant::now();
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<u64> = apps.iter().map(canvas_hash).collect();
            if hashes.first() == hashes.last() && hashes.first().is_some_and(|hash| *hash != 0) {
                return;
            }
            assert!(
                started.elapsed() <= CONVERGE_BUDGET,
                "canvases did not converge: {hashes:?}"
            );
            tokio::time::sleep(SETTLE).await;
        }
    }
}
