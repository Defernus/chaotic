use nannou::image;

pub trait ChaoticSystem {
    /// Mutates the system by a `mutation` factor.
    fn mutate(&mut self, mutation: f64);

    /// Updates the system state by a time step `dt`.
    fn update(&mut self, dt: f64);

    /// Creates a new system instance by interpolating between `self` and `other` at a factor `t`
    /// (between `0` and `1`).
    fn lerp(&self, other: &Self, t: f64) -> Self;

    /// Returns the RGB color representation of the system.
    fn color(&self) -> image::Rgba<u8>;
}
