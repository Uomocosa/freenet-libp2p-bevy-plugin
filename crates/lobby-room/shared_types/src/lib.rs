#[path = "__basic__/mod.rs"]
mod basic;
pub use basic::enums::RoomMembership;
pub use basic::newtypes::RoomName;
pub use basic::structs::RoomMember;
mod constants;
pub use constants::MAX_LOBBY_ROOMS;
mod lobby;
pub use lobby::Lobby;
mod room;
pub use room::Room;
mod room_size;
pub use room_size::RoomSize;
mod room_size_error;
pub use room_size_error::RoomSizeError;

#[path = "../__methods__/mod.rs"]
mod methods;
