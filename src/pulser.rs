
use crate::context;

#[derive(Debug, Clone)]
pub enum PulseShape {
    Flat,
    Square,
    Triangle,
    SawTooth,
}

fn samplepulse(shape: &PulseShape, pos: f32) -> f32 {
    match shape {
        PulseShape::Flat => 1.0,
        PulseShape::Square => {
            if pos >= 0.0 && pos < 1.0 {
                1.0
            }
            else {
                0.0
            }
        },
        PulseShape::SawTooth => {
            if pos >= 0.0 && pos < 1.0 {
                pos
            }
            else {
                0.0
            }
        },
        PulseShape::Triangle => {
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
    }                
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
            self.pulses.push(Pulse { spaceshape:PulseShape::Triangle, timeshape:PulseShape::Triangle });
        }
    }

    pub fn render(&self, ctx: &context::RunContext, buf: &mut [f32]) {
        let bufrange = buf.len() as f32;
        buf.fill(0.0);
        if !self.pulses.is_empty() {
            let time = ctx.age as f32 / 60.0;
            for ix in 0..buf.len() {
                let pos = (ix as f32) / bufrange;
                let mut val = 0.0;
                for pulse in &self.pulses {
                    let spaceval = samplepulse(&pulse.spaceshape, pos);
                    let timeval = samplepulse(&pulse.timeshape, time);
                    val += spaceval * timeval;
                }
                buf[ix] = val;
            }
        }
    }
}
