
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
    pub fn as_hex(&self) -> String {
        format!("${:02x}{:02x}{:02x}", (self.r*255.0) as u8, (self.g*255.0) as u8, (self.b*255.0) as u8)
    }
    
    pub fn lerp(&self, other: &Pix<f32>, pos: f32) -> Pix<f32> {
        Pix {
            r: self.r * (1.0-pos) + other.r * pos,
            g: self.g * (1.0-pos) + other.g * pos,
            b: self.b * (1.0-pos) + other.b * pos,
        }
    }

    pub fn from_hsv(hue: f32, sat: f32, value: f32) -> Pix<f32> {
        let chr = value * sat;
        let hp = hue.rem_euclid(1.0) * 6.0;
        let xp = chr * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
        let (rval, gval, bval) = match hp as u8 {
            0 => (chr, xp, 0.0),
            1 => (xp, chr, 0.0),
            2 => (0.0, chr, xp),
            3 => (0.0, xp, chr),
            4 => (xp, 0.0, chr),
            5 => (chr, 0.0, xp),
            6 => (chr, xp, 0.0),
            _ => panic!("hsv math is wrong"),
        };
        let m = value - chr;
        Pix::new(rval+m, gval+m, bval+m)
    }
}

