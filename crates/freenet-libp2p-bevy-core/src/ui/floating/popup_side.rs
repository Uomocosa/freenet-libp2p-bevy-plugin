#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupSide {
    Below,
    Above,
    Right,
    Left,
}

impl PopupSide {
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Below => Self::Above,
            Self::Above => Self::Below,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PopupSide;

    #[test]
    fn test_usage() {
        assert_eq!(PopupSide::Below.opposite(), PopupSide::Above);
        assert_eq!(PopupSide::Above.opposite(), PopupSide::Below);
        assert_eq!(PopupSide::Right.opposite(), PopupSide::Left);
        assert_eq!(PopupSide::Left.opposite(), PopupSide::Right);
    }
}
