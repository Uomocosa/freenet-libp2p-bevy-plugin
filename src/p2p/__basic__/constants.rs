use std::num::NonZeroU8;

pub const KADEMLIA_MAX_PACKET_BYTES: usize = 128 * 1024;
pub const KADEMLIA_MAX_RECORDS: usize = 8192;
pub const DIAL_CONCURRENCY_FACTOR: NonZeroU8 = NonZeroU8::MIN;

// no test_usage necessary
