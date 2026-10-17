#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::structs::NetLink;

mod dialable;
pub use dialable::dialable;

mod wait_ready;
pub use wait_ready::wait_ready;
