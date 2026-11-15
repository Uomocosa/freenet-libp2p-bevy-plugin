use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Zone {
    Global,
    Restricted(identity::ZoneName),
}

impl Zone {
    #[must_use]
    pub const fn restricted(&self) -> Option<&identity::ZoneName> {
        match self {
            Self::Global => None,
            Self::Restricted(name) => Some(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Zone;
    use crate::identity;

    #[test]
    fn test_usage() {
        let name = identity::ZoneName::from("zone-a");
        assert_eq!(Zone::Global.restricted(), None);
        assert_eq!(Zone::Restricted(name.clone()).restricted(), Some(&name));
    }
}
