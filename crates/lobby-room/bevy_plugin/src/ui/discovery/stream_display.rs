use bevy::prelude::Resource;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StreamDisplay {
    #[default]
    Panel,
    Fullscreen,
}

#[rustfmt::skip]
impl StreamDisplay {
    pub const fn toggle(&mut self) { *self = self.toggled(); }

    pub const fn show_panel(&mut self) { *self = Self::Panel; }
}

impl StreamDisplay {
    #[must_use]
    pub const fn toggled(&self) -> Self {
        match self {
            Self::Panel => Self::Fullscreen,
            Self::Fullscreen => Self::Panel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamDisplay;

    #[test]
    fn test_usage() {
        let mut display = StreamDisplay::default();
        display.toggle();
        assert_eq!(display, StreamDisplay::Fullscreen);
        display.show_panel();
        assert_eq!(display, StreamDisplay::Panel);
    }

    #[test]
    fn toggle_flips_between_the_two_displays() {
        assert_eq!(StreamDisplay::Panel.toggled(), StreamDisplay::Fullscreen);
        assert_eq!(StreamDisplay::Fullscreen.toggled(), StreamDisplay::Panel);
    }

    #[test]
    fn show_panel_returns_from_fullscreen() {
        let mut display = StreamDisplay::Fullscreen;
        display.show_panel();
        assert_eq!(display, StreamDisplay::Panel);
        display.show_panel();
        assert_eq!(display, StreamDisplay::Panel);
    }
}
