use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Whereabouts {
    pub place: Option<identity::RoomName>,
    pub display_name: Option<identity::Username>,
}
