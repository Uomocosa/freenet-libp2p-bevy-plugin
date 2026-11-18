#[must_use]
pub fn ease_out_cubic(progress: f32) -> f32 {
    let inverted = 1.0 - progress.clamp(0.0, 1.0);
    (inverted * inverted).mul_add(-inverted, 1.0)
}
