/// Window origins for the three peers of a scenario. Windows are 960x540 on the 1280x800 test
/// display, so the third one overlaps the first two; each canvas centre stays on screen.
pub const PEER_WINDOWS: [(i32, i32); 3] = [(0, 0), (481, 0), (0, 260)];
