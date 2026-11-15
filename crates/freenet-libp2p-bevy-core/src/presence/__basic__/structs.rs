use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Whereabouts {
    pub zone: identity::Zone,
    pub display_name: Option<identity::Username>,
}
