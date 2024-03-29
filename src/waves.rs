#[derive(Debug, Clone, Copy)]
pub enum WaveShape {
    Flat,
    Square,
    HalfSquare,
    Triangle,
    Trapezoid,
    SawTooth,
    SqrTooth,
    SawDecay,
    SqrDecay,
    Sine,
}

impl WaveShape {
    pub fn sample(&self, pos: f32) -> f32 {
        match self {
            WaveShape::Flat => 1.0,
            WaveShape::Square => {
                if pos >= 0.0 && pos < 1.0 {
                    1.0
                }
                else {
                    0.0
                }
            },
            WaveShape::HalfSquare => {
                if pos >= 0.0 && pos < 0.5 {
                    1.0
                }
                else {
                    0.0
                }
            },
            WaveShape::SawTooth => {
                if pos >= 0.0 && pos < 1.0 {
                    pos
                }
                else {
                    0.0
                }
            },
            WaveShape::SqrTooth => {
                if pos >= 0.0 && pos < 1.0 {
                    pos*pos
                }
                else {
                    0.0
                }
            },
            WaveShape::SawDecay => {
                if pos >= 0.0 && pos < 1.0 {
                    1.0 - pos
                }
                else {
                    0.0
                }
            },
            WaveShape::SqrDecay => {
                if pos >= 0.0 && pos < 1.0 {
                    (1.0-pos)*(1.0-pos)
                }
                else {
                    0.0
                }
            },
            WaveShape::Triangle => {
                if pos >= 0.0 && pos < 0.5 {
                    pos * 2.0
                }
                else if pos >= 0.5 && pos < 1.0 {
                    (1.0 - pos) * 2.0
                }
                else {
                    0.0
                }
            },
            WaveShape::Trapezoid => {
                if pos >= 0.0 && pos < 0.25 {
                    pos * 4.0
                }
                else if pos >= 0.75 && pos < 1.0 {
                    (1.0 - pos) * 4.0
                }
                else if pos >= 0.25 && pos < 0.75 {
                    1.0
                }
                else {
                    0.0
                }
            },
            WaveShape::Sine => {
                if pos >= 0.0 && pos < 1.0 {
                    0.5 - 0.5 * (2.0*std::f32::consts::PI*pos).cos()
                }
                else {
                    0.0
                }
            },
        }                
    }
}
