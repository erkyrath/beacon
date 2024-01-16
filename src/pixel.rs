use crate::lerp::Lerp;

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
        format!("${:02X}{:02X}{:02X}", (self.r*255.0) as u8, (self.g*255.0) as u8, (self.b*255.0) as u8)
    }
    
    pub fn lerp(&self, other: &Pix<f32>, pos: f32) -> Pix<f32> {
        Pix {
            r: self.r.lerp(&other.r, &pos),
            g: self.g.lerp(&other.g, &pos),
            b: self.b.lerp(&other.b, &pos),
        }
    }

    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let value = self.r.max(self.g).max(self.b);
        if value <= 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let m = self.r.min(self.g).min(self.b);
        let sat = (value - m) / value;
        let hue: f32;
        if self.r <= m {
            if self.g > self.b {
                hue = 2.0 + (self.b - m) / (self.g - m);
            }
            else {
                hue = 4.0 - (self.g - m) / (self.b - m);
            }
        }
        else if self.g <= m {
            if self.b > self.r {
                hue = 4.0 + (self.r - m) / (self.b - m);
            }
            else {
                hue = 6.0 - (self.b - m) / (self.r - m);
            }
        }
        else if self.b <= m {
            if self.r >= self.g {
                hue = (self.g - m) / (self.r - m);
            }
            else {
                hue = 2.0 - (self.r - m) / (self.g - m);
            }
        }
        else {
            panic!("to_hsv math is wrong");
        }
        (hue/6.0, sat, value)
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

