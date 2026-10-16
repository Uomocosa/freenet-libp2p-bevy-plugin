use std::marker::PhantomData;

use crate::p2p;

pub struct Config<T: p2p::Message> {
    pub mode: p2p::TransportMode,
    pub mdns: p2p::MdnsMode,
    pub marker: PhantomData<T>,
}

impl<T: p2p::Message> Config<T> {
    #[must_use]
    pub const fn new(mode: p2p::TransportMode, mdns: p2p::MdnsMode) -> Self {
        Self {
            mode,
            mdns,
            marker: PhantomData,
        }
    }
}
// no test_usage necessary
