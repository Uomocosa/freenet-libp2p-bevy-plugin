#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;

#[path = "__basic__/mod.rs"]
mod basic;

#[path = "__traits__/mod.rs"]
pub mod traits;

pub use basic::constants::FREENET_NODE_STARTED;
pub use basic::enums::{FakeWireClientFrame, FakeWireServerFrame, FanOutTarget, SyntheticInput};
pub use basic::messages::E2EExtensionEffect;
pub use basic::newtypes::{PointerSpeed, TypedText};
pub use basic::structs::{
    CursorTarget, DirectDialBookEntry, DirectDialStatus, FakeWireHello, FanOut, PointerPath,
    RoutedEvent,
};

mod fake_network_plugin;
pub use fake_network_plugin::FakeNetworkPlugin;

mod fake_network_server;
pub use fake_network_server::FakeNetworkServer;

mod no_fake_network_kit;
pub use no_fake_network_kit::NoFakeNetworkKit;

mod fake_task_handles;
pub use fake_task_handles::FakeTaskHandles;

mod fake_wire_framing;
pub use fake_wire_framing::FakeWireFraming;

pub(crate) mod orchestrator_shared;

mod network;
pub use network::Network;

mod network_args;
pub use network_args::NetworkArgs;

mod peer_message_router;
pub use peer_message_router::PeerMessageRouter;

mod script_checkpoint;
pub use script_checkpoint::ScriptCheckpoint;

mod e2e_driver_plugin;
pub use e2e_driver_plugin::E2EDriverPlugin;

mod direct_dial_e2e_methods;
pub use direct_dial_e2e_methods::DirectDialE2EMethods;

mod named_key;
pub use named_key::NamedKey;

mod synthetic_input_queue;
pub use synthetic_input_queue::SyntheticInputQueue;

mod typed_key;
pub use typed_key::TypedKey;

mod node_centre;
pub use node_centre::NodeCentre;

#[cfg(feature = "e2e-harness")]
pub mod harness;
