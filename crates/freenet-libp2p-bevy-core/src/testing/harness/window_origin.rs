use std::str::FromStr;

use atomic_delegate_macros::atomic_delegates;
use bevy::math::IVec2;
use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct WindowOrigin(pub IVec2);

#[atomic_delegates]
impl FromStr for WindowOrigin {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {}
}

#[cfg(test)]
mod tests {
    use bevy::math::IVec2;

    use super::WindowOrigin;

    #[test]
    fn test_usage() {
        assert_eq!(
            "481, 0".parse::<WindowOrigin>(),
            Ok(WindowOrigin(IVec2::new(481, 0)))
        );
        assert!("481".parse::<WindowOrigin>().is_err());
    }
}
