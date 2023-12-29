
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Pix<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl<T: Copy> Pix<T> {
    pub fn new(valr: T, valg: T, valb: T) -> Pix<T> {
        Pix { r:valr, g:valg, b:valb }
    }
    
    pub fn grey(val: T) -> Pix<T> {
        Pix { r:val, g:val, b:val }
    }
}

impl Pix<f32> {
    pub fn lerp(&self, other: &Pix<f32>, pos: f32) -> Pix<f32> {
        Pix {
            r: self.r * (1.0-pos) + other.r * pos,
            g: self.r * (1.0-pos) + other.g * pos,
            b: self.r * (1.0-pos) + other.b * pos,
        }
    }
}

