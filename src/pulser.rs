
use crate::context;

#[derive(Debug, Clone)]
pub enum PulseShape {
    Flat,
    Square,
    Triangle,
    SawTooth,
    SqrTooth,
    SawDecay,
    SqrDecay,
    Sine,
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
        PulseShape::SqrTooth => {
            if pos >= 0.0 && pos < 1.0 {
                pos*pos
            }
            else {
                0.0
            }
        },
        PulseShape::SawDecay => {
            if pos >= 0.0 && pos < 1.0 {
                1.0 - pos
            }
            else {
                0.0
            }
        },
        PulseShape::SqrDecay => {
            if pos >= 0.0 && pos < 1.0 {
                (1.0-pos)*(1.0-pos)
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
        PulseShape::Sine => {
            if pos >= 0.0 && pos < 1.0 {
                0.5 - 0.5 * (2.0*std::f32::consts::PI*pos).cos()
            }
            else {
                0.0
            }
        },
    }                
}

pub struct Pulse {
    birth: f64,
    pub spaceshape: PulseShape,
    pub timeshape: PulseShape,
}

pub struct Pulser {
    birth: f64,
    nextpulse: f64,
    pulses: Vec<Pulse>,
}

impl Pulser {
    pub fn new(ctx: &context::RunContext) -> Pulser {
        Pulser {
            birth: ctx.age(),
            nextpulse: 0.5,
            pulses: Vec::new(),
        }
    }

    pub fn tick(&mut self, ctx: &context::RunContext) {
        let age = ctx.age() - self.birth;
        if age >= self.nextpulse && self.pulses.is_empty() {
            self.pulses.push(Pulse {
                birth: ctx.age(),
                spaceshape:PulseShape::Triangle,
                timeshape:PulseShape::Triangle });
        }
    }

    pub fn render(&self, ctx: &context::RunContext, buf: &mut [f32]) {
        let bufrange = buf.len() as f32;
        buf.fill(0.0);
        if !self.pulses.is_empty() {
            for ix in 0..buf.len() {
                let pos = (ix as f32) / bufrange;
                let mut val = 0.0;
                for pulse in &self.pulses {
                    let time = (ctx.age() - pulse.birth) as f32;
                    let spaceval = samplepulse(&pulse.spaceshape, pos);
                    let timeval = samplepulse(&pulse.timeshape, time);
                    val += spaceval * timeval;
                }
                buf[ix] = val;
            }
        }
    }
}
