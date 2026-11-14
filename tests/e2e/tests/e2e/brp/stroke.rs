/// A hand stroke in board cells, independent of the window size.
#[derive(Debug, Clone, Copy)]
pub struct Stroke(pub &'static [(f32, f32)]);
