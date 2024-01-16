
pub trait Lerp<Rhs=Self> {
    fn lerp(&self, other: &Rhs, frac: &Rhs) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &f32, frac: &f32) -> f32 {
        self * (1.0 - frac) + other * frac
    }
}
