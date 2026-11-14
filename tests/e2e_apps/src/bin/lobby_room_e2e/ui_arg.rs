use clap::ValueEnum;
use lobby_room::app::UiChoice;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UiArg {
    Default,
    Custom,
}

impl UiArg {
    #[must_use]
    pub const fn choice(self) -> UiChoice {
        match self {
            Self::Default => UiChoice::Default,
            Self::Custom => UiChoice::Custom,
        }
    }
}

#[cfg(test)]
mod tests {
    use lobby_room::app::UiChoice;

    use super::UiArg;

    #[test]
    fn test_usage() {
        assert_eq!(UiArg::Default.choice(), UiChoice::Default);
        assert_eq!(UiArg::Custom.choice(), UiChoice::Custom);
    }
}
