#[path = "__app__/mod.rs"]
pub mod app;

#[path = "__basic__/mod.rs"]
mod basic;

#[path = "__fakes__/mod.rs"]
pub mod fakes;

#[path = "__fixtures__/mod.rs"]
pub mod fixtures;

#[path = "__queries__/mod.rs"]
pub mod queries;

#[path = "__app__/__bevy_systems__/mod.rs"]
pub mod bevy_systems;

#[path = "__fakes__/test_doubles/__traits__/mod.rs"]
pub mod traits;

#[cfg(feature = "e2e-harness")]
pub use app::harness;

pub use app::connect_and_seed_kad::connect_and_seed_kad;
pub use app::e2e_driver_app::e2e_driver_app;
pub use app::e2e_driver_plugin::E2EDriverPlugin;
pub use app::fake_network_plugin::FakeNetworkPlugin;
pub use app::friends_app_with_book::friends_app_with_book;
pub use app::join_player::join_player;
pub use app::network_node::network_node;
pub use app::prepare_second_zone::prepare_second_zone;
pub use app::publish_zone_message::publish_zone_message;
pub use app::spawn_joined_zone::spawn_joined_zone;
pub use app::spectate::spectate;
pub use app::spectator_node::spectator_node;
pub use app::spectator_pair::spectator_pair;
pub use app::stop_spectating::stop_spectating;
pub use app::sync_app_with_capture::sync_app_with_capture;
pub use app::test_app::TestApp;
pub use app::wait_connected::wait_connected;
pub use app::wait_connected_to_all::wait_connected_to_all;
pub use app::wait_for_history::wait_for_history;
pub use app::wait_ready::wait_ready;

pub use basic::enums::{
    FakeWireClientFrame, FakeWireServerFrame, KeyStep, KeyStepError, PointerStep, SyntheticInput,
};
pub use basic::messages::E2EExtensionEffect;
pub use basic::newtypes::{EscapePresses, Millis, PointerSpeed, RepeatCount, TypedText};
pub use basic::structs::{
    CursorTarget, DirectDialBookEntry, DirectDialStatus, FanOut, KeyEvent, Node, PointerPath,
    RoutedEvent, Seen,
};
pub use fakes::captured_outputs::CapturedOutputs;
pub use fakes::direct_dial_e2e_methods::DirectDialE2EMethods;
pub use fakes::fake_machine_state::FakeMachineState;
pub use fakes::fake_network_server::FakeNetworkServer;
pub use fakes::fake_replica::FakeReplica;
pub use fakes::fake_task_handles::FakeTaskHandles;
pub use fakes::fake_wire_framing::FakeWireFraming;
pub use fakes::no_fake_network_kit::NoFakeNetworkKit;
pub(crate) use fakes::orchestrator_shared;
pub use fakes::peer_message_router::PeerMessageRouter;
pub use fakes::probe::Probe;
pub use fakes::stub_board::StubBoard;
pub use fakes::synthetic_input_queue::SyntheticInputQueue;

pub use fixtures::DummyMessage;
pub use fixtures::FREENET_NODE_STARTED;
pub use fixtures::FakeWireHello;
pub use fixtures::FanOutTarget;
pub use fixtures::KeyValueOp;
pub use fixtures::KeyValueReplica;

pub use fixtures::alive_zone_liveness::alive_zone_liveness;
pub use fixtures::anchor::anchor;
pub use fixtures::area::area;
pub use fixtures::at_secs::at_secs;
#[cfg(feature = "e2e-harness")]
pub use fixtures::board_status::board_status;
pub use fixtures::dial::dial;
pub use fixtures::dial_state_with_friend::dial_state_with_friend;
pub use fixtures::dialable::dialable;
pub use fixtures::dialed::dialed;
pub use fixtures::gossipsub_message_event::gossipsub_message_event;
pub use fixtures::hashes::hashes;
pub use fixtures::history_chunk::history_chunk;
pub use fixtures::history_config::history_config;
pub use fixtures::history_record_not_found_input::history_record_not_found_input;
pub use fixtures::is_history_event::is_history_event;
pub use fixtures::is_ready_event::is_ready_event;
pub use fixtures::joined_input::joined_input;
pub use fixtures::kademlia_record_found::kademlia_record_found;
pub use fixtures::key::Key;
pub use fixtures::key_value_map::KeyValueMap;
pub use fixtures::libp2p_event::libp2p_event;
pub use fixtures::libp2p_events::libp2p_events;
#[cfg(feature = "e2e-harness")]
pub use fixtures::local_board::local_board;
pub use fixtures::long_peer_id::long_peer_id;
pub use fixtures::long_two_addr_peer::long_two_addr_peer;
pub use fixtures::machine::machine;
pub use fixtures::manifest::manifest;
pub use fixtures::meta::meta;
pub use fixtures::network::Network;
pub use fixtures::network_args::NetworkArgs;
pub use fixtures::network_args_probe::NetworkArgsProbe;
pub use fixtures::node::node;
pub use fixtures::other_zone_remote_input::other_zone_remote_input;
pub use fixtures::peer::peer;
pub use fixtures::peer_presence::peer_presence;
pub use fixtures::placement::placement;
pub use fixtures::ready_replica_state::ready_replica_state;
pub use fixtures::recorded_key_step::RecordedKeyStep;
pub use fixtures::registered_peer::registered_peer;
pub use fixtures::saved_peer_books::saved_peer_books;
pub use fixtures::script_checkpoint::ScriptCheckpoint;
pub use fixtures::snapshot_part_input::snapshot_part_input;
pub use fixtures::spectated_state::spectated_state;
pub use fixtures::spectating_machine::spectating_machine;
pub use fixtures::spectating_state::spectating_state;
pub use fixtures::spectator_state::spectator_state;
pub use fixtures::subscribe::subscribe;
pub use fixtures::swarm_loop_state::swarm_loop_state;
pub use fixtures::swarm_task_override::swarm_task_override;
pub use fixtures::typed_key::TypedKey;
pub use fixtures::wait_for_message::wait_for_message;
pub use fixtures::wait_until::wait_until;
pub use fixtures::zone_subscriptions::zone_subscriptions;

pub use queries::captured_outputs::captured_outputs;
pub use queries::component::component;
pub use queries::count_matching::count_matching;
pub use queries::count_session_outputs::count_session_outputs;
pub use queries::fetched_replica_slice_indices::fetched_replica_slice_indices;
pub use queries::find_entity::find_entity;
pub use queries::find_matching::find_matching;
pub use queries::has_spectated::has_spectated;
pub use queries::is_live_with::is_live_with;
pub use queries::is_subscribed::is_subscribed;
pub use queries::lease_outputs::lease_outputs;
pub use queries::libp2p_event_from_kademlia_result::libp2p_event_from_kademlia_result;
pub use queries::node_centre::NodeCentre;
pub use queries::ready_peer::ready_peer;
pub use queries::requested_replica_slice_indices::requested_replica_slice_indices;
pub use queries::session_outputs::session_outputs;
pub use queries::spectator_count::spectator_count;
pub use queries::written_messages::written_messages;
pub use queries::zone_message_count::zone_message_count;
