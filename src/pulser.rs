
#[derive(Debug, Clone)]
pub enum PulseShape {
    Flat,
    Square,
    Triangle,
    SawTooth,
}

pub struct Pulse {
    pub spaceshape: PulseShape,
    pub timeshape: PulseShape,
}

pub struct Pulser {
    pulses: Vec<Pulse>,
}

impl Pulser {
    pub fn new() -> Pulser {
        Pulser {
            pulses: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        if self.pulses.is_empty() {
            self.pulses.push(Pulse { spaceshape:PulseShape::Triangle, timeshape:PulseShape::Flat });
        }
    }

    pub fn render(&self, buf: &mut [f32]) {
        let bufrange = buf.len() as f32;
        buf.fill(0.0);
        if !self.pulses.is_empty() {
            for ix in 0..buf.len() {
                let pos = (ix as f32) / bufrange;
                let mut val = 0.0;
                for pulse in &self.pulses {
                    let spaceval = if pos >= 0.0 && pos < 0.5 {
                        pos * 2.0
                    }
                    else if pos >= 0.5 && pos < 1.0 {
                        (1.0 - pos) * 2.0
                    }
                    else {
                        0.0
                    };
                    val += spaceval;
                }
                buf[ix] = val;
            }
        }
    }
}
