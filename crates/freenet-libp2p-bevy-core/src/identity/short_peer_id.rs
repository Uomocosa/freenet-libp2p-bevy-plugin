use derive_more::Deref;

use atomic_delegate_macros::atomic_delegates;

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct ShortPeerId(pub String);

#[atomic_delegates]
impl From<&identity::PeerId> for ShortPeerId {
    fn from(id: &identity::PeerId) -> Self {}
}
