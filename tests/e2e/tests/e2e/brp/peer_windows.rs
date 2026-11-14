/// Window origins for the peers of a scenario. Windows are 960x540 on the 1280x800 test
/// display, so the later ones overlap the first two; each canvas centre stays on screen.
pub const PEER_WINDOWS: [(i32, i32); 4] = [(0, 0), (481, 0), (0, 260), (481, 260)];
