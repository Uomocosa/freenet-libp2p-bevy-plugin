use crate::testing;

pub type KeyValueReplica = testing::FakeReplica<testing::KeyValueOp, Vec<(u8, u32)>>;
